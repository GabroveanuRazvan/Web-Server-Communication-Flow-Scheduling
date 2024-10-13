

use std::ffi::CString;
use std::{mem, thread};
use std::io::{BufReader, Read,Result};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::thread::Thread;
use std::time::Duration;
use utils::sctp_server::SctpServerBuilder;

use libc::{in_addr_t, AF_INET};
use utils::libc_wrappers::{debug_sctp_sndrcvinfo, debug_sockaddr, safe_inet_pton, SctpSenderInfo, SockAddrIn};
use utils::sctp_api::{SctpEventSubscribe, events_to_u8, SctpPeer,SctpPeerBuilder};
use std::ascii::escape_default;
//netstat -lnp | grep sctp

const MAX_CONNECTIONS: u16 = 100;
const PORT: u16 = 7878;
const IPV4: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);

fn main() -> Result<()> {
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

    let mut buffer: Vec<u8> = vec![0; 4096];

    thread::sleep(Duration::from_secs(5));
    println!("Server started");

    let mut client_address : SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);
    let mut sender_info: SctpSenderInfo = unsafe { mem::zeroed() };
    let mut flags = 0;

    for stream in server.incoming(){

        let mut stream = stream.unwrap();
        println!("New client");

        println!("Client address: {}", client_address);

        let bytes_read = stream.read(&mut buffer,Some(&mut sender_info),&mut flags)?;
        println!("Read {bytes_read} bytes");

        println!("Client address: {}", client_address);
        // if let Err(error) = server.accept(Some(&mut client_address)){
        //     panic!("Failed to accept client: {}", error);
        // }


        debug_sctp_sndrcvinfo(&sender_info);
        println!("{:?}",String::from_utf8(buffer.clone()).unwrap());

        match stream.write(&mut buffer,bytes_read,sender_info.sinfo_stream,sender_info.sinfo_flags,0){
            Ok(bytes) => println!("Wrote {bytes}"),
            Err(e) => println!("Write Error: {:?}",e)
        }
    }



    Ok(())
}
