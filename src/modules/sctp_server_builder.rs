use std::net::Ipv4Addr;
use super::sctp_api;
use libc::{
    socket,
    AF_INET,
    SOCK_STREAM,
    IPPROTO_SCTP,
};
use std::io::Result;

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

    pub fn descriptor(mut self) -> Result<Self>{



        self
    }

}