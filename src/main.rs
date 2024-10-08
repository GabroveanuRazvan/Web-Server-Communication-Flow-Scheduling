
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
use crate::modules::sctp_api::{SctpEventSubscribe,events_to_u8};

//netstat -lnp | grep sctp
fn main() {

    let mut events = SctpEventSubscribe::new();
    events.sctp_data_io_event = 1;

    let mut server = SctpServerBuilder::new()
        .descriptor()
        .address(Ipv4Addr::new(0,0,0,0))
        .port(7878)
        .max_connections(20)
        .events(events)
        .build();

    server.bind()
          .listen()
          .options();

    let mut buffer: Vec<u8> = vec![0; 50];

    thread::sleep(Duration::from_secs(5));
    println!("Server started");

    let mut client_address : SockAddrIn = unsafe { mem::zeroed() };
    let mut sender_info: SctpSenderInfo = unsafe { mem::zeroed() };
    let mut flags = 0;

    loop{
        server.read(&mut buffer,Some(&mut client_address),Some(&mut sender_info),flags).unwrap();

        debug_sockaddr(&client_address);
        debug_sctp_sndrcvinfo(&sender_info);
        println!("{:?}",String::from_utf8(buffer.clone()));

        match server.write(&mut buffer,&mut client_address,sender_info.sinfo_stream+1,0,10){
            Ok(_) => (),
            Err(e) => println!("Write Error: {:?}",e)
        }
    }


}
