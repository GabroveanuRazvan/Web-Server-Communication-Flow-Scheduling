use std::{io,thread};
use std::net::{Ipv4Addr, Shutdown, TcpListener, TcpStream};
use crate::sctp::sctp_api::{SctpEventSubscribeBuilder, SctpPeerBuilder, MAX_STREAM_NUMBER};
use crate::sctp::sctp_client::{SctpStream, SctpStreamBuilder};
use io::Result;
use std::io::{Read, Write};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use http::Uri;
use crate::http_parsers::{basic_http_get_request, extracts_http_paths, http_request_to_string, http_response_to_string, string_to_http_request, string_to_http_response};
use crate::libc_wrappers::{debug_sctp_sndrcvinfo, new_sctp_sndrinfo, SctpSenderInfo};
use crate::cache::lru_cache::TempFileCache;
use crate::constants::{KILOBYTE, MEGABYTE};

const BUFFER_SIZE: usize = 64 * KILOBYTE;
const CACHE_CAPACITY: usize = 100 * MEGABYTE;
const CHUNK_SIZE: usize = 64 * KILOBYTE;

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
        let mut cache = TempFileCache::new(CACHE_CAPACITY);

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


            Self::handle_client(stream,sctp_client,&mut cache)


        }

        Ok(())
    }



    fn tcp_reader_thread(mut tcp_reader_stream: TcpStream,
                         temp_file_cache: &TempFileCache,
                         tcp_writer_tx: Sender<Uri>,
                         sctp_writer_tx: Sender<Uri>) -> JoinHandle<Result<()>>{

        thread::spawn(move || {

            let mut buffer = vec![0; BUFFER_SIZE];

            loop{

                match tcp_reader_stream.read(&mut buffer){

                    Ok(0) => {
                        println!("TCP reader thread ended");
                        break;
                    }

                    Err(error) => return Err(error),

                    Ok(bytes_read) => {

                        // get the request from the buffer and extract the uri
                        let http_request = string_to_http_request(&String::from_utf8_lossy(&buffer[..bytes_read]));
                        let uri = http_request.uri().to_owned();

                        // if the file is in the cache send a job to the tcp writer thread
                        if temp_file_cache.contains_key(&uri.to_string()) {
                            tcp_writer_tx.send(uri)?;
                        }
                        //if the file is not cached send the job to the sctp writer thread
                        else{
                            sctp_writer_tx.send(uri)?;
                        }

                    }

                }
            }
            return Ok(())
        })

    }

    pub fn tcp_writer_thread(mut tcp_writer_stream: TcpStream,tcp_writer_rx: Receiver<Uri>) -> JoinHandle<Result<()>>{

        thread::spawn(move || {

            while let Some(uri) = tcp_writer_rx.recv(){
                // TODO aici le ia din cache si le trimite
            }

            Ok(())
        })

    }

    pub fn sctp_reader_thread(sctp_reader_stream: SctpStream) -> JoinHandle<Result<()>>{

        thread::spawn(move || {

            let mut buffer = vec![0; BUFFER_SIZE];

            loop{

                //read the response
                match sctp_reader_stream.read(&mut buffer,None,None) {
                    Err(error) => return Err(error),

                    // response received
                    Ok(n) => {
                        // TODO: aici stochezi in cache ce primesti prin download th pool
                    }
                }

                loop{
                    match sctp_reader_stream.read(&mut buffer,None,None) {
                        Err(error) => return Err(error),

                        Ok(1) => break,

                        // response received
                        Ok(n) => {
                            // TODO: aici stochezi in cache ce primesti prin download th pool
                        }
                    }
                }

                // TODO prefetching

            }


            Ok(())
        })

    }

    pub fn sctp_writer_thread(sctp_writer_stream: SctpStream,sctp_writer_rx: Receiver<Uri>) -> JoinHandle<Result<()>>{

        thread::spawn(move || {

            let mut stream_number = 0u16;
            loop{

                let uri = sctp_writer_rx.recv()?;
                let mut uri = uri.to_string();

                if uri == "/"{
                    uri = "/index.html".to_string()
                }

                let http_request = http_request_to_string(basic_http_get_request(&uri));

                sctp_writer_stream.write_all(http_request.as_bytes(),stream_number,0)?;
                stream_number = (stream_number + 1) % MAX_STREAM_NUMBER;

            }

            Ok(())
        })

    }


    /// Client handler method
    fn handle_client(mut tcp_stream: TcpStream, sctp_client: SctpStream, cache: &mut TempFileCache){

        // used to RR over the streams
        let mut stream_number = 0u16;

        println!("New client");

        let mut buffer: Vec<u8> = vec![0;BUFFER_SIZE];

        loop{

            println!("Tcp listener waiting for messages...");

            //TODO main thread de read de la browser

            // the tcp stream waits for a request
            match tcp_stream.read(&mut buffer){

                Ok(0) => {
                    println!("Tcp client closed");
                    break;
                }

                Err(error) => {
                    panic!("Tcp Client error: {}", error);
                }

                // request received
                Ok(n) => {
                    let received_message = String::from_utf8_lossy(&buffer[..n]);

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
                    sctp_client.write(&mut buffer[..],n,stream_number,0).expect("Sctp Client write error");

                    // simple RR over the streams
                    stream_number = (stream_number + 1) % MAX_STREAM_NUMBER;

                    let mut sender_info = new_sctp_sndrinfo();

                    // TODO aici vine threadul de receptie care o sa faca in loop read cu select

                    //read the response
                    match sctp_client.read(&mut buffer,Some(&mut sender_info),None){

                        Err(error)=>{
                            panic!("Sctp read error: {}", error);
                        }

                        // response received
                        Ok(n) =>{

                            // write the response header into the cache
                            cache.write_append(&uri,&buffer[..n]).expect("Temporary file write error");

                            // write into tcp stream
                            if let Err(error) = tcp_stream.write(&buffer[..n]){
                                panic!("Tcp write error: {}", error);
                            }

                            println!("Received on stream {}",sender_info.sinfo_stream)
                        }
                    }

                    // TODO tot threadul de receptie
                    // now loop to receive the chunked response body
                    loop{
                        // the sctp-stream waits to get a response
                        match sctp_client.read(&mut buffer,Some(&mut sender_info),None){
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
                                cache.write_append(&uri,&buffer[..n]).expect("Temporary file write error");

                                // write to tcp stream
                               tcp_stream.write(&buffer[..n]).expect("Tcp stream write error");

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


                            // get a string request and send it to the server
                            let request = http_request_to_string(basic_http_get_request(&uri));

                            sctp_client.write_all(request.as_bytes(),stream_number,0).expect("Sctp Client prefetch write error");

                            // RR over the streams
                            stream_number = (stream_number + 1) % MAX_STREAM_NUMBER;
                            //read the response body
                            match sctp_client.read(&mut buffer,Some(&mut sender_info),None){

                                Err(error)=>{
                                    panic!("Sctp read error: {}", error);
                                }

                                // response received
                                Ok(n) =>{

                                    let response = string_to_http_response(String::from_utf8_lossy(&buffer[..n]).as_ref());
                                    let content_length = response.headers().get("Content-Length").unwrap().to_str().unwrap().parse::<u64>().unwrap();
                                    println!("Received SCTP response length {content_length} from {uri}");

                                    // write to temporary file
                                    cache.write_append(&uri,&buffer[..n]).expect("Temporary file write error");

                                }
                            }

                            // now loop to receive the chunked response body
                            loop{
                                // the sctp-stream waits to get a response
                                match sctp_client.read(&mut buffer,Some(&mut sender_info),None){
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
                                        cache.write_append(&uri,&buffer[..n]).expect("Temporary file write error");
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