use std::io::{Read,Result};
use std::mem;
use std::net::{Ipv4Addr};
use libc::{AF_INET,close};
use super::sctp_api::{safe_sctp_socket, safe_sctp_bindx, SCTP_BINDX_ADD_ADDR, safe_sctp_recvmsg};
use super::libc_wrappers::{SockAddrIn, safe_inet_pton, debug_sockaddr, safe_listen, SctpSenderInfo};

#[derive(Debug)]
pub struct SctpServer {
    sock_fd: i32,
    addresses: Vec<Ipv4Addr>,
    port: u16,
    max_connections: u16,
}

/// Abstract implementation of a sctp server
impl SctpServer{
    /// Binds the given ipv4 address using sctp_bindx() on given port
    pub fn bind(&self) -> &Self{

        let mut socket_addresses: Vec<SockAddrIn> = vec![];

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

    /// Method used to read data from the socket, stores the client address and info
    pub fn read(&mut self,buf: &mut [u8],
                client_address: Option<&mut SockAddrIn>,
                sender_info: Option<&mut SctpSenderInfo>,
                flags: i32) ->Result<usize>{

        let mut flags = 0;

        match safe_sctp_recvmsg(self.sock_fd,buf,client_address,sender_info,&mut flags){
            Ok(size) => Ok(size as usize),
            Err(error) => Err(error),
        }
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
}

impl SctpServerBuilder{

    /// Creates a new builder with default values
    pub fn new() -> Self{

        Self{
            sock_fd: 0,
            addresses: vec![],
            port: 8080,
            max_connections: 0,
        }
    }

    /// Creates a new sctp socket with delimited packets and stores its file descriptor
    pub fn descriptor(mut self) -> Self{

        let result = safe_sctp_socket();

        match result{
            Ok(descriptor) => self.sock_fd = descriptor,
            Err(e) => panic!("Sctp socket error: {e}"),
        };

        self
    }

    /// Adds a new address to be later bound
    pub fn address(mut self,address: Ipv4Addr) -> Self{

        self.addresses.push(address);
        self
    }

    /// Adds a subset of addresses to be later bound
    pub fn addresses(mut self, mut addresses: Vec<Ipv4Addr>) -> Self{

        self.addresses.append(&mut addresses);
        self
    }
    /// Sets the port that the server will run on
    pub fn port(mut self,port: u16) -> Self{

        self.port = port;
        self
    }

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
        }
    }

}