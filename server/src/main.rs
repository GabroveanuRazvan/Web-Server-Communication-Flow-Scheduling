

use std::ffi::CString;
use std::{mem, thread};
use std::io::{BufReader, Read,Result};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::thread::Thread;
use std::time::Duration;
use utils::sctp_server::{SctpServer, SctpServerBuilder};

use libc::{in_addr_t, AF_INET};
use utils::libc_wrappers::{debug_sctp_sndrcvinfo, debug_sockaddr, safe_inet_pton, SctpSenderInfo, SockAddrIn};
use utils::sctp_api::{SctpEventSubscribe, events_to_u8, SctpPeerBuilder, SctpEventSubscribeBuilder};
use std::ascii::escape_default;
use std::path::Path;
//netstat -lnp | grep sctp

const MAX_CONNECTIONS: u16 = 100;
const PORT: u16 = 7878;
const IPV4: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
const PATH_STR: &str = "./web_files";

fn main() -> Result<()> {
    let events = SctpEventSubscribeBuilder::new().sctp_data_io_event().build();

    let mut server = SctpServerBuilder::new()
        .socket()
        .address(IPV4)
        .port(PORT)
        .max_connections(MAX_CONNECTIONS)
        .events(events)
        .path(Path::new(PATH_STR))
        .build();

    server.bind()
          .listen()
          .options();

    thread::sleep(Duration::from_secs(5));
    println!("Server started");

    for stream in server.incoming(){

        let mut stream = stream.unwrap();

        SctpServer::handle_client(stream)?
    }



    Ok(())
}
