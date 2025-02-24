use std::{fs, io, thread};
use std::net::{Ipv4Addr, Shutdown, TcpListener, TcpStream};
use crate::sctp::sctp_api::{SctpEventSubscribeBuilder, SctpPeerBuilder, MAX_STREAM_NUMBER};
use crate::sctp::sctp_client::{SctpStream, SctpStreamBuilder};
use io::Result;
use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, LazyLock, Mutex, RwLock};
use std::sync::atomic::AtomicU32;
use std::thread::JoinHandle;
use std::time::Duration;
use http::Uri;
use memmap2::{MmapMut, MmapOptions};
use crate::http_parsers::{basic_http_get_request, encode_path, extract_http_paths, http_request_to_string, http_response_to_string, string_to_http_request, string_to_http_response};
use crate::libc_wrappers::{debug_sctp_sndrcvinfo, new_sctp_sndrinfo, SctpSenderInfo};
use crate::cache::lru_cache::TempFileCache;
use crate::constants::{BYTE, KILOBYTE, MEGABYTE};
use crate::packets::byte_packet::BytePacket;
use crate::packets::chunk_type::FilePacketType;
use crate::pools::thread_pool::ThreadPool;

const BUFFER_SIZE: usize = 64 * KILOBYTE;
const CACHE_CAPACITY: usize = 100 * MEGABYTE;
const CHUNK_SIZE: usize = 64 * KILOBYTE;

const DOWNLOAD_THREADS: usize = 6;
const CACHE_PATH: &str = "/tmp/tmpfs";
const DOWNLOAD_SUFFIX: &str = ".tmp";

///Maps each payload protocol id to the requested file name (not encoded).
static PPID_MAP: LazyLock<RwLock<HashMap<u32,String>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Maps each payload protocol id to the current number of chunks that need to be processed.
static PROCESSED_CHUNKS_COUNT: LazyLock<Mutex<HashMap<u32,u16>>> = LazyLock::new(|| Mutex::new(HashMap::new()));


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
            .build();

        sctp_client.connect();
        sctp_client.options();


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

                let path = match path.trim() {
                    "/" => "/index.html".to_string(),
                    _ => {
                        path.to_string()
                    }
                };

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
            let mut sender_info = new_sctp_sndrinfo();
            let mut download_pool = ThreadPool::new(DOWNLOAD_THREADS);

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
                        
                        // get the ppid
                        let ppid = sender_info.sinfo_ppid as u32;


                        let mut byte_packet = BytePacket::from(&buffer[..bytes_read]);
                        let res = Self::parse_metadata_packet(&mut byte_packet,ppid);

                        if res.is_ok(){
                            continue;
                        }


                        download_pool.execute(move || {

                            let res = Self::parse_chunk_packet(&mut byte_packet,ppid);

                            if res.is_err(){
                                panic!("Wrong packet type")
                            }

                            // let file_path = Self::get_file_path(ppid);
                            //
                            // // Parse the received chunk packet
                            // // chunk_index + total_chunks + chunk_size + file_size + content
                            // let mut byte_packet = BytePacket::from(&buffer[..bytes_read]);
                            //
                            // let chunk_index = byte_packet.read_u16().expect("Unable to read chunk index");
                            // let expected_chunk_num = byte_packet.read_u16().expect("Unable to read expected chunk num");
                            // let original_chunk_size = byte_packet.read_u16().expect("Unable to read chunk size");
                            // let file_size = byte_packet.read_u64().expect("Unable to read file size");
                            // let file_chunk = byte_packet.read_all().expect("Unable to read chunk");
                            // let current_chunk_size = bytes_read - 14 * BYTE;
                            //
                            // let chunk_begin = chunk_index as usize * original_chunk_size as usize;
                            // let chunk_end = chunk_begin + current_chunk_size;
                            //
                            // // Open the already existing file
                            // let file = OpenOptions::new()
                            //     .read(true)
                            //     .write(true)
                            //     .create(false)
                            //     .open(&file_path)
                            //     .expect(format!("Unexpected file that does not exist: {}",file_path).as_str());
                            //
                            // // Set the file size if necessary
                            //
                            // {
                            //     // Read the map, and get a read lock to the flag value
                            //     let file_resized = FILE_RESIZED.read().unwrap();
                            //     let flag_lock = file_resized.get(&ppid).unwrap();
                            //     let flag_value = flag_lock.read().unwrap();
                            //
                            //     // Check if the file was resized already
                            //     if !*flag_value{
                            //
                            //         // Drop the read guard
                            //         drop(flag_value);
                            //
                            //         // Get a write guard
                            //         let mut flag_value = flag_lock.write().unwrap();
                            //
                            //         // Check again if the file still needs to be resized and do it
                            //         if !*flag_value{
                            //             *flag_value = true;
                            //             file.set_len(file_size).unwrap();
                            //
                            //         }
                            //
                            //     }
                            // }
                            //
                            //
                            //
                            //
                            // // Map the file and write the chunk
                            // let mut mmap = unsafe{MmapMut::map_mut(&file).unwrap()};
                            //
                            // mmap[chunk_begin..chunk_end].copy_from_slice(file_chunk);
                            //
                            // // Add 1 to the total processed chunks
                            // let chunk_count = {
                            //     let mut chunk_map = PROCESSED_CHUNKS_COUNT.lock().unwrap();
                            //
                            //      *chunk_map.entry(ppid)
                            //         .and_modify(|count| *count += 1)
                            //         .or_insert(1)
                            // };
                            //
                            // // Rename the file to mark it as completed
                            // if chunk_count == expected_chunk_num{
                            //
                            //
                            //     let file_path_clone = file_path.clone();
                            //     let new_file_path = file_path.strip_suffix(DOWNLOAD_SUFFIX).unwrap();
                            //     println!("Renaming file {new_file_path}");
                            //
                            //     fs::rename(file_path_clone, new_file_path).expect("Unable to rename file");
                            //
                            // }


                        })

                    }

                }

            }

            Ok(())
        })

    }

    /// Checks if the current byte packet is a first metadata packet of a new file to be downloaded.
    /// When the packet is just a chunk packet, the function returns and resets the byte packet offset.
    fn parse_metadata_packet(byte_packet: &mut BytePacket,ppid: u32)-> std::result::Result<(),()>{

        // Parse the packet type and end the function if it is not of metadata type
        let packet_type = FilePacketType::from(byte_packet.read_u8().unwrap());
        match packet_type {
            FilePacketType::Metadata => (),
            _ => {
                byte_packet.seek(0);
                return Err(());
            }
        }

        let chunk_count = byte_packet.read_u16().unwrap();
        let file_size = byte_packet.read_u64().unwrap();
        let file_path_bytes = byte_packet.read_all().unwrap();
        let file_path = String::from_utf8_lossy(&file_path_bytes);

        let mut cache_file_name = encode_path(&file_path);
        cache_file_name += DOWNLOAD_SUFFIX;
        let cache_file_path = PathBuf::from(CACHE_PATH).join(cache_file_name);

        // Create the file and set its length
        let file = File::create(cache_file_path).expect("Could not create cache file");
        file.set_len(file_size).expect("Could not set cache file size");


        // Insert an entry into the ppid map and processed chunks
        PPID_MAP.write().unwrap().insert(ppid,file_path.to_string());
        PROCESSED_CHUNKS_COUNT.lock().unwrap().insert(ppid,chunk_count);

        Ok(())
    }

    fn parse_chunk_packet(byte_packet: &mut BytePacket,ppid: u32) -> std::result::Result<(),()> {

        // Extract the packet data
        let packet_type = FilePacketType::from(byte_packet.read_u8().unwrap());
        let chunk_index = byte_packet.read_u16().unwrap();
        let file_chunk = byte_packet.read_all().unwrap();
        let chunk_size = file_chunk.len();

        // Get the cache file path of the file, open and map it into memory
        let mut cache_file_name = {
            let ppid_map = PPID_MAP.read().unwrap();
            let file_name = ppid_map.get(&ppid).unwrap();
            encode_path(file_name)
        };

        cache_file_name += DOWNLOAD_SUFFIX;
        let cache_file_path = Path::new(CACHE_PATH).join(cache_file_name);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(false)
            .open(&cache_file_path)
            .expect("Could not open cache file");

        let mut mmap = unsafe{MmapMut::map_mut(&file).expect("Could not map file")};

        // Compute the start file index based on the type of packet
        let chunk_begin = match packet_type{
            FilePacketType::Chunk => chunk_index as usize * chunk_size,
            FilePacketType::LastChunk => mmap.len() - chunk_size,
            _ => return Err(()),
        };
        let chunk_end = chunk_begin + chunk_size;

        mmap[chunk_begin..chunk_end].copy_from_slice(&file_chunk);

        // Decrement the processed chunks number
        let remaining_chunks = {
            let mut processed_chunks = PROCESSED_CHUNKS_COUNT.lock().unwrap();
            let entry = processed_chunks.entry(ppid).and_modify(|count| *count -= 1);
            processed_chunks.get(&ppid).unwrap().clone()
        };

        // Rename the file if it was done downloading
        if remaining_chunks == 0{

                let cache_file_stem = PathBuf::from(cache_file_path.file_stem().unwrap());
                let new_file_path = PathBuf::from(CACHE_PATH).join(cache_file_stem);
                println!("Renaming file {} to {}",cache_file_path.display(),new_file_path.display());

                fs::rename(cache_file_path, new_file_path).expect("Unable to rename file");

        }

        Ok(())

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