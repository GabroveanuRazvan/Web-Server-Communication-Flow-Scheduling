use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use libc::{close, recvmsg, IPPROTO_SCTP, MSG_PEEK, SCTP_EVENTS};
use crate::libc_wrappers::{debug_sockaddr, new_sock_addr_in, safe_recv, safe_setsockopt, sock_addr_to_c, SctpSenderInfo, SockAddrIn};
use crate::sctp_api::{events_to_u8, safe_sctp_connectx, safe_sctp_recvmsg, safe_sctp_sendmsg, safe_sctp_socket, SctpEventSubscribe, SctpPeerBuilder};
use io::Result;

#[derive(Debug)]
pub struct SctpStream{
    sock_fd: i32,
    // this will be completed if the stream was created by an accept call or be the first peer address if the client connects
    address: SocketAddrV4,
    // this will be completed if the stream calls connect
    peer_addresses: Option<Vec<Ipv4Addr>>,
    // if the stream is created by accept this will be None
    active_events: Option<SctpEventSubscribe>,
    ttl: u32,
}

impl SctpStream{

    pub fn new(sock_fd: i32, address: SocketAddrV4) -> Self{

        Self{
            sock_fd,
            address,
            peer_addresses: None,
            active_events: None,
            ttl: 0,
        }
    }

    pub fn connect(&mut self) -> &Self{

        // crate a new socket
        match safe_sctp_socket(){
            Ok(sock_fd) => self.sock_fd = sock_fd,
            Err(error)=> panic!("Sctp stream socket error: {}",error),
        }

        // check if we have any addresses to connect to
        let peer_addresses = match self.peer_addresses{
            Some(ref addresses) => addresses,
            None => panic!("Sctp stream peer addresses is None while using connect"),
        };

        let mut socket_addresses: Vec<SockAddrIn> = Vec::new();

        // convert the ivp4 peer addresses to C sockaddr_in
        for address in peer_addresses{

            let mut current_socket_address: SockAddrIn = new_sock_addr_in(self.address.port(),address.clone());

            debug_sockaddr(&current_socket_address);

            socket_addresses.push(current_socket_address)

        }

        if let Err(error) = safe_sctp_connectx(self.sock_fd, &mut socket_addresses){
            panic!("Connect error: {}", error);
        }

        self
    }

    /// Method used to set write ttl
    pub fn set_ttl(&mut self, ttl: u32) -> &Self{
        self.ttl = ttl;
        self
    }

    /// Method used to get ttl
    pub fn ttl(&self) ->u32{
        self.ttl
    }

    /// Method used to get the local address of the stream that was returned by accept
    pub fn local_address(&self) -> SocketAddrV4{
        self.address.clone()
    }

    /// Method used to read data from the socket, stores the client address and info
    pub fn read(&mut self, buffer: &mut [u8],
                sender_info: Option<&mut SctpSenderInfo>,
                flags: Option<&mut i32>) ->Result<usize>{

        let mut returned_sock_addr_c = sock_addr_to_c(&self.local_address());

        let mut dummy_flags = 0;

        // if flags is None just pass the reference of dummyflags
        match safe_sctp_recvmsg(self.sock_fd, buffer, Some(&mut returned_sock_addr_c), sender_info, match flags{
            Some(flags) => flags,
            None => &mut dummy_flags,
        }){
            Ok(size) => Ok(size as usize),
            Err(error) => Err(error),
        }


    }

    /// Method used to write data to a peer using a designated stream
    pub fn write(&mut self, buffer: &mut [u8], num_bytes: usize, stream_number: u16, flags: u32) -> Result<usize>{

        let mut sock_addr_c = sock_addr_to_c(&self.local_address());

        match safe_sctp_sendmsg(self.sock_fd,buffer,num_bytes,&mut sock_addr_c,0,flags,stream_number,self.ttl,0){
            Ok(size) => Ok(size as usize),
            Err(error) => Err(error),
        }

    }

    /// Method used to peek into the socket buffer
    pub fn peek(&self, buffer: &mut[u8]) -> Result<usize>{

        let message_size = buffer.len();

        match safe_recv(self.sock_fd,buffer,message_size,MSG_PEEK){
            Ok(size) => Ok(size as usize),
            Err(error) => Err(error),
        }
    }

    /// Method used to activate the event options of the client
    pub fn options(&self) ->&Self{

        let events = match self.active_events{
            Some(events) => events,
            None => panic!("No sctp stream were specified"),
        };

        let events_ref = events_to_u8(&events);

        if let Err(error) = safe_setsockopt(self.sock_fd,IPPROTO_SCTP,SCTP_EVENTS,events_ref){
            panic!("SCTP setsockopt error: {error}");
        }

        self
    }

}

/// Used to gracefully close the socket descriptor when the client goes out of scope
impl Drop for SctpStream{
    fn drop(&mut self){

        unsafe{close(self.sock_fd);}
        println!("Sctp client closed");

    }

}

/// Builder pattern for sctp stream used when the stream acts as a client that will call connect


pub struct SctpStreamBuilder{
    sock_fd: i32,
    // this will be completed if the stream was created by an accept call or be the first peer address if the client connects
    address: SocketAddrV4,
    // this will be completed if the stream calls connect
    peer_addresses: Option<Vec<Ipv4Addr>>,
    // if the stream is created by accept this will be None
    active_events: Option<SctpEventSubscribe>,
    ttl: u32,
}

impl SctpStreamBuilder{

    /// Sets the ttl
    pub fn ttl(mut self, ttl: u32)->Self{
        self.ttl = ttl;
        self
    }

    /// Builds the client based on the given information
    pub fn build(self) -> SctpStream{

        SctpStream{
            sock_fd: self.sock_fd,
            address: self.address,
            peer_addresses: self.peer_addresses,
            active_events: self.active_events,
            ttl: self.ttl,
        }
    }
}

impl SctpPeerBuilder for SctpStreamBuilder {

    /// Creates a new builder with default values
    fn new() -> Self{

        Self{
            sock_fd: 0,
            address: SocketAddrV4::new(Ipv4Addr::new(0,0,0,0),0),
            peer_addresses: None,
            active_events: None,
            ttl: 0,
        }
    }

    /// Creates a new stream like sctp socket
    fn socket(mut self) -> Self{

        let result = safe_sctp_socket();

        match result{
            Ok(descriptor) => self.sock_fd = descriptor,
            Err(e) => panic!("Sctp socket error: {e}"),
        };

        self
    }

    /// Adds the main address that the client will use to read and write data
    fn address(mut self,ipv4: Ipv4Addr) -> Self{

        self.address.set_ip(ipv4);
        self
    }

    /// Adds a subset of addresses to be later connected to
    fn addresses(mut self, mut addresses: Vec<Ipv4Addr>) -> Self{

        self.peer_addresses = Some(addresses);
        self
    }

    /// Sets the port of where the clinet will connect to
    fn port(mut self,port: u16) -> Self{

        self.address.set_port(port);
        self
    }

    /// Sets the events that the client will be subscribed to
    fn events(mut self, events: SctpEventSubscribe) -> Self{

        self.active_events = Some(events);
        self
    }

}

