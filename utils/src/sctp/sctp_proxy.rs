use std::{fs, io, thread};
use std::net::{Ipv4Addr};
use crate::sctp::sctp_api::{SctpEventSubscribeBuilder, SctpPeerBuilder, SctpSenderReceiveInfo};
use crate::sctp::sctp_client::{SctpStream, SctpStreamBuilder};
use io::Result;
use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Read, Write};
use std;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{mpsc, Arc, LazyLock, Mutex, RwLock};
use std::thread::JoinHandle;
use inotify::{EventMask, Inotify, WatchMask};
use memmap2::Mmap;
use path_clean::PathClean;
use crate::config::sctp_proxy_config::SctpProxyConfig;
use crate::http_parsers::{decode_path, encode_path, extract_http_paths};
use crate::constants::{KILOBYTE, PACKET_BUFFER_SIZE};
use crate::libc_wrappers::CStruct;
use crate::packets::byte_packet::BytePacket;
use crate::packets::chunk_type::FilePacketType;
use crate::packets::status_code::StatusCode::Unknown;


/// Abstraction for a tcp to sctp proxy
/// The tcp server will listen on a given address and redirect its data to the sctp client
/// The client will connect to the sctp-server using its addresses and send the data to be processes
pub struct SctpProxy{
    port: u16,
    sctp_peer_addresses: Vec<Ipv4Addr>,

    tcp_child: Option<Child>,
    sender_sctp_thread: Option<JoinHandle<Result<()>>>,
    receiver_sctp_thread: Option<JoinHandle<Result<()>>>,
    prefetch_thread: Option<JoinHandle<Result<()>>>,
    tcp_child_reader_thread: Option<JoinHandle<Result<()>>>,
}

impl SctpProxy{
    /// Method that starts the proxy
    pub fn start(mut self) -> Result<()>{

        // Check the validity of the executable path
        if !SctpProxyConfig::browser_child_exec_path().exists(){
            panic!("Browser connection executable path does not exist");
        }

        // Cache setup
        create_dir_all(SctpProxyConfig::cache_path())?;

        // Sctp client setup
        let events = SctpEventSubscribeBuilder::new().sctp_data_io_event().build();

        let mut sctp_client = SctpStreamBuilder::new()
            .socket()
            .port(self.port)
            .addresses(self.sctp_peer_addresses.clone())
            .ttl(0)
            .events(events)
            .set_outgoing_streams(10)
            .set_incoming_streams(24)
            .build();

        sctp_client.connect();
        sctp_client.events();

        // Browser child setup
        self.tcp_child = Some(
            Command::new(SctpProxyConfig::browser_child_exec_path())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to spawn browser connection child")
        );

        println!("Browser connection child started!");

        // Channel used to communicate between multiple tcp receiver threads and the transmitter sctp thread
        let (sctp_tx,sctp_rx) = mpsc::channel();

        let sender_sctp_stream = sctp_client.try_clone()?;
        let receiver_sctp_stream = sctp_client.try_clone()?;

        // Run the sctp client threads
        self.prefetch_thread = Some(Self::sender_sctp_thread(sender_sctp_stream,sctp_rx));
        self.receiver_sctp_thread = Some(Self::receiver_sctp_thread(receiver_sctp_stream));
        self.prefetch_thread = Some(Self::prefetch_thread(sctp_tx.clone()));

        // The main thread becomes the bridge between the child and the sctp proxy
        let child_stdout = self.tcp_child.unwrap().stdout.take().unwrap();
        Self::get_browser_requests(child_stdout,sctp_tx)?;

        Ok(())
    }

    /// Reads the requests send by the browser process from its stdout.
    fn get_browser_requests(child_channel: ChildStdout, sctp_tx: Sender<PathBuf>) -> Result<()>{

        let reader = BufReader::new(child_channel);

        for request in reader.lines(){
            let request = PathBuf::from(request?);
            sctp_tx.send(request).map_err(
                |e| Error::new(ErrorKind::Other,format!("Transmitter send error: {}",e))
            )?;
        }

        Ok(())
    }

