use std::{fs, io, thread};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use crate::sctp::sctp_api::{SctpEventSubscribeBuilder, SctpPeerBuilder, SctpSenderReceiveInfo};
use crate::sctp::sctp_client::{SctpStream, SctpStreamBuilder};
use io::Result;
use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Read, Write};
use std::num::NonZero;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, LazyLock, Mutex, RwLock};
use std::thread::JoinHandle;
use memmap2::Mmap;
use path_clean::PathClean;
use crate::config::sctp_proxy_config::SctpProxyConfig;
use crate::http_parsers::{decode_path, encode_path, extract_http_paths};
use crate::constants::{PACKET_BUFFER_SIZE};
use crate::libc_wrappers::CStruct;
use crate::packets::byte_packet::BytePacket;
use crate::pools::indexed_thread_pool::IndexedTreadPool;

///Maps each payload protocol id to the requested file name (not encoded).
static PPID_MAP: LazyLock<RwLock<HashMap<u32,String>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Maps each payload protocol id to its corresponding opened file buf writer while the file is being downloaded
static FILE_MAP: LazyLock<RwLock<HashMap<u32,Mutex<Option<BufWriter<File>>>>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Set of the not encoded downloaded files, used to
static DOWNLOADED_FILES: LazyLock<RwLock<HashSet<String>>> = LazyLock::new(|| RwLock::new(HashSet::new()));


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
        let (prefetch_tx, prefetch_rx) = mpsc::channel();

        let sender_sctp_stream = sctp_client.try_clone()?;
        let receiver_sctp_stream = sctp_client.try_clone()?;

        // Run the sctp client threads
        self.prefetch_thread = Some(Self::sender_sctp_thread(sender_sctp_stream,sctp_rx));
        self.receiver_sctp_thread = Some(Self::receiver_sctp_thread(receiver_sctp_stream,prefetch_tx));
        self.prefetch_thread = Some(Self::prefetch_thread(prefetch_rx,sctp_tx.clone()));

        // The main thread becomes the bridge between the child and the sctp proxy
        let child_stdout = self.tcp_child.unwrap().stdout.take().unwrap();
        Self::get_browser_requests(child_stdout,sctp_tx)?;

        Ok(())
    }

    fn get_browser_requests(stdout: ChildStdout, sctp_tx: Sender<PathBuf>) -> Result<()>{

        let reader = BufReader::new(stdout);

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

        thread::spawn(move || {

            let mut stream_number = 0u16;
            let mut current_ppid = 0;

            for path in sctp_rx {

                let path = String::from(path.to_str().unwrap());

                let file_path = match path.trim() {
                    "/" => "/index.html".to_string(),
                    _ => {
                        path.to_string()
                    }
                };

                //Check if the file is already in the cache
                {
                    match DOWNLOADED_FILES.read().unwrap().get(file_path.as_str()){
                        Some(_) => continue,
                        None => (),
                    };

                    DOWNLOADED_FILES.write().unwrap().insert(file_path.clone());
                }

                let cache_file_name = encode_path(&file_path) + SctpProxyConfig::download_suffix();
                let cache_file_path = PathBuf::from(SctpProxyConfig::cache_path()).join(&cache_file_name);

                // Insert an entry into the ppid map and processed chunks
                {
                    PPID_MAP.write().unwrap().insert(current_ppid,file_path.to_string());
                }

                // Insert the opened file into its map
                {
                    let file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .read(true)
                        .open(cache_file_path)?;

                    FILE_MAP.write().unwrap().insert(current_ppid,Mutex::new(Some(BufWriter::new(file))));
                }


                // Send the request to the server
                sctp_client.write_all(path.as_bytes(),stream_number,current_ppid,0)?;

                // Round-robin over the streams
                stream_number = (stream_number + 1) % outgoing_stream_num;
                current_ppid += 1;


            }

            Ok(())
        })

    }


    /// Thread that receives the paths of downloaded files. Parses the html files in order to make requests in advance, before the browser does.
    fn prefetch_thread(prefetch_rx: Receiver<PathBuf>, sctp_tx: Sender<PathBuf>) -> JoinHandle<Result<()>> {

        thread::spawn(move || {

            for file_path in prefetch_rx{

                // Check the file extension
                if let Some(extension) = file_path.extension(){
                    if extension != "html"{
                        continue;
                    }
                }
                else{
                    continue;
                }

                // Read the file and parse it
                let file = OpenOptions::new()
                    .read(true)
                    .create(false)
                    .truncate(false)
                    .open(&file_path).expect("Could not open file to prefetch");


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


            Ok(())
        })

    }

    /// Sctp thread that reads the incoming messages of the server.
    /// The server sends chunked files that need to be downloaded.
    /// Each file is identified through a unique ppid value.
    /// After the message is received, it is sent to a download thread pool to be processed.
    ///
    fn receiver_sctp_thread(sctp_client: SctpStream,prefetch_tx: Sender<PathBuf>) -> JoinHandle<Result<()>>{

        println!("Sctp receiver thread started!");

        thread::spawn(move || {

            // Init a new thread pool that will download the files
            let mut sender_info = SctpSenderReceiveInfo::new();

            // The number of workers will be equal to the incoming stream count of the sctp association
            let incoming_stream_count = sctp_client.get_sctp_status().sstat_instrms;
            let mut download_pool = IndexedTreadPool::new(incoming_stream_count as usize);

            loop{

                // create a new buffer for each request that will be owned by the thread pool
                let mut buffer = vec![0;PACKET_BUFFER_SIZE];
                match sctp_client.read(&mut buffer,Some(&mut sender_info),None){

                    Err(error) => return Err(From::from(error)),

                    Ok(0) =>{
                        println!("Sctp connection closed!");
                        break;
                    }

                    Ok(bytes_read) => {

                        let ppid = sender_info.sinfo_ppid;
                        let stream_number = sender_info.sinfo_stream;
                        let prefetch_tx = prefetch_tx.clone();

                        // Send the packet to be downloaded by the designated thread
                        download_pool.execute(stream_number as usize,move || {
                            let mut byte_packet = BytePacket::from(&buffer[..bytes_read]);
                            Self::parse_chunk_packet(&mut byte_packet,ppid,prefetch_tx);
                        })

                    }

                }

            }

            Ok(())
        })

    }

    /// Parses the received file chunk bytes.
    fn parse_chunk_packet(byte_packet: &mut BytePacket,ppid: u32,prefetch_tx: Sender<PathBuf>){

        // Extract the packet data
        let chunk_index = byte_packet.read_u16().unwrap();
        let total_chunks = byte_packet.read_u16().unwrap();
        let file_chunk = byte_packet.read_all().unwrap();

        // Get the file out of the file map
        let file_map = FILE_MAP.read().unwrap();
        let mut file_ref = file_map.get(&ppid).unwrap().lock().unwrap();
        let file = file_ref.as_mut().unwrap();

        // Write the chunk
        file.write_all(file_chunk).unwrap();

        // File ended to download
        if chunk_index == total_chunks -1{
            // Flush the contents of the buffer into the file, drop the buffer and the active mutexes
            file.flush().unwrap();
            file_ref.take();
            drop(file_ref);
            drop(file_map);

            //Rename the file by deleting the download suffix to mark it as finished
            let cache_file_name = Self::get_file_path(ppid);
            let new_file_name = cache_file_name.with_extension("");

            fs::rename(cache_file_name, &new_file_name).unwrap();
            prefetch_tx.send(new_file_name).unwrap();
        }

    }


    /// Based on a payload protocol id, retrieves the file request and formats it into a path to be stored.
    ///
    fn get_file_path(ppid: u32) -> PathBuf{

        // Lock the RwLock and read the file name
        let ppid_map = PPID_MAP.read().expect("ppid map lock poisoned");
        let file_name = encode_path(ppid_map.get(&ppid).unwrap()) + SctpProxyConfig::download_suffix();
        let file_path = PathBuf::from(SctpProxyConfig::cache_path()).join(file_name);

        file_path
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