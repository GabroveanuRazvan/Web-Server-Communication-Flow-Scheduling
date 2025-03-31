use std::collections::HashMap;
use std::fs::File;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::io::{Error, ErrorKind, Read, Result, Write};
use std::path::{PathBuf};
use std::sync::{RwLock, LazyLock};
use crate::constants::KILOBYTE;
use crate::http_parsers::{basic_http_response, encode_path, extract_uri, http_response_to_string};
use std::sync::mpsc::{channel,Sender,Receiver};
use std::{thread};
use std::thread::JoinHandle;
use inotify::{EventMask, Inotify, WatchMask};
use memmap2::Mmap;
use crate::config::sctp_proxy_config::SctpProxyConfig;
use crate::pools::thread_pool::ThreadPool;

const REQUEST_BUFFER_SIZE: usize = 4 * KILOBYTE;
const INOTIFY_BUFFER_SIZE: usize = 16 * KILOBYTE;
const BROWSER_CHUNK_SIZE: usize = 32 * KILOBYTE;

/// Structure used to store the sender for each thread to be notified about the complete download of the requested file
static DOWNLOADING_FILES: LazyLock<RwLock<HashMap<PathBuf,Sender<bool>>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Debug)]
pub struct TcpProxy{
    port: u16,
    tcp_address: Ipv4Addr,

    inotify_thread: Option<JoinHandle<Result<()>>>,
    proxy_writer_thread: Option<JoinHandle<Result<()>>>,
}


impl TcpProxy{

    /// Start the proxy by starting a browser server.
    /// For each client connect to the sctp proxy.
    pub fn start(mut self) ->Result<()> {

        let browser_server = TcpListener::bind(SocketAddrV4::new(self.tcp_address, self.port))?;
        let client_pool = ThreadPool::new(SctpProxyConfig::max_browser_connections() as usize);

        // println!("Listening on {}:{}", self.tcp_address,self.port);

        let (writer_tx,writer_rx) = channel();
        self.inotify_thread = Some(Self::inotify_thread());
        self.proxy_writer_thread = Some(Self::proxy_writer_thread(writer_rx));

        for mut stream in browser_server.incoming(){

            let stream = stream?;
            let writer_tx = writer_tx.clone();

            client_pool.execute(move || {
                Self::handle_client(stream, writer_tx).unwrap();
            })
        }

        Ok(())
    }

    /// Method used to handle the connection of each client.
    pub fn handle_client(mut stream: TcpStream,writer_tx: Sender<String>) -> Result<()>{

        let mut buffer = vec![0; REQUEST_BUFFER_SIZE];

        loop{

            match stream.read(&mut buffer){

                // The browser closes the connection, just end the function
                Err(ref error) if error.kind() == ErrorKind::ConnectionReset => break,
                Err(error) => return Err(error),

                Ok(0) => {
                    // println!("Browser connection closed.");
                    break;
                }

                Ok(_bytes_received) => {

                    // TODO better parsing
                    // Extract the first line of the request
                    let new_line_position = buffer.iter().position(|&b| b == b'\n').unwrap();
                    let request_line = String::from_utf8_lossy(&buffer[..new_line_position]).to_string();

                    // println!("Request: {}", request_line);

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
                    let cache_file_path = PathBuf::from(SctpProxyConfig::cache_path()).join(&cache_file_name);
                    let file_path_request = format!("{}\n",file_name);

                    // println!("Request: {}", file_path_request);

                    // If the requested file does not exist in the cache, send a request to the SCTP proxy, and wait for the file to be downloaded
                    if !cache_file_path.exists(){

                        let (download_tx,download_rx) = channel();

                        // Insert into the map the sender so that the thread can be notified
                        DOWNLOADING_FILES.write()
                                         .unwrap()
                                         .insert(PathBuf::from(cache_file_name),download_tx);

                        // Send the request to the sctp proxy
                        writer_tx.send(file_path_request).map_err(
                            |e| Error::new(ErrorKind::Other,format!("Error sending file request: {}",e))
                        )?;

                        // Check again for the existence of the file in case it was downloaded right before inserting the receiver into the map
                        if !cache_file_path.exists(){
                            // Wait to be notified
                            download_rx.recv().unwrap();
                        }


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


                    // Check for broken pipe error in case the browser abruptly shut down the connection
                    if let Err(error) = stream.write_all(string_response.as_bytes()){
                        if error.kind() == ErrorKind::BrokenPipe{
                            break;
                        }
                    }

                    for chunk in mmap.chunks(BROWSER_CHUNK_SIZE){

                        if let Err(error) = stream.write_all(&chunk){
                            if error.kind() == ErrorKind::BrokenPipe{
                                break;
                            }
                        }

                    }

                    if let Err(error) = stream.write_all(b"\r\n"){
                        if error.kind() == ErrorKind::BrokenPipe{
                            break;
                        }
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
                SctpProxyConfig::cache_path(),
                WatchMask::MOVED_TO,
            )
            .expect("Failed to add file watch");

        let mut events_buffer = vec![0u8; INOTIFY_BUFFER_SIZE];

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

    /// Receives sctp-proxy requests through the channel and writes them to the standard output for the proxy to process.
    pub fn proxy_writer_thread(writer_rx: Receiver<String>) -> JoinHandle<Result<()>>{

        let mut stdout = std::io::stdout();

        thread::spawn(move || {

            for request in writer_rx{
                stdout.write_all(request.as_bytes())?;
            }

            Ok(())
        })

    }

}

/// Builder pattern used for the TCP Proxy.
pub struct TcpProxyBuilder{
    port: u16,
    tcp_address: Ipv4Addr,
}

impl TcpProxyBuilder{
    pub fn new() -> Self{
        Self{
            port: 0,
            tcp_address: Ipv4Addr::UNSPECIFIED,
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

    pub fn build(self) -> TcpProxy{

        TcpProxy{
            port: self.port,
            tcp_address: self.tcp_address,

            inotify_thread: None,
            proxy_writer_thread: None,
        }
    }


}