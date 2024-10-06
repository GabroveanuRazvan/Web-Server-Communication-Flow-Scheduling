
mod modules;

use std::ffi::CString;
use std::mem;
use std::net::Ipv4Addr;
use crate::modules::sctp_server::SctpServerBuilder;

use libc::{in_addr_t, AF_INET};
use crate::modules::libc_wrappers::{debug_sockaddr, safe_inet_pton, SockAddrIn};

fn main() {

    let server = SctpServerBuilder::new()
        .descriptor()
        .address(Ipv4Addr::new(127, 0, 0, 1))
        .address(Ipv4Addr::new(127, 0, 0, 2))
        .address(Ipv4Addr::new(192,168,1,123))
        .port(7878)
        .build();

    println!("{:?}",server);

    server.bind();

    // let ip = Ipv4Addr::new(127, 0, 0, 1);
    // let mut c : u32 = 1;
    // let mut sock : SockAddrIn = unsafe{mem::zeroed()};
    // sock.sin_family = 2;
    // sock.sin_port = 32_u16.to_be();
    //
    // let ceva = safe_inet_pton(ip.to_string(),&mut sock.sin_addr.s_addr);
    // debug_sockaddr(&sock);



}
