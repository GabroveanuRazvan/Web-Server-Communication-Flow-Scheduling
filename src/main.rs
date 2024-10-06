
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
        .max_connections(20)
        .build()
        .bind()
        .listen();

}
