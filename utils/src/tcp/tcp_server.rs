use std::env;
use std::fs::OpenOptions;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::io::{ErrorKind, Read, Result, Write};
use memmap2::Mmap;
use crate::constants::REQUEST_BUFFER_SIZE;
use crate::http_parsers::{basic_http_response, extract_uri, http_response_to_string};
use crate::pools::thread_pool::ThreadPool;

pub struct TcpServer{
    address: SocketAddrV4,
    root: PathBuf,
    worker_count: usize,
    file_packet_size: usize,
}

impl TcpServer {
    
    pub fn start(self) -> Result<()>{
        
        let listener = TcpListener::bind(self.address)?;
        let thread_pool = ThreadPool::new(self.worker_count);
        env::set_current_dir(self.root)?;
        
        for stream in listener.incoming(){
            let stream = stream?;
            let file_chunk_size = self.file_packet_size;
            
            thread_pool.execute(move || {
                println!("Accepted connection from {}", stream.peer_addr().unwrap());
                Self::handle_client(stream,file_chunk_size).unwrap();
            })
        }
        
        Ok(())
    }
    
    fn handle_client(mut stream: TcpStream, file_chunk_size: usize) -> Result<()>{
        
        let mut buffer = [0u8;REQUEST_BUFFER_SIZE];
        
        'stream_loop: loop{
            
            match stream.read(&mut buffer){
                Err(ref error ) if error.kind() == ErrorKind::ConnectionReset =>{
                    eprintln!("Connection reset by peer");
                    break 'stream_loop
                },
                
                Err(error) => return Err(error),
                
                Ok(0) => {
                    println!("Connection closed.");
                    break 'stream_loop;
                }
                
                Ok(_bytes_received) => {
                    // TODO better parsing
                    // Extract the first line of the request
                    let new_line_position = buffer.iter().position(|&b| b == b'\n').unwrap();
                    let request_line = String::from_utf8_lossy(&buffer[..new_line_position]).to_string();

                    // Get the server-side file name, the cache side file name and path
                    let path_request = extract_uri(request_line).unwrap();

                    let file_path = match path_request.trim() {
                        "/" => "./index.html".to_string(),
                        _ => {
                            // Remove query operator ? in path
                            String::from(".") + &path_request.trim_end_matches("?")
                        }
                    };

                    let file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(false)
                        .truncate(false)
                        .open(&file_path);

                    let file = file.unwrap_or_else(|_|{
                        OpenOptions::new()
                            .read(true)
                            .write(true)
                            .create(false)
                            .truncate(false)
                            .open("./404.html").unwrap()
                    });
                    
                    let mmap = unsafe{Mmap::map(&file)}?;
                    let file_size = mmap.len();
                    
                    
                    // Send the response header of given size
                    let http_response = basic_http_response(file_size);
                    let string_response = http_response_to_string(http_response);

                    // Check for broken pipe error in case the browser abruptly shut down the connection
                    if let Err(error) = stream.write_all(string_response.as_bytes()){
                        if error.kind() == ErrorKind::BrokenPipe{
                            break 'stream_loop;
                        }
                    }
                    
                    // Send the file in chunks
                    for file_chunk in mmap.chunks(file_chunk_size){
                        if let Err(error) = stream.write_all(file_chunk){
                            if error.kind() == ErrorKind::BrokenPipe{
                                println!("Broken pipe");
                                break 'stream_loop;
                            }
                        }
                    }
                    
                }
            }
            
        }
        
        Ok(())
    }
    
}

pub struct TcpServerBuilder{
    address: SocketAddrV4,
    root: PathBuf,
    worker_count: usize,
    file_packet_size: usize,
}

impl Default for TcpServerBuilder {
    fn default() -> Self{
        Self{
            address: SocketAddrV4::new(Ipv4Addr::UNSPECIFIED,0),
            root: PathBuf::from("."),
            worker_count: 0,
            file_packet_size: 0,
        }
    }
}

impl TcpServerBuilder {
    pub fn new() -> Self{
        Self::default()
    }
    
    pub fn ipv4_address(mut self,address: Ipv4Addr) -> Self{
        self.address.set_ip(address);
        self
    }
    
    pub fn port(mut self, port: u16) -> Self{
        self.address.set_port(port);
        self
    }
    
    pub fn worker_count(mut self, worker_count: usize) -> Self{
        self.worker_count = worker_count;
        self
    }
    
    pub fn file_packet_size(mut self, file_packet_size: usize) -> Self {
        self.file_packet_size = file_packet_size;
        self
    }
    
    pub fn root<P: AsRef<Path>>(mut self, root: P) -> Self{
        self.root = PathBuf::from(root.as_ref());
        self
    }
    
    pub fn build(self) -> TcpServer{
        TcpServer{
            address: self.address,
            root: self.root,
            worker_count: self.worker_count,
            file_packet_size: self.file_packet_size,
        }
    }
}