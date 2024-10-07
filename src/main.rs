
mod modules;

use std::ffi::CString;
use std::{mem, thread};
use std::io::{BufReader, Read};
use std::net::Ipv4Addr;
use std::thread::Thread;
use std::time::Duration;
use crate::modules::sctp_server::SctpServerBuilder;

use libc::{in_addr_t, AF_INET};
use crate::modules::libc_wrappers::{debug_sctp_sndrcvinfo, debug_sockaddr, safe_inet_pton, SctpSenderInfo, SockAddrIn};

//netstat -lnp | grep sctp
fn main() {

    let mut server = SctpServerBuilder::new()
        .descriptor()
        .address(Ipv4Addr::new(0,0,0,0))
        .port(7878)
        .max_connections(20)
        .build();

    server.bind()
          .listen();

    let mut buffer: Vec<u8> = vec![0; 50];

    thread::sleep(Duration::from_secs(15));
    println!("server started");

    let mut client_address : SockAddrIn = unsafe { mem::zeroed() };
    let mut sender_info: SctpSenderInfo = unsafe { mem::zeroed() };
    let mut flags = 0;

    loop{
        server.read(&mut buffer,Some(&mut client_address),Some(&mut sender_info),flags).unwrap();

        debug_sockaddr(&client_address);
        debug_sctp_sndrcvinfo(&sender_info);
        println!("{:?}",String::from_utf8(buffer.clone()));
    }




}
