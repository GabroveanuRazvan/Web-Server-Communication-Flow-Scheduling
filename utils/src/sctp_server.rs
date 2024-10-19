use std::io::{Error, Read, Result};
use std::mem;
use std::net::{Ipv4Addr, SocketAddrV4};
use libc::{AF_INET, close, IPPROTO_SCTP, SCTP_EVENTS, SCTP_ASSOCINFO, socklen_t};
use crate::sctp_client::SctpStream;
use super::sctp_api::{safe_sctp_socket, safe_sctp_bindx, SCTP_BINDX_ADD_ADDR, safe_sctp_recvmsg, sctp_opt_info, SctpEventSubscribe, events_to_u8, safe_sctp_sendmsg, SctpPeerBuilder, safe_sctp_connectx, events_to_u8_mut};
use super::libc_wrappers::{SockAddrIn, safe_inet_pton, debug_sockaddr, safe_listen, SctpSenderInfo, safe_setsockopt, safe_accept, new_sock_addr_in, sock_addr_to_c, c_to_sock_addr, debug_sctp_sndrcvinfo, safe_getsockopt, new_sctp_sndrinfo};
use super::http_parsers::{basic_http_response, parse_http_request, response_to_string};

const BUFFER_SIZE: usize = 4096;

#[derive(Debug)]
pub struct SctpServer {
    sock_fd: i32,
    addresses: Vec<Ipv4Addr>,
    port: u16,
    max_connections: u16,
    active_events: SctpEventSubscribe,
}

/// Abstract implementation of a sctp server
impl SctpServer{
    /// Binds the given ipv4 address using sctp_bindx() on given port
    pub fn bind(&self) -> &Self{

        let mut socket_addresses: Vec<SockAddrIn> = Vec::new();

        // convert all ipv4 addresses to C SockAddrIn
        for address in &self.addresses{

            let mut current_socket_address: SockAddrIn = new_sock_addr_in(self.port,address.clone());

            debug_sockaddr(&current_socket_address);

            socket_addresses.push(current_socket_address);

        }

        if let Err(error) = safe_sctp_bindx(self.sock_fd,&mut socket_addresses,SCTP_BINDX_ADD_ADDR){
            panic!("SCTP bindx error: {error}");
        }

        self

    }

    /// Puts the server on passive mode
    pub fn listen(&self) -> &Self{

        if let Err(error) = safe_listen(self.sock_fd,self.max_connections as i32){
            panic!("SCTP Listen error: {error}");
        };

        self

    }

    /// Method used to accept a new client, stores the address into client_address if specified
    pub fn accept(&self) -> Result<SctpStream>{


        let mut dummy_size = size_of::<SockAddrIn>();

        // a new SockAddrIn where the client data will be stored
        let mut returned_sock_addr_c = new_sock_addr_in(0,Ipv4Addr::UNSPECIFIED);

        let mut sock_fd = safe_accept(self.sock_fd,Some(&mut returned_sock_addr_c),Some(&mut dummy_size))?;

        // create a new stream and its data
        Ok(SctpStream::new(sock_fd,c_to_sock_addr(&returned_sock_addr_c)))

    }

    pub fn options(&self) ->&Self{

        let events_ref = events_to_u8(&self.active_events);

        if let Err(error) = safe_setsockopt(self.sock_fd,IPPROTO_SCTP,SCTP_EVENTS,events_ref){
            panic!("SCTP setsockopt error: {error}");
        }

        self
    }

    pub fn get_options(&self) -> SctpEventSubscribe{
        let mut events = SctpEventSubscribe::new();

        if let Err(error) = safe_getsockopt(self.sock_fd,IPPROTO_SCTP,SCTP_EVENTS,events_to_u8_mut(&mut events)){
            panic!("SCTP getsockopt error: {error}");
        }

        events
    }

    /// Method used to create an Iterator that yields new SctpStreams
    pub fn incoming(&self) -> Incoming{
        Incoming::new(self)
    }

    ///Method used to handle clients

    pub fn handle_client(mut stream: SctpStream) -> Result<()>{

        println!("New client");

        let mut buffer: Vec<u8> = vec![0; BUFFER_SIZE];
        let mut client_address : SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
        let mut sender_info: SctpSenderInfo = new_sctp_sndrinfo();

        loop{
            let bytes_read = stream.read(&mut buffer,Some(&mut sender_info),None)?;

            if bytes_read == 0{
                break;
            }

            println!("Read {bytes_read} bytes");
            println!("Client address: {}", stream.local_address());
            debug_sctp_sndrcvinfo(&sender_info);

            let request = parse_http_request(&String::from_utf8(buffer.clone()).unwrap());

            println!("{request:?}");

            let mut response_body = "<h1>Hello world</h1>".to_string().into_bytes();
            let mut response_body_size = response_body.len();

            let mut response_bytes = response_to_string(&basic_http_response(response_body_size)).into_bytes();
            let mut response_size = response_bytes.len();


            match stream.write(&mut response_bytes,response_size,sender_info.sinfo_stream,sender_info.sinfo_flags as u32){
                Ok(bytes) => println!("Wrote {bytes}"),
                Err(e) => println!("Write Error: {:?}",e)
            }

            match stream.write(&mut response_body,response_body_size,sender_info.sinfo_stream,sender_info.sinfo_flags as u32){
                Ok(bytes) => println!("Wrote {bytes}"),
                Err(e) => println!("Write Error: {:?}",e)
            }

            // end message
            match stream.write(&mut response_body,0,sender_info.sinfo_stream,sender_info.sinfo_flags as u32){
                Ok(bytes) => println!("Wrote {bytes}"),
                Err(e) => println!("Write Error: {:?}",e)
            }

        }


        Ok(())
    }

}

/// Used to gracefully close the socket descriptor when the server goes out of scope
impl Drop for SctpServer{
    fn drop(&mut self){

        unsafe{close(self.sock_fd);}
        println!("Sctp Server closed");

    }

}


/// Iterator struct for the incoming method of the SctpServer
pub struct Incoming<'a>{
    sctp_listener: &'a SctpServer,
}

/// Create a new wrapper over a SctpServer
impl <'a> Incoming<'a>{
    fn new(sctp_listener: &'a SctpServer) -> Self{
        Incoming{sctp_listener}
    }
}

/// Implementation the iterator trait, the next method will call accept and yield the iterator
impl<'a> Iterator for Incoming<'a>{
    type Item = Result<SctpStream>;

    fn next(&mut self) -> Option<Self::Item>{
        match self.sctp_listener.accept(){
            Ok(stream) => Some(Ok(stream)),
            Err(error) => Some(Err(error)),
        }
    }

}

/// Used to initialize the data of the sctp server
pub struct SctpServerBuilder{
    sock_fd: i32,
    addresses: Vec<Ipv4Addr>,
    port: u16,
    max_connections: u16,
    active_events: SctpEventSubscribe,
}

impl SctpServerBuilder{

    /// Sets the maximum connections that the server can handle
    pub fn max_connections(mut self,max_connections: u16) -> Self{
        self.max_connections = max_connections;
        self
    }

    /// Builds the server based on the given information
    pub fn build(self) -> SctpServer{

        SctpServer{
            sock_fd: self.sock_fd,
            addresses: self.addresses,
            port: self.port,
            max_connections: self.max_connections,
            active_events: self.active_events,
        }
    }
}

impl SctpPeerBuilder for SctpServerBuilder {

    /// Creates a new builder with default values
    fn new() -> Self{

        Self{
            sock_fd: 0,
            addresses: vec![],
            port: 8080,
            max_connections: 0,
            active_events: SctpEventSubscribe::new(),
        }
    }

    /// Creates a new sctp socket with delimited packets and stores its file descriptor
    fn socket(mut self) -> Self{

        let result = safe_sctp_socket();

        match result{
            Ok(descriptor) => self.sock_fd = descriptor,
            Err(e) => panic!("Sctp socket error: {e}"),
        };

        self
    }

    /// Adds a new address to be later bound
    fn address(mut self,address: Ipv4Addr) -> Self{

        self.addresses.push(address);
        self
    }

    /// Adds a subset of addresses to be later bound
    fn addresses(mut self, mut addresses: Vec<Ipv4Addr>) -> Self{

        self.addresses.append(&mut addresses);
        self
    }

    /// Sets the port that the server will run on
    fn port(mut self,port: u16) -> Self{

        self.port = port;
        self
    }

    /// Sets the events that the server will be subscribed to
    fn events(mut self, events: SctpEventSubscribe) -> Self{

        self.active_events = events;
        self
    }

}