    /// Sctp thread that sends incoming requests to the server to be processed.
    /// Each request is mapped to a unique ppid value.
    ///
    fn sender_sctp_thread(sctp_client: SctpStream, sctp_rx: Receiver<PathBuf>) -> JoinHandle<Result<()>>{

        println!("Sctp sender thread started!");

        let sctp_status = sctp_client.get_sctp_status();
        let outgoing_stream_num = sctp_status.sstat_outstrms;
        let mut downloaded_files = HashSet::new();

        thread::spawn(move || {

            let mut stream_number = 0u16;

            for path in sctp_rx {

                // Sanitize the path
                let path = String::from(path.to_str().unwrap());
                let file_path = match path.trim() {
                    "/" => "/index.html".to_string(),
                    _ => {
                        path.to_string()
                    }
                };

                //Check if the file is already in the cache
                match downloaded_files.get(file_path.as_str()){
                    Some(_) => continue,
                    None => (),
                };

                downloaded_files.insert(file_path.clone());

                // Send the request to the server
                sctp_client.write_all(path.as_bytes(),stream_number,0,0)?;

                // Round-robin over the streams
                stream_number = (stream_number + 1) % outgoing_stream_num;


            }

            Ok(())
        })

    }

    /// Thread that receives the paths of downloaded files. Parses the html files in order to make requests in advance, before the browser does.
    pub fn prefetch_thread(sctp_tx: Sender<PathBuf>) -> JoinHandle<Result<()>> {

        // Initialize inotify
        let mut inotify = Inotify::init()
            .expect("Error while initializing inotify instance");

        // Configure inotify to only look for moved files events
        inotify
            .watches()
            .add(
                SctpProxyConfig::cache_path(),
                WatchMask::MOVED_TO,
            )
            .expect("Failed to add file watch");

        let mut events_buffer = vec![0u8; 16 * KILOBYTE];

        // Spawn a thread that reads in a loop the events
        thread::spawn(move || {

            loop {
                let events = inotify.read_events_blocking(&mut events_buffer)
                    .expect("Error while reading events");

                for event in events {

                    // File downloaded
                    if event.mask.contains(EventMask::MOVED_TO){

                        let file_path = Path::new(event.name.unwrap());

                        // Check the file extension
                        if let Some(extension) = file_path.extension(){
                            if extension != "html"{
                                continue;
                            }
                        }
                        else{
                            continue;
                        }

                        let file_path = PathBuf::from(SctpProxyConfig::cache_path()).join(file_path);

                        // Read the file and parse it
                        let file = OpenOptions::new()
                            .read(true)
                            .create(false)
                            .truncate(false)
                            .open(&file_path).expect(format!("Could not open file to prefetch {:?}",file_path).as_str());


                        let mmap = unsafe{Mmap::map(&file).unwrap()};
                        let file_content = String::from_utf8_lossy(&mmap);
                        let prefetched_file_names = extract_http_paths(&file_content);

                        // Need to resolve each prefetched paths
                        let server_path = file_path.strip_prefix(SctpProxyConfig::cache_path())
                            .unwrap()
                            .as_os_str()
                            .to_str()
                            .unwrap();

                        // Get the parent of this html file in server side format
                        let decoded_server_path = PathBuf::from(decode_path(server_path));
                        let parent_path = decoded_server_path.parent().unwrap();

                        // Send the file requests to the sctp sender
                        for file_name in prefetched_file_names{
                            // Join the parent of this html file with the file to be prefetched and clean the path
                            let file_path = parent_path.join(file_name).clean();

                            sctp_tx.send(file_path).map_err(
                                |e| Error::new(ErrorKind::Other,format!("Transmitter send error: {}",e))
                            )?;
                        }


                    }

                }
            }

            Ok(())
        })
    }

    /// Sctp thread that reads the incoming messages of the server.
    /// The server sends chunked files that need to be downloaded.
    /// Each file is identified through a unique ppid value.
    /// After the message is received, it is sent to a download thread pool to be processed.
    ///
    fn receiver_sctp_thread(sctp_client: SctpStream) -> JoinHandle<Result<()>>{

        println!("Sctp receiver thread started!");

        thread::spawn(move || {

            // Init a new thread pool that will download the files
            let mut sender_info = SctpSenderReceiveInfo::new();

            // The number of workers will be equal to the incoming stream count of the sctp association
            let incoming_stream_count = sctp_client.get_sctp_status().sstat_instrms;

            let mut download_pool = DownloaderPool::new(incoming_stream_count as usize);

            let mut buffer = vec![0;PACKET_BUFFER_SIZE];

            loop{

                match sctp_client.read(&mut buffer,Some(&mut sender_info),None){

                    Err(error) => return Err(From::from(error)),

                    Ok(0) =>{
                        println!("Sctp connection closed!");
                        break;
                    }

                    Ok(bytes_read) => {

                        let stream_number = sender_info.sinfo_stream;
                        let byte_packet = BytePacket::from(&buffer[..bytes_read]);

                        // Send the packet to be downloaded by the designated thread
                        download_pool.send_packet(byte_packet,stream_number as usize)?;


                    }

                }

            }

            Ok(())
        })

    }

    fn cache_downloading_path(server_file_path: &str) -> PathBuf {
        let cache_file_name = encode_path(server_file_path) + SctpProxyConfig::download_suffix();
        let cache_file_path = PathBuf::from(SctpProxyConfig::cache_path()).join(cache_file_name);
        cache_file_path
    }

    fn cache_downloaded_path(server_file_path: &str) -> PathBuf {
        let cache_file_name = encode_path(server_file_path);
        let cache_file_path = PathBuf::from(SctpProxyConfig::cache_path()).join(cache_file_name);
        cache_file_path
    }

}

/// Builder pattern for SctpProxy

pub struct SctpProxyBuilder{

    port: u16,
    sctp_peer_addresses: Vec<Ipv4Addr>,
}

impl SctpProxyBuilder {

    /// Creates a new builder for the proxy
    pub fn new() -> Self {

        Self{
            port: 0,
            sctp_peer_addresses: vec![],
        }
    }

    /// Sets the port
    pub fn port(mut self, port: u16) -> Self {

        self.port = port;
        self
    }

    /// Sets the addresses of the sctp client
    pub fn sctp_peer_addresses(mut self, addresses: Vec<Ipv4Addr>) -> Self {

        self.sctp_peer_addresses = addresses;
        self
    }

    /// Builds the proxy based on the given data
    pub fn build(self) -> SctpProxy{

        SctpProxy{
            port: self.port,
            sctp_peer_addresses: self.sctp_peer_addresses,

            tcp_child: None,
            sender_sctp_thread: None,
            receiver_sctp_thread: None,
            prefetch_thread: None,
            tcp_child_reader_thread: None,
        }
    }
}

/// Metadata needed for each downloading thread about the current file being downloaded.
#[derive(Debug)]
struct DownloadingFileMetadata {
    writer: Option<BufWriter<File>>,
    total_chunks: u16,
    current_chunks: u16,
    download_path: PathBuf,
    downloaded_path: PathBuf,
}

impl DownloadingFileMetadata {

    /// Create a new metadata structure
    fn new(writer: BufWriter<File>, total_chunks: u16, download_path: PathBuf,downloaded_path: PathBuf) -> Self {
        Self{
            writer: Some(writer),
            total_chunks,
            current_chunks: 0,
            download_path,
            downloaded_path,
        }
    }
}

impl Default for DownloadingFileMetadata {
    fn default() -> Self {
        Self{
            writer: None,
            total_chunks: 0,
            current_chunks: 0,
            download_path: PathBuf::new(),
            downloaded_path: PathBuf::new(),
        }
    }
}


/// Thread pool used to download a file coming from a fixed stream.
struct DownloaderPool{

    num_workers: usize,
    workers: Vec<DownloaderWorker>,
    channels: Vec<Sender<BytePacket>>,

}

impl DownloaderPool {
    /// Create a download pool of given size.
    pub fn new(num_workers: usize) -> Self {
        assert!(num_workers > 0);

        let mut workers = Vec::with_capacity(num_workers);
        let mut channels = Vec::with_capacity(num_workers);

        for id in 0..num_workers {
            let (tx,rx) = mpsc::channel();
            workers.push(DownloaderWorker::new(id,rx));
            channels.push(tx);
        }

        Self{
            num_workers,
            workers,
            channels,
        }

    }

    /// Sends a SCTP server packet to a download worker to be processed.
    pub fn send_packet(&mut self, packet: BytePacket, to: usize) -> Result<()>{
        self.channels[to].send(packet).map_err(
            |e| Error::new(ErrorKind::Other,format!("Transmitter send error: {}",e))
        )
    }
}

impl Drop for DownloaderPool{
    fn drop(&mut self){

        // Close the channels
        self.channels.drain(..);
        // Wait the workers
        self.workers.drain(..).for_each(|mut worker|{
            let thread = worker.thread.take().unwrap();
            thread.join().unwrap();
        });

    }
}

/// Worker used in the DownloaderPool.
struct DownloaderWorker{
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl DownloaderWorker {

    /// Starts the worker thread.
    fn new(id: usize, rx: Receiver<BytePacket>) -> Self {

        let thread = thread::spawn(move || {

            let mut metadata = DownloadingFileMetadata::default();
            for mut packet in rx {

                let packet_type = FilePacketType::from(packet.read_u8().unwrap());

                match packet_type {
                    FilePacketType::Metadata => metadata = Self::parse_metadata_packet(&mut packet),
                    FilePacketType::Chunk => Self::parse_chunk_packet(&mut packet,&mut metadata),
                    FilePacketType::Unknown(code) => {
                        let packet_size = packet.packet_size();
                        let residue = packet.read_all().unwrap_or(b"0");
                        let residue = String::from_utf8_lossy(residue);

                        eprintln!("Unknown packet type: {} of size {}, rest of packet: {}", code, packet_size,residue);
                        eprintln!("Last metadata: {:#?}",metadata);
                    },
                }

            }

            println!("DownloaderWorker thread stopped");

        });


        Self{
            id,
            thread: Some(thread),
        }
    }

    /// Parsed an already identified metadata packet.
    fn parse_metadata_packet(byte_packet: &mut BytePacket) -> DownloadingFileMetadata {
        println!("Parsing metadata packet");
        let total_chunks = byte_packet.read_u16().unwrap();
        let server_file_path = String::from_utf8_lossy(byte_packet.read_all().unwrap());
        let download_path = SctpProxy::cache_downloading_path(&server_file_path);
        let downloaded_path = SctpProxy::cache_downloaded_path(&server_file_path);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&download_path).expect("Could not open download file");

        let writer = BufWriter::new(file);

        DownloadingFileMetadata::new(writer, total_chunks, download_path, downloaded_path)

    }

    /// Parses an already identified chunk packet.
    fn parse_chunk_packet(byte_packet: &mut BytePacket,file_metadata: &mut DownloadingFileMetadata){

        // Extract the packet data
        let file_chunk = byte_packet.read_all().unwrap();

        let writer = file_metadata.writer.as_mut().unwrap();

        // Write the chunk
        writer.write_all(file_chunk).unwrap();

        file_metadata.current_chunks += 1;

        // File ended to download
        if file_metadata.total_chunks == file_metadata.current_chunks{
            // Flush the contents of the buffer into the file, drop the buffer and the active mutexes
            writer.flush().unwrap();

            fs::rename(file_metadata.download_path.as_path(), file_metadata.downloaded_path.as_path()).unwrap();
        }

    }

}