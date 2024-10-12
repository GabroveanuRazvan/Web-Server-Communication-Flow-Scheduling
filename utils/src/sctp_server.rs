use std::io::{Read,Result};
use std::mem;
use std::net::{Ipv4Addr};
use libc::{AF_INET,close,IPPROTO_SCTP,SCTP_EVENTS};
use super::sctp_api::{safe_sctp_socket, safe_sctp_bindx, SCTP_BINDX_ADD_ADDR, safe_sctp_recvmsg, SctpEventSubscribe, events_to_u8, safe_sctp_sendmsg, SctpPeer, SctpPeerBuilder};
use super::libc_wrappers::{SockAddrIn, safe_inet_pton, debug_sockaddr, safe_listen, SctpSenderInfo, safe_setsockopt, safe_accept, new_sock_addr_in};

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
    fn accept(&self,client_address: Option<&mut SockAddrIn>) -> Result<i32>{

        let mut dummy_size = size_of::<SockAddrIn>();

        let client_size = match client_address{
            None => None,
            Some(_) => Some(&mut dummy_size),
        };

         safe_accept(self.sock_fd,client_address,client_size)

    }


}


impl SctpPeer for SctpServer{
    /// Method used to read data from the socket, stores the client address and info
    fn read(&mut self, buffer: &mut [u8],
                client_address: Option<&mut SockAddrIn>,
                sender_info: Option<&mut SctpSenderInfo>,
                flags: &mut i32) ->Result<isize>{

        match safe_sctp_recvmsg(self.sock_fd, buffer, client_address, sender_info, flags){
            Ok(size) => Ok(size as isize),
            Err(error) => Err(error),
        }
    }

    /// Method used to write data to a peer using a designated stream
    fn write(&mut self, buffer: &mut [u8], num_bytes: isize, to_address: &mut SockAddrIn, stream_number: u16, flags: u16, ttl: u32) -> Result<usize>{

        match safe_sctp_sendmsg(self.sock_fd,buffer,num_bytes,to_address,0,flags as u32,stream_number,ttl,0){
            Ok(size) => Ok(size as usize),
            Err(error) => Err(error),
        }

    }

    /// Method used to activate the event options of the server
    fn options(&self) ->&Self{

        let events_ref = events_to_u8(&self.active_events);

        if let Err(error) = safe_setsockopt(self.sock_fd,IPPROTO_SCTP,SCTP_EVENTS,events_ref){
            panic!("SCTP setsockopt error: {error}");
        }

        self
    }


}

/// Used to gracefully close the socket descriptor when the server goes out of scope
impl Drop for SctpServer{
    fn drop(&mut self){

        unsafe{close(self.sock_fd);}
        println!("Sctp Server closed");

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