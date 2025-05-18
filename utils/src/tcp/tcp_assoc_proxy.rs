use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::sync::{mpsc, Arc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use crate::config::tcp_proxy_config::TcpProxyConfig;
use crate::constants::REQUEST_BUFFER_SIZE;
use crate::http_parsers::{basic_http_response, extract_uri, http_response_to_string};
use crate::packets::byte_packet::BytePacket;
use crate::pools::thread_pool::ThreadPool;
use crate::tcp::tcp_association::TcpAssociation;

pub struct TcpAssocRelay {
    peer_addr: SocketAddrV4,
    browser_addr: SocketAddrV4,
    worker_count: usize,
    stream_count: u8,
    
    sender_assoc_thread: Option<JoinHandle<std::io::Result<()>>>,
    receiver_assoc_thread: Option<JoinHandle<std::io::Result<()>>>,

}

impl TcpAssocRelay {

    /// Method that starts the relay
    pub fn start(mut self) -> std::io::Result<()> {

        println!("Starting assoc tcp relay...");
        
        let assoc = TcpAssociation::connect(self.peer_addr,self.stream_count)?;
        
        // Channel used to communicate between multiple tcp receiver threads and the transmitter assoc thread
        let (assoc_tx, assoc_rx) = mpsc::channel();
        let channel_map = Arc::new(RwLock::new(HashMap::new()));

        let sender_assoc = assoc.try_clone()?;
        let receiver_assoc_stream = assoc.try_clone()?;

        // Run the assoc client threads
        self.receiver_assoc_thread = Some(Self::receiver_assoc_thread(receiver_assoc_stream, Arc::clone(&channel_map)));
        self.sender_assoc_thread = Some(Self::sender_assoc_thread(sender_assoc, assoc_rx));
        Self::get_browser_requests(assoc_tx, channel_map, self.worker_count)?;

        Ok(())
    }

    /// Opens a threadpool of fixed size and handles the browser clients.
    fn get_browser_requests(assoc_tx: Sender<(String, u32)>, channel_map: Arc<RwLock<HashMap<u32,Sender<BytePacket>>>>, thread_count: usize) -> std::io::Result<()> {

        println!("Starting browser requests...");

        let browser_server = TcpListener::bind(SocketAddrV4::new(
            *TcpProxyConfig::browser_server_address(),
            TcpProxyConfig::browser_server_port())
        )?;

        let client_pool = ThreadPool::new(thread_count);

        let mut ppid = 1;

        for mut stream in browser_server.incoming(){

            let stream = stream?;
            let assoc_tx = assoc_tx.clone();
            let channel_map = Arc::clone(&channel_map);

            client_pool.execute(move || {
                Self::handle_client(stream, assoc_tx,channel_map,ppid).unwrap();
            });

            ppid = ppid.wrapping_add(1);
        }

        Ok(())
    }

    /// Reads a request and sends it to the assoc client to fetch. Wait to receive each packet and forwards it to the tcp client.
    fn handle_client(mut stream: TcpStream, assoc_tx: Sender<(String, u32)>, channel_map: Arc<RwLock<HashMap<u32,Sender<BytePacket>>>>, ppid: u32) -> std::io::Result<()> {

        let mut buffer = [0u8; REQUEST_BUFFER_SIZE];

        let (tcp_tx,tcp_rx) = mpsc::channel();

        // Assign a channel of byte packets for this client session
        {
            let mut channel_map = channel_map.write().unwrap();
            channel_map.insert(ppid, tcp_tx);
        }

        'stream_loop :loop{
            
            match stream.read(&mut buffer){

                // Connection reset, just break the loop
                Err(ref error) if error.kind() == ErrorKind::ConnectionReset => break 'stream_loop,
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
                    let file_name = extract_uri(request_line).unwrap();

                    let file_name = match file_name.trim() {
                        "/" => "/index.html".to_string(),
                        _ => {
                            // Remove query operator ? in path
                            file_name.trim_end_matches("?").to_string()
                        }
                    };

                    // Send the request
                    assoc_tx.send((file_name, ppid)).unwrap();

                    // File metadata
                    let mut current_file_size = 0;
                    let mut file_size = 0;

                    // Receive the incoming packets from the assoc client
                    'packet_loop: loop{

                        let mut byte_packet = match tcp_rx.recv(){
                            Ok(packet) => packet,
                            Err(_) => break 'packet_loop,
                        };


                        // Expecting a metadata packet when there is no current file size
                        if file_size == 0{
                            file_size = Self::parse_metadata_packet(&mut byte_packet);
                            current_file_size = 0;

                            // Send the response header of given size
                            let http_response = basic_http_response(file_size as usize);
                            let string_response = http_response_to_string(http_response);

                            // Check for broken pipe error in case the browser abruptly shut down the connection
                            if let Err(error) = stream.write_all(string_response.as_bytes()){
                                if error.kind() == ErrorKind::BrokenPipe{
                                    break 'stream_loop;
                                }
                            }
                        }
                        else{

                            // Send the chunk
                            let file_chunk = Self::parse_chunk_packet(&mut byte_packet);

                            if let Err(error) = stream.write_all(file_chunk){
                                if error.kind() == ErrorKind::BrokenPipe{
                                    println!("Broken pipe");
                                    break 'stream_loop;
                                }
                            }

                            current_file_size += file_chunk.len() as u64;

                            // Break the packet loop
                            if file_size == current_file_size{
                                file_size = 0;
                                break 'packet_loop;
                            }

                        }


                    }



                }

            }

        }
        Ok(())
    }

    /// Parses an already identified metadata packet.
    fn parse_metadata_packet(byte_packet: &mut BytePacket) -> u64 {
        byte_packet.read_u64().unwrap()
    }

    /// Parses an already identified chunk packet.
    fn parse_chunk_packet(byte_packet: &mut BytePacket) -> &[u8]{
        // Extract the packet data
        byte_packet.read_all().unwrap()
    }


    /// Assoc thread that sends incoming requests to the server to be processed.
    /// Each tcp connection is mapped to a unique ppid value.
    fn sender_assoc_thread(mut assoc: TcpAssociation, assoc_rx: Receiver<(String, u32)>) -> JoinHandle<std::io::Result<()>>{

        println!("Assoc sender thread started!");
        let stream_count =  assoc.stream_count();

        thread::Builder::new()
            .name(String::from("Assoc sender thread"))
            .spawn(move || {

                let mut stream_number = 0u8;

                for (path,ppid) in assoc_rx {

                    // Sanitize the path
                    let path = match path.trim() {
                        "/" => "/index.html".to_string(),
                        _ => {
                            path.to_string()
                        }
                    };


                    // Send the request to the server
                    assoc.send(path.as_bytes(),stream_number,ppid)?;

                    // Round-robin over the streams
                    stream_number = (stream_number + 1) % stream_count;


                }

                Ok(())
            }).unwrap()

    }

    /// Receive the server packets sequentially. Send them to their owner based on the received ppid.
    fn receiver_assoc_thread(mut assoc: TcpAssociation, channel_map: Arc<RwLock<HashMap<u32,Sender<BytePacket>>>>) -> JoinHandle<std::io::Result<()>>{

        println!("Assoc receiver thread started!");

        thread::Builder::new()
            .name(String::from("Assoc receiver thread"))
            .spawn(move || {
                
                loop{
                    
                    let message_info = match assoc.receive(){
                        Ok(message_info) => message_info,
                        Err(error ) if error.kind() == ErrorKind::UnexpectedEof => {
                            println!("Assoc connection closed!");
                            break;
                        }
                        
                        Err(error) => return Err(error),
                    };
                    
                    let byte_packet = BytePacket::from(message_info.message.as_slice());

                    // Get the channel of the tcp connection that waits for this packet
                    let channel_map = channel_map.read().unwrap();
                    let client_tx = channel_map.get(&message_info.ppid).expect("Ppid not in map");

                    match client_tx.send(byte_packet){
                        _ => ()
                    }
                    

                }

                Ok(())
            }).unwrap()

    }

}

/// Builder pattern for TcpAssocRelay

pub struct TcpAssocRelayBuilder {

    peer_addr: SocketAddrV4,
    browser_addr: SocketAddrV4,
    worker_count: usize,
    stream_count: u8,
}

impl Default for TcpAssocRelayBuilder {
    fn default() -> Self{
        Self{
            peer_addr: SocketAddrV4::new(Ipv4Addr::UNSPECIFIED,0),
            browser_addr: SocketAddrV4::new(Ipv4Addr::UNSPECIFIED,0),
            worker_count: 0,
            stream_count: 0,
        }
    }
}

impl TcpAssocRelayBuilder {
    
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn browser_port(mut self, port: u16) -> Self {
        self.browser_addr.set_port(port);
        self
    }

    pub fn peer_port(mut self, port: u16) -> Self {
        self.peer_addr.set_port(port);
        self
    }

    pub fn browser_ipv4(mut self, ipv4addr: Ipv4Addr) -> Self{
        self.browser_addr.set_ip(ipv4addr);
        self
    }

    pub fn peer_ipv4(mut self, ipv4addr: Ipv4Addr) -> Self{
        self.peer_addr.set_ip(ipv4addr);
        self
    }
    
    pub fn worker_count(mut self, worker_count: usize) -> Self{
        self.worker_count = worker_count;
        self
    }
    
    pub fn stream_count(mut self, stream_count: u8) -> Self{
        self.stream_count = stream_count;
        self
    }
    
    pub fn build(self) -> TcpAssocRelay {
        TcpAssocRelay {
            peer_addr: self.peer_addr,
            browser_addr: self.browser_addr,
            worker_count: self.worker_count,
            stream_count: self.stream_count,

            sender_assoc_thread: None,
            receiver_assoc_thread: None,
        }
    }
}