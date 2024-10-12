use libc::{close, AF_INET, IPPROTO_SCTP, SCTP_EVENTS};
use crate::libc_wrappers::{debug_sockaddr, safe_inet_pton, safe_setsockopt, SctpSenderInfo, SockAddrIn};
use crate::sctp_api::{events_to_u8, safe_sctp_connectx, safe_sctp_recvmsg, safe_sctp_sendmsg, safe_sctp_socket, SctpEventSubscribe, SctpPeer, SctpPeerBuilder};
use crate::sctp_server::SctpServer;
use std::io::Result;
use std::mem;
use std::net::Ipv4Addr;
use std::ops::BitAnd;


#[derive(Debug)]
pub struct SctpClient{
    sock_fd: i32,
    addresses: Vec<Ipv4Addr>,
    port: u16,
    active_events: SctpEventSubscribe,
}

impl SctpClient{

    pub fn connect(&self, flags: i32) -> &Self{

        let mut socket_addresses: Vec<SockAddrIn> = Vec::new();

        for address in &self.addresses{

            let mut current_socket_address: SockAddrIn = unsafe{mem::zeroed()};

            current_socket_address.sin_family = AF_INET as u16;
            current_socket_address.sin_port = self.port.to_be();

            // strange bug: if inet_pton is called after the initialization of family and port s_addr will be 0 no matter the ip given
            if let Err(error) = safe_inet_pton(address.to_string(),&mut current_socket_address.sin_addr.s_addr) {
                panic!("Inet_pton error: {error}");
            }

            debug_sockaddr(&current_socket_address);

            socket_addresses.push(current_socket_address);

        }

        if let Err(error) = safe_sctp_connectx(self.sock_fd, &mut socket_addresses, flags){
            panic!("Connect error {}", error);
        }

        self
    }

    pub fn get_socket_address(&self) -> SockAddrIn{
        let mut socket_address: SockAddrIn = unsafe{mem::zeroed()};

        socket_address.sin_family = AF_INET as u16;
        socket_address.sin_port = self.port.to_be();

        // strange bug: if inet_pton is called after the initialization of family and port s_addr will be 0 no matter the ip given
        if let Err(error) = safe_inet_pton(self.addresses[0].to_string(),&mut socket_address.sin_addr.s_addr) {
            panic!("Inet_pton error: {error}");
        }

        debug_sockaddr(&socket_address);

        socket_address
    }

}

impl SctpPeer for SctpClient{
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

/// Used to gracefully close the socket descriptor when the client goes out of scope
impl Drop for SctpClient{
    fn drop(&mut self){

        unsafe{close(self.sock_fd);}
        println!("Sctp Client closed");

    }

}
pub struct SctpClientBuilder {
    sock_fd: i32,
    addresses: Vec<Ipv4Addr>,
    port: u16,
    active_events: SctpEventSubscribe,
}

impl SctpClientBuilder{

    pub fn build(self)-> SctpClient{

        SctpClient{
            sock_fd: self.sock_fd,
            addresses: self.addresses,
            port: self.port,
            active_events: self.active_events,
        }

    }

}

impl SctpPeerBuilder for SctpClientBuilder{

    /// Creates a new builder with default values
    fn new() -> Self{
        Self{
            sock_fd: 0,
            addresses: vec![],
            port: 8080,
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

    /// Sets the port
    fn port(mut self,port: u16) -> Self{

        self.port = port;
        self
    }

    /// Sets the events
    fn events(mut self, events: SctpEventSubscribe) -> Self{
        self.active_events = events;
        self
    }

}