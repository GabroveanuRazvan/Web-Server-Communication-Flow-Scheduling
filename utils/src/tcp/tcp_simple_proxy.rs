use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use crate::pools::thread_pool::ThreadPool;
use std::io::{BufRead, BufReader, ErrorKind, Read, Result, Write};
use crate::constants::SERVER_RECEIVE_BUFFER_SIZE;

pub struct TcpSimpleProxy{

    browser_server_address: SocketAddrV4,
    peer_connection_address: SocketAddrV4,
    worker_count: usize,

}

impl TcpSimpleProxy{
    
    pub fn start(self) -> Result<()>{
        
        let thread_pool = ThreadPool::new(self.worker_count);
        let browser_listener = TcpListener::bind(self.browser_server_address)?;
        
        for stream in browser_listener.incoming(){
            println!("Incoming TCP connection");
            
            let stream = stream?;
            let peer_stream = TcpStream::connect(self.peer_connection_address)?;
            
            thread_pool.execute(move || {
                
                Self::handle_client(stream, peer_stream);
                
            })
        }
        
        Ok(())
        
    }
    
    fn handle_client(mut browser_stream: TcpStream, mut peer_stream: TcpStream){
        
        let mut buffer = [0u8; SERVER_RECEIVE_BUFFER_SIZE];
        
        'splice_loop: loop{
            
            match browser_stream.read(&mut buffer){
                
                Err(ref error) if error.kind() == ErrorKind::ConnectionReset => break 'splice_loop,
                Err(ref error) => {
                    eprintln!("Browser stream: {}", error);
                    break 'splice_loop;
                },
                
                Ok(0) => {
                    eprintln!("Browser connection closed");
                    break 'splice_loop
                },
                
                Ok(bytes_received) => {
                    
                    // Send the request
                    if let Err(error) = peer_stream.write(&buffer[..bytes_received]){
                        eprintln!("Peer stream error: {error}");
                        break 'splice_loop
                    }
                    
                    // Create a bufreader and wait for the response
                    let mut reader = BufReader::new(&mut peer_stream);
                    let mut http_response_header = String::new();
                    
                    // Read the first headers and append them to the header
                    for _ in 0..4{
                        
                        let mut line = String::new();
                        if let Err(error) = reader.read_line(&mut line){
                            eprintln!("Peer stream error: {error}");
                            break 'splice_loop
                        }
                        
                        http_response_header.push_str(&line);
                    }
                    
                    // Get the content header
                    let mut content_header = String::new();
                    if let Err(error) = reader.read_line(&mut content_header){
                        eprintln!("Peer stream error: {error}");
                        break 'splice_loop
                    }
                    
                    http_response_header.push_str(&content_header);
                    
                    // Get the end \r\n
                    let mut end_line = String::new();
                    if let Err(error) = reader.read_line(&mut end_line){
                        eprintln!("Peer stream error: {error}");
                        break 'splice_loop
                    }
                   
                    http_response_header.push_str(&end_line);
                    
                    // Parse the content header and send the whole http header
                    let file_size = content_header.split_whitespace().nth(1).unwrap();
                    let file_size: usize = file_size.parse().unwrap();

                    if let Err(error) = browser_stream.write_all(http_response_header.as_bytes()){
                        eprintln!("Browser stream: {}", error);
                        break 'splice_loop;
                    }
                    
                    // Get the remaining bytes in the reader and send them
                    let residue_bytes = reader.fill_buf().unwrap();
                    let mut current_file_size = if residue_bytes.len() > 0 {
                        
                        if let Err(error) = browser_stream.write_all(residue_bytes){
                            eprintln!("Browser stream: {}", error);
                            break 'splice_loop;
                        }
                        
                        residue_bytes.len()
                    }
                    else{
                        0
                    };
                    
                    drop(reader);
                    
                    if current_file_size == file_size{
                        continue;
                    }
                    
                    // If the file was not whole, read the chunks in a loop and break when it is done
                    'file_loop: loop{
                        
                        match peer_stream.read(&mut buffer){
                            
                            Err(ref error) if error.kind() == ErrorKind::ConnectionReset => break 'file_loop,
                            Err(ref error) => {
                                eprintln!("Peer stream: {}", error);
                                break 'file_loop;
                            },

                            Ok(0) => {
                                eprintln!("Peer connection closed");
                                break 'file_loop
                            },
                            
                            Ok(bytes_received) => {
                                
                                current_file_size += bytes_received;
                                
                                if let Err(error) = browser_stream.write_all(&buffer[..bytes_received]){
                                    eprintln!("Browser stream: {}", error);
                                    break 'splice_loop;
                                }
                                
                                if current_file_size == file_size{
                                    break 'file_loop;
                                }
                                
                            }
                        }
                        
                    }
                    
                    
                }
                
            }
            
        }
        
    }
    
}


pub struct TcpSimpleProxyBuilder{
    
    browser_server_address: SocketAddrV4,
    peer_connection_address: SocketAddrV4,
    worker_count: usize,
    
}

impl Default for TcpSimpleProxyBuilder {
    fn default() -> Self{
        
        Self{
            browser_server_address: SocketAddrV4::new(Ipv4Addr::UNSPECIFIED,0),
            peer_connection_address: SocketAddrV4::new(Ipv4Addr::UNSPECIFIED,0),
            worker_count: 0,
        }
        
    }
}

impl TcpSimpleProxyBuilder{
    
    pub fn new() -> Self{
        Self::default()
    }
    pub fn browser_port(mut self,port: u16) -> Self{
        self.browser_server_address.set_port(port);
        self
    }
    
    pub fn browser_ipv4(mut self,ip: Ipv4Addr) -> Self{
        self.browser_server_address.set_ip(ip);
        self
    }
    
    pub fn peer_connection_port(mut self, port: u16) -> Self{
        self.peer_connection_address.set_port(port);
        self
    }
    
    pub fn peer_connection_ipv4(mut self,ip: Ipv4Addr) -> Self{
        self.peer_connection_address.set_ip(ip);
        self
    }
    
    pub fn worker_count(mut self, worker_count: usize) -> Self{
        self.worker_count = worker_count;
        self
    }
    
    pub fn build(self) -> TcpSimpleProxy{
        
        TcpSimpleProxy{
            browser_server_address: self.browser_server_address,
            peer_connection_address: self.peer_connection_address,
            worker_count: self.worker_count,
        }
        
    }
    
    
}