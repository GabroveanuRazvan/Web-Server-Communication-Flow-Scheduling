use std::{fs, io, thread};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use crate::sctp::sctp_api::{SctpEventSubscribeBuilder, SctpPeerBuilder, SctpSenderReceiveInfo, MAX_STREAM_NUMBER};
use crate::sctp::sctp_client::{SctpStream, SctpStreamBuilder};
use io::Result;
use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Read, Write};
use std::path::{PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, LazyLock, Mutex, RwLock};
use std::thread::JoinHandle;
use crate::http_parsers::{encode_path};
use crate::constants::{KILOBYTE, MEGABYTE};
use crate::libc_wrappers::CStruct;
use crate::packets::byte_packet::BytePacket;
use crate::pools::indexed_thread_pool::IndexedTreadPool;

const BUFFER_SIZE: usize = 64 * KILOBYTE;
const CACHE_CAPACITY: usize = 100 * MEGABYTE;
const CHUNK_SIZE: usize = 64 * KILOBYTE;

const DOWNLOAD_THREADS: usize = 12;
const CACHE_PATH: &str = "/tmp/tmpfs";
const DOWNLOAD_SUFFIX: &str = ".tmp";

///Maps each payload protocol id to the requested file name (not encoded).
static PPID_MAP: LazyLock<RwLock<HashMap<u32,String>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Maps each payload protocol id to its corresponding opened file when the file is being downloaded
static FILE_MAP: LazyLock<RwLock<HashMap<u32,Mutex<Option<BufWriter<File>>>>>> = LazyLock::new(|| RwLock::new(HashMap::new()));


/// Abstraction for a tcp to sctp proxy
/// The tcp server will listen on a given address and redirect its data to the sctp client
/// The client will connect to the sctp-server using its addresses and send the data to be processes
pub struct SctpProxy{
    port: u16,
    sctp_address: Ipv4Addr,
    sctp_peer_addresses: Vec<Ipv4Addr>,
    tcp_address: Ipv4Addr,
}

impl SctpProxy{
    /// Method that starts the proxy
    pub fn start(self) -> Result<()>{

        let mut tcp_server =TcpListener::bind((self.tcp_address.to_string(),self.port))?;

        println!("Sctp Proxy started and listening on {:?}:{}",self.tcp_address,self.port);
        println!("Messages redirected to: {:?}:{}",self.sctp_address,self.port);

        // cache setup
        create_dir_all(CACHE_PATH)?;

        // sctp client setup
        let events = SctpEventSubscribeBuilder::new().sctp_data_io_event().build();

        let mut sctp_client = SctpStreamBuilder::new()
            .socket()
            .port(self.port)
            .address(self.sctp_address)
            .addresses(self.sctp_peer_addresses.clone())
            .ttl(0)
            .events(events)
            .set_outgoing_streams(10)
            .set_incoming_streams(24)
            .build();

        sctp_client.connect();
        sctp_client.events();

        // channel used to communicate between multiple tcp receiver threads and the transmitter sctp thread
        let (sctp_tx,sctp_rx) = mpsc::channel();

        let sender_sctp_stream = sctp_client.try_clone()?;
        let receiver_sctp_stream = sctp_client.try_clone()?;

        // run the sctp client threads
        Self::sender_sctp_thread(sender_sctp_stream,sctp_rx);
        Self::receiver_sctp_thread(receiver_sctp_stream);

        // for each tcp client init a tcp receiver thread
        for stream in tcp_server.incoming(){

            let stream = stream?;

            let sctp_tx_clone = sctp_tx.clone();

            Self::receiver_tcp_thread(stream,sctp_tx_clone);

        }

        Ok(())
    }

    /// Tcp receiver thread that reads incoming request and sends them to be forwarded by the sctp sender thread.
    ///
    fn receiver_tcp_thread(tcp_stream: TcpStream, sctp_tx: Sender<Vec<u8>>) -> JoinHandle<Result<()>>{

        let reader = BufReader::new(tcp_stream);

        thread::spawn(move || {

            for line in reader.lines(){

                let request = line?;

                sctp_tx.send(request.as_bytes().to_vec()).map_err(
                    |e| Error::new(ErrorKind::Other,format!("Transmitter send error: {}",e))
                )?;

            }

            println!("Tcp connection closed!");
            Ok(())

        })

    }

    /// Sctp thread that sends incoming requests to the server to be processed.
    /// Each request is mapped to a unique ppid value.
    ///
    fn sender_sctp_thread(sctp_client: SctpStream, sctp_rx: Receiver<Vec<u8>>) -> JoinHandle<Result<()>>{

        println!("Sctp sender thread started!");

        thread::spawn(move || {

            let mut stream_number = 0u16;
            let mut current_ppid = 0;

            for request_buffer in sctp_rx {

                let path = String::from_utf8_lossy(&request_buffer);

                let file_path = match path.trim() {
                    "/" => "/index.html".to_string(),
                    _ => {
                        path.to_string()
                    }
                };


                let cache_file_name = encode_path(&file_path) + DOWNLOAD_SUFFIX;
                let cache_file_path = PathBuf::from(CACHE_PATH).join(&cache_file_name);

                // Insert an entry into the ppid map and processed chunks
                PPID_MAP.write().unwrap().insert(current_ppid,file_path.to_string());

                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .read(true)
                    .open(cache_file_path)?;

                // Insert the opened file into its map
                FILE_MAP.write().unwrap().insert(current_ppid,Mutex::new(Some(BufWriter::new(file))));

                // Send the request to the server
                sctp_client.write_all(&request_buffer,stream_number,current_ppid,0)?;

                // Round-robin over the streams
                stream_number = (stream_number + 1) % MAX_STREAM_NUMBER;
                current_ppid += 1;


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

            // init a new thread pool that will download the files
            let mut sender_info = SctpSenderReceiveInfo::new();
            let mut download_pool = IndexedTreadPool::new(DOWNLOAD_THREADS);

            loop{

                // create a new buffer for each request that will be owned by the thread pool
                let mut buffer = vec![0;BUFFER_SIZE];
                match sctp_client.read(&mut buffer,Some(&mut sender_info),None){

                    Err(error) => return Err(From::from(error)),

                    Ok(0) =>{
                        println!("Sctp connection closed!");
                        break;
                    }

                    Ok(bytes_read) => {

                        let ppid = sender_info.sinfo_ppid;
                        let stream_number = sender_info.sinfo_stream;

                        // Send the packet to be downloaded by the designated thread
                        download_pool.execute(stream_number as usize,move || {
                            let mut byte_packet = BytePacket::from(&buffer[..bytes_read]);
                            Self::parse_chunk_packet(&mut byte_packet,ppid);
                        })

                    }

                }

            }

            Ok(())
        })

    }

    /// Parses the received file chunk bytes.
    fn parse_chunk_packet(byte_packet: &mut BytePacket,ppid: u32){

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
            let cache_file_name = PathBuf::from(&Self::get_file_path(ppid));
            let new_file_name = cache_file_name.with_extension("");

            fs::rename(cache_file_name, new_file_name).unwrap();

        }

    }


    /// Based on a payload protocol id, retrieves the file request and formats it into a path to be stored.
    ///
    fn get_file_path(ppid: u32) -> String{

        // lock the RwLock and read the file name
        let ppid_map = PPID_MAP.read().expect("ppid map lock poisoned");
        let file_name = encode_path(ppid_map.get(&ppid).unwrap());

        // get the actual file path
        let file_path = format!("{}/{}{}", CACHE_PATH, file_name, DOWNLOAD_SUFFIX);

        file_path
    }

}


/// Builder pattern for SctpProxy

pub struct SctpProxyBuilder{

    port: u16,
    sctp_address: Ipv4Addr,
    sctp_peer_addresses: Vec<Ipv4Addr>,
    tcp_address: Ipv4Addr,
}

impl SctpProxyBuilder {

    /// Creates a new builder for the proxy
    pub fn new() -> Self {

        Self{
            port: 0,
            sctp_address: Ipv4Addr::new(0, 0, 0, 0),
            sctp_peer_addresses: vec![],
            tcp_address: Ipv4Addr::new(0, 0, 0, 0),
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

    /// Sets the address that will be used to send data
    pub fn sctp_address(mut self, address: Ipv4Addr) -> Self {

        self.sctp_address = address;
        self
    }

    /// Sets the address that the tcp server will listen to
    pub fn tcp_address(mut self, address: Ipv4Addr) -> Self {

        self.tcp_address = address;
        self
    }

    /// Builds the proxy based on the given data
    pub fn build(self) -> SctpProxy{

        SctpProxy{
            port: self.port,
            sctp_address: self.sctp_address,
            sctp_peer_addresses: self.sctp_peer_addresses,
            tcp_address: self.tcp_address,
        }
    }
}