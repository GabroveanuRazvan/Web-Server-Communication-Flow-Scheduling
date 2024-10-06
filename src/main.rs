
mod modules;

use std::ffi::CString;
use std::{mem, thread};
use std::net::Ipv4Addr;
use std::thread::Thread;
use std::time::Duration;
use crate::modules::sctp_server::SctpServerBuilder;

use libc::{in_addr_t, AF_INET};
use crate::modules::libc_wrappers::{debug_sockaddr, safe_inet_pton, SockAddrIn};

//netstat -lnp | grep sctp
fn main() {

    let server = SctpServerBuilder::new()
        .descriptor()
        .address(Ipv4Addr::new(0,0,0,0))
        .port(7878)
        .max_connections(20)
        .build();

    server.bind()
          .listen();



}
