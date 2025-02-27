use std::collections::HashMap;
use std::fs::File;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Read, Result, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Condvar, RwLock, LazyLock};
use crate::constants::KILOBYTE;
use crate::http_parsers::{basic_http_response, encode_path, extract_uri, http_response_to_string};
use std::sync::mpsc::{channel, Receiver,Sender };
use std::{fs, thread};
use std::thread::JoinHandle;
use inotify::{EventMask, Inotify, WatchMask};
use memmap2::Mmap;
use crate::pools::thread_pool::ThreadPool;

const BUFFER_SIZE: usize = 4 * KILOBYTE;
const CHUNK_SIZE: usize = 4 * KILOBYTE;

const NUM_THREADS: usize = 6;

/// Structure used to store the sender for each thread to be notified about the complete download of the requested file
static DOWNLOADING_FILES: LazyLock<RwLock<HashMap<PathBuf,Sender<bool>>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

const CACHE_PATH: &str = "/tmp/tmpfs";

#[derive(Debug)]
pub struct TcpProxy{
    port: u16,
    tcp_address: Ipv4Addr,
    sctp_proxy_address: SocketAddrV4,
}


impl TcpProxy{

    /// Start the proxy by starting a browser server.
    /// For each client connect to the sctp proxy.
    pub fn start(self) ->Result<()> {

        let browser_server = TcpListener::bind(SocketAddrV4::new(self.tcp_address, self.port))?;
        let client_pool = ThreadPool::new(NUM_THREADS);

        println!("Listening on {}:{}", self.tcp_address,self.port);

        Self::inotify_thread();

        for mut stream in browser_server.incoming(){

            let stream = stream?;
            let proxy_stream = TcpStream::connect(self.sctp_proxy_address)?;

            client_pool.execute(move || {
                Self::handle_client(stream, proxy_stream).unwrap();
            })
        }

        Ok(())
    }

    /// Method used to handle the connection of each client.
    pub fn handle_client(mut stream: TcpStream,mut proxy_stream: TcpStream) -> Result<()>{

        let mut buffer = vec![0; BUFFER_SIZE];

        loop{

            match stream.read(&mut buffer){

                Err(error) => return Err(error),

                Ok(0) => {
                    println!("Browser connection closed.");
                    break;
                }

                Ok(_bytes_received) => {

                    // TODO better parsing
                    // Extract the first line of the request
                    let new_line_position = buffer.iter().position(|&b| b == b'\n').unwrap();
                    let request_line = String::from_utf8_lossy(&buffer[..new_line_position]).to_string();

                    println!("Request: {}", request_line);

                    // Get the server-side file name, the cache side file name and path
                    let file_name = extract_uri(request_line).unwrap();

                    let file_name = match file_name.trim() {
                        "/" => "/index.html".to_string(),
                        _ => {
                            // Remove query operator ? in path
                            file_name.trim_end_matches("?").to_string()
                        }
                    };

                    let cache_file_name = encode_path(&file_name);
                    let cache_file_path = PathBuf::from(CACHE_PATH).join(&cache_file_name);
                    let file_path_request = format!("{}\n",file_name);

                    println!("Request: {}", file_path_request);

                    // If the requested file does not exist in the cache, send a request to the SCTP proxy, and wait for the file to be downloaded
                    if !cache_file_path.exists(){

                        let (download_tx,download_rx) = channel();

                        // Insert into the map the sender so that the thread can be notified
                        DOWNLOADING_FILES.write()
                                         .unwrap()
                                         .insert(PathBuf::from(cache_file_name),download_tx);

                        // Send the request to the sctp proxy
                        proxy_stream.write_all(file_path_request.as_bytes())?;

                        // Wait to be notified
                        download_rx.recv().unwrap();

                        // Remove the map entry
                        DOWNLOADING_FILES.write()
                                         .unwrap()
                                         .remove(&cache_file_path);
                    }

                    // Send the file to the client in chunks
                    let file = File::open(cache_file_path)?;

                    let mmap = unsafe { Mmap::map(&file)? };

                    let file_size  = mmap.len();

                    let http_response = basic_http_response(file_size);
                    let string_response = http_response_to_string(http_response);

                    stream.write_all(string_response.as_bytes())?;

                    for chunk in mmap.chunks(BUFFER_SIZE){

                        stream.write_all(&chunk)?;

                    }

                }

            }

        }

        Ok(())
    }


    /// Thread that runs Inotify API: https://man7.org/linux/man-pages/man7/inotify.7.html.
    /// After getting an event the thread will retrieve the cache entry of the file to send the signal to the waiting client thread.
    pub fn inotify_thread() -> JoinHandle<Result<()>> {

        // Initialize inotify
        let mut inotify = Inotify::init()
            .expect("Error while initializing inotify instance");

        // Configure inotify to only look for moved files events
        inotify
            .watches()
            .add(
                CACHE_PATH,
                WatchMask::MOVED_TO,
            )
            .expect("Failed to add file watch");

        let mut events_buffer = vec![0u8; BUFFER_SIZE];

        // Spawn a thread that reads in a loop the events
        thread::spawn(move || {

            loop {
                let events = inotify.read_events_blocking(&mut events_buffer)
                    .expect("Error while reading events");

                for event in events {

                    // File downloaded
                    if event.mask.contains(EventMask::MOVED_TO){

                        let file_name = event.name
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string();

                        // Retrieve the transmitter and send a signal
                        let download_map = DOWNLOADING_FILES.read().unwrap();

                        if let Some(sender) = download_map.get(&PathBuf::from(&file_name)){
                            sender.send(true).expect(format!("Error while sending file: {}", file_name).as_str());
                        }


                    }
                }
            }


            Ok(())
        })

    }

}

/// Builder pattern used for the TCP Proxy.
pub struct TcpProxyBuilder{
    port: u16,
    tcp_address: Ipv4Addr,
    sctp_proxy_address: SocketAddrV4,
}

impl TcpProxyBuilder{
    pub fn new() -> Self{
        Self{
            port: 7878,
            tcp_address: Ipv4Addr::UNSPECIFIED,
            sctp_proxy_address: SocketAddrV4::new(Ipv4Addr::UNSPECIFIED,7979),
        }
    }

    pub fn port(mut self, port: u16) -> Self{
        self.port = port;
        self
    }

    pub fn tcp_address(mut self, tcp_address: Ipv4Addr) -> Self{
        self.tcp_address = tcp_address;
        self
    }

    pub fn sctp_proxy_address(mut self, sctp_proxy_address: SocketAddrV4) -> Self{
        self.sctp_proxy_address = sctp_proxy_address;
        self
    }

    pub fn sctp_proxy_ipv4(mut self, sctp_proxy_ipv4: Ipv4Addr) -> Self{
        self.sctp_proxy_address.set_ip(sctp_proxy_ipv4);
        self
    }

    pub fn sctp_proxy_port(mut self,sctp_proxy_port: u16) -> Self{
        self.sctp_proxy_address.set_port(sctp_proxy_port);
        self
    }

    pub fn build(self) -> TcpProxy{

        TcpProxy{
            port: self.port,
            tcp_address: self.tcp_address,
            sctp_proxy_address: self.sctp_proxy_address,
        }
    }


}