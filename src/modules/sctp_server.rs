use std::net::Ipv4Addr;
use crate::modules::sctp_api::safe_sctp_socket;

#[derive(Debug)]
pub struct SctpServer {
    sock_fd: i32,
    address: Ipv4Addr,
    port: u16,
}


impl SctpServer{

}

pub struct SctpServerBuilder{
    sock_fd: i32,
    address: Ipv4Addr,
    port: u16,
}

impl SctpServerBuilder{

    pub fn new() -> Self{
        Self{
            sock_fd: 0,
            address: Ipv4Addr::new(127,0,0,1),
            port: 8080,
        }
    }

    pub fn descriptor(mut self) -> Self{


        let result = safe_sctp_socket();

        match result{
            Ok(descriptor) => self.sock_fd = descriptor,
            Err(e) => panic!("Sctp socket error: {e}"),
        };

        self
    }

    pub fn address(mut self,address: Ipv4Addr) -> Self{
        self.address = address;
        self
    }

    pub fn port(mut self,port: u16) -> Self{
        self.port = port;
        self
    }

    pub fn build(self) -> SctpServer{
        SctpServer{
            sock_fd: self.sock_fd,
            address: self.address,
            port: self.port,
        }
    }

}