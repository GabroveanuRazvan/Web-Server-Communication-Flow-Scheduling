use std::{io,thread};
use std::net::{Ipv4Addr, Shutdown, TcpListener, TcpStream};
use crate::sctp::sctp_api::{SctpEventSubscribeBuilder, SctpPeerBuilder, MAX_STREAM_NUMBER};
use crate::sctp::sctp_client::{SctpStream, SctpStreamBuilder};
use io::Result;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;
use http::Uri;
use memmap2::MmapMut;
use crate::http_parsers::{basic_http_get_request, encode_path, extracts_http_paths, http_request_to_string, http_response_to_string, string_to_http_request, string_to_http_response};
use crate::libc_wrappers::{debug_sctp_sndrcvinfo, new_sctp_sndrinfo, SctpSenderInfo};
use crate::cache::lru_cache::TempFileCache;
use crate::constants::{KILOBYTE, MEGABYTE};
use crate::pools::thread_pool::ThreadPool;

const BUFFER_SIZE: usize = 64 * KILOBYTE;
const CACHE_CAPACITY: usize = 100 * MEGABYTE;
const CHUNK_SIZE: usize = 64 * KILOBYTE;

const DOWNLOAD_THREADS: usize = 6;
const CACHE_PATH: &str = "/tmp/tmpfs";


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
        println!("Connect by: http://127.0.0.1:{}",self.port);

        // cache setup
        // let mut cache = TempFileCache::new(CACHE_CAPACITY);

        for stream in tcp_server.incoming(){

            let stream = stream?;

            // create a new sctp client

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

            let sctp_client_clone = sctp_client.try_clone()?;

            // Self::handle_client(stream,sctp_client_clone,&mut cache);

            Self::handle_client2(stream, sctp_client_clone);

        }

        Ok(())
    }


    /// Thread that received new requests and sends them to the server using the sctp stream.
    /// Maps each request to a unique payload protocol id.
    ///
    fn sender_thread(mut tcp_stream: TcpStream,sctp_client: SctpStream, ppid_map: Arc<RwLock<HashMap<u32,String>>>) -> JoinHandle<Result<()>>{

        println!("Tcp server waiting for GET requests;");
        let mut stream_number = 0u16;
        let mut current_ppid = 0;
        let mut browser_buffer: Vec<u8> = vec![0;4 * KILOBYTE];

        thread::spawn(move || {

            loop{

                //receive a request
                match tcp_stream.read(&mut browser_buffer){
                    Ok(0) => {
                        println!("Connection ended");
                        break;
                    }

                    Err(error) => {
                        panic!("Tcp Client error: {}", error);
                    }

                    // send the request to be processed by the server
                    Ok(bytes_received) => {

                        println!("Got a tcp request!");

                        // map each request to a payload protocol id
                        let mut ppid_map = ppid_map.write().expect("ppid map lock poisoned");
                        let path = String::from_utf8_lossy(&browser_buffer[..bytes_received]);

                        let path = match path.trim() {
                            "/" => "/index.html".to_string(),
                            _ => {
                                path.to_string()
                            }
                        };

                        let path = String::from(path);
                        let file_name = encode_path(&path);

                        // create the file using the encoded file name
                        let file_path = CACHE_PATH.to_string() + "/" + file_name.as_str();
                        File::create(file_path)?;

                        // map the current ppid to the file name
                        ppid_map.insert(current_ppid,file_name);

                        // send the request to the server
                        sctp_client.write_all(&browser_buffer[..bytes_received],stream_number,current_ppid,0)?;

                        // round-robin over the streams
                        stream_number = (stream_number + 1) % MAX_STREAM_NUMBER;
                        current_ppid += 1;

                    }
                }

            }

            Ok(())

        })

    }


    pub fn receiver_thread(sctp_client: SctpStream,ppid_map: Arc<RwLock<HashMap<u32,String>>>) -> JoinHandle<Result<()>>{

        thread::spawn(move || {

            // init a new thread pool that will download the files
            let mut sender_info = new_sctp_sndrinfo();
            let mut download_pool = ThreadPool::new(DOWNLOAD_THREADS);

            loop{

                // create a new buffer for each request that will be owned by the thread pool
                let mut buffer = vec![0;BUFFER_SIZE];
                match sctp_client.read(&mut buffer,Some(&mut sender_info),None){

                    Err(error) => return Err(From::from(error)),

                    Ok(1) => {
                        //TODO file ended
                        println!("File was processed");
                        debug_sctp_sndrcvinfo(&sender_info);
                    }

                    Ok(bytes_read) => {

                        debug_sctp_sndrcvinfo(&sender_info);

                        // get the ppid and the ppid_map
                        let ppid = sender_info.sinfo_ppid as u32;
                        let ppid_map = Arc::clone(&ppid_map);

                        download_pool.execute(move || {

                            // lock the RwLock and read the file name
                            let ppid_map = ppid_map.read().expect("ppid map lock poisoned");
                            let file_name = encode_path(ppid_map.get(&ppid).unwrap());

                            // get the actual file path
                            let file_path = CACHE_PATH.to_string() + "/" + file_name.as_str();

                            // retrieve the chunk index from the first 4 bytes of the payload
                            let chunk_index = u32::from_be_bytes(buffer[..4].try_into().unwrap());

                            // retrieve the first and last index of the chunk with respect to the first 4 bytes of the payload
                            let chunk_begin = chunk_index as usize * (CHUNK_SIZE-4);
                            let chunk_end = chunk_begin + bytes_read - 4;

                            // open the already existing file
                            let file = OpenOptions::new()
                                .read(true)
                                .write(true)
                                .create(false)
                                .open(&file_path)
                                .unwrap();

                            let file_size = file.metadata().unwrap().len();

                            // resize the file if the size exceeds the current chunk_end size
                            if chunk_end > file_size as usize {
                                file.set_len(chunk_begin as u64 + bytes_read as u64).unwrap();
                            }

                            //map the file and write the chunk
                            let mut mmap = unsafe{MmapMut::map_mut(&file).unwrap()};

                            mmap[chunk_begin..chunk_end].copy_from_slice(&buffer[4..bytes_read]);


                        })

                    }

                }

            }

            Ok(())
        })

    }

    fn handle_client2(mut tcp_stream: TcpStream, sctp_client: SctpStream){

        let ppid_map: Arc<RwLock<HashMap<u32,String>>> =  Arc::new(RwLock::new(HashMap::new()));

        let sctp_reader_client = sctp_client.try_clone().expect("Sctp client cloning error");

        Self::sender_thread(tcp_stream,sctp_client,Arc::clone(&ppid_map));
        Self::receiver_thread(sctp_reader_client,Arc::clone(&ppid_map));

    }
    /// Client handler method
    fn handle_client(mut tcp_stream: TcpStream, sctp_client: SctpStream, cache: &mut TempFileCache){

        // used to RR over the streams
        let mut stream_number = 0u16;

        println!("New client");

        let mut browser_buffer: Vec<u8> = vec![0;BUFFER_SIZE];

        loop{

            println!("Tcp listener waiting for messages...");

            //TODO main thread de read de la browser

            // the tcp stream waits for a request
            match tcp_stream.read(&mut browser_buffer){

                Ok(0) => {
                    println!("Tcp client closed");
                    break;
                }

                Err(error) => {
                    panic!("Tcp Client error: {}", error);
                }

                // request received
                Ok(n) => {
                    let received_message = String::from_utf8_lossy(&browser_buffer[..n]);

                    // get the uri
                    let mut uri = string_to_http_request(received_message.as_ref())
                        .uri()
                        .to_string()
                        .trim_matches('?')
                        .to_string();

                    if uri == "/"{
                        uri = "/index.html".to_string()
                    }

                    // cache hit case
                    if let Some(file) = cache.get(&uri){
                        println!("Cache hit {uri}!");

                        let mapped_file = file.borrow();
                        let mmap_ptr = mapped_file.mmap_as_slice();

                        // send the file in chunks
                        for chunk in mmap_ptr.chunks(CHUNK_SIZE){
                            tcp_stream.write(chunk).expect("Tcp stream write error cache");
                        }

                        continue;
                    }

                    // cache miss case
                    println!("Cache miss {uri}!");
                    // create a cache entry
                    cache.insert(uri.clone());

                    // TODO aici vine un thread care trimite cererile (nu in interiorul threadului principal)

                    // send the request
                    sctp_client.write_all(uri.as_bytes(),stream_number,0,0).expect("Sctp Client write error");

                    // simple RR over the streams
                    stream_number = (stream_number + 1) % MAX_STREAM_NUMBER;

                    let mut sender_info = new_sctp_sndrinfo();

                    // TODO aici vine threadul de receptie care o sa faca in loop read cu select



                    //read the response
                    match sctp_client.read(&mut browser_buffer,Some(&mut sender_info),None){

                        Err(error)=>{
                            panic!("Sctp read error: {}", error);
                        }

                        // response received
                        Ok(n) =>{

                            // write the response header into the cache
                            cache.write_append(&uri,&browser_buffer[..n]).expect("Temporary file write error");

                            // write into tcp stream
                            if let Err(error) = tcp_stream.write(&browser_buffer[..n]){
                                panic!("Tcp write error: {}", error);
                            }

                            println!("Received on stream {}",sender_info.sinfo_stream)
                        }
                    }

                    // TODO tot threadul de receptie
                    // now loop to receive the chunked response body
                    loop{
                        // the sctp-stream waits to get a response
                        match sctp_client.read(&mut browser_buffer,Some(&mut sender_info),None){
                            // end message received
                            Ok(1) => {
                                println!("Sctp client ended processing");
                                break;
                            }

                            Err(error)=>{
                                panic!("Sctp read error: {}", error);
                            }

                            // response chunk received
                            Ok(n) =>{

                                // write to temporary file
                                cache.write_append(&uri,&browser_buffer[..n]).expect("Temporary file write error");

                                // write to tcp stream
                               tcp_stream.write(&browser_buffer[..n]).expect("Tcp stream write error");

                                println!("Received on stream {}",sender_info.sinfo_stream)
                            }
                        }
                    }

                    // TODO aici o sa vina thread poolul de download inauntrul threadului de receptie; fiecare thread face send si read
                    // after caching the file it's time to do some prefetching

                    let mapped_file = cache.get(&uri).unwrap();
                    let borrowed_mapped_file = mapped_file.borrow();
                    let mmap_ptr = borrowed_mapped_file.mmap_as_slice();

                    if uri.ends_with(".html"){

                        let future_uri = extracts_http_paths(String::from_utf8_lossy(mmap_ptr).as_ref());

                        for uri in future_uri {

                            let uri = uri.trim_matches('/');
                            let uri = "/".to_string() + uri;

                            // if the file is already cached just continue
                            if let Some(_) = cache.peek(&uri){
                                continue;
                            }

                            // insert the new value into the cache
                            cache.insert(uri.clone());

                            // send the path to the server
                            sctp_client.write_all(uri.as_bytes(),stream_number,0,0).expect("Sctp Client prefetch write error");

                            // RR over the streams
                            stream_number = (stream_number + 1) % MAX_STREAM_NUMBER;
                            //read the response body
                            match sctp_client.read(&mut browser_buffer,Some(&mut sender_info),None){

                                Err(error)=>{
                                    panic!("Sctp read error: {}", error);
                                }

                                // response received
                                Ok(n) =>{

                                    let response = string_to_http_response(String::from_utf8_lossy(&browser_buffer[..n]).as_ref());
                                    let content_length = response.headers().get("Content-Length").unwrap().to_str().unwrap().parse::<u64>().unwrap();
                                    println!("Received SCTP response length {content_length} from {uri}");

                                    // write to temporary file
                                    cache.write_append(&uri,&browser_buffer[..n]).expect("Temporary file write error");

                                }
                            }

                            // now loop to receive the chunked response body
                            loop{
                                // the sctp-stream waits to get a response
                                match sctp_client.read(&mut browser_buffer,Some(&mut sender_info),None){
                                    // end message received
                                    Ok(1) => {
                                        println!("Sctp client ended processing prefetch");
                                        break;
                                    }

                                    Err(error)=>{
                                        panic!("Sctp read error: {}", error);
                                    }

                                    // response chunk received
                                    Ok(n) =>{

                                        // write to temporary file
                                        cache.write_append(&uri,&browser_buffer[..n]).expect("Temporary file write error");
                                        println!("Received on stream {}",sender_info.sinfo_stream)

                                    }
                                }
                            }

                        }

                    }

                }

            }

        }

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