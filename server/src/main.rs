

use std::ffi::CString;
use std::{mem, thread};
use std::io::{BufReader, Read};
use std::net::Ipv4Addr;
use std::thread::Thread;
use std::time::Duration;
use utils::sctp_server::SctpServerBuilder;

use libc::{in_addr_t, AF_INET};
use utils::libc_wrappers::{debug_sctp_sndrcvinfo, debug_sockaddr, safe_inet_pton, SctpSenderInfo, SockAddrIn};
use utils::sctp_api::{SctpEventSubscribe,events_to_u8};

//netstat -lnp | grep sctp

const MAX_CONNECTIONS: u16 = 100;
const PORT: u16 = 7878;
const IPV4: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

fn main() {
    let mut events = SctpEventSubscribe::new();
    events.sctp_data_io_event = 1;

    let mut server = SctpServerBuilder::new()
        .socket()
        .address(IPV4)
        .port(PORT)
        .max_connections(MAX_CONNECTIONS)
        .events(events)
        .build();

    server.bind()
          .listen()
          .options();

    let mut buffer: Vec<u8> = vec![0; 1024];

    thread::sleep(Duration::from_secs(5));
    println!("Server started");

    let mut client_address : SockAddrIn = unsafe { mem::zeroed() };
    let mut sender_info: SctpSenderInfo = unsafe { mem::zeroed() };
    let mut flags = 0;

    loop{

        let bytes_read = server.read(&mut buffer,Some(&mut client_address),Some(&mut sender_info),&mut flags).unwrap();
        println!("Read {bytes_read} bytes");


        // if let Err(error) = server.accept(Some(&mut client_address)){
        //     panic!("Failed to accept client: {}", error);
        // }

        debug_sockaddr(&client_address);
        debug_sctp_sndrcvinfo(&sender_info);
        println!("{:?}",String::from_utf8(buffer.clone()).unwrap());

        // match server.write(&mut buffer,bytes_read,&mut client_address,sender_info.sinfo_stream,sender_info.sinfo_flags,0){
        //     Ok(bytes) => println!("Wrote {bytes}"),
        //     Err(e) => println!("Write Error: {:?}",e)
        // }
    }


}
