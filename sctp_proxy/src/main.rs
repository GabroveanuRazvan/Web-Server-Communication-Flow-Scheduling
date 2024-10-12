use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::{mem, thread};
use libc::{sa_family_t, AF_INET};
use utils::libc_wrappers::{debug_sockaddr, safe_inet_pton, SockAddrIn};
use utils::sctp_api::{safe_sctp_recvmsg, safe_sctp_sendmsg, safe_sctp_socket, SctpEventSubscribe, SctpPeer, SctpPeerBuilder};
use utils::sctp_client::SctpClientBuilder;

const PORT: u16 = 7878;
const IPV4: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

fn main() {

    let mut events = SctpEventSubscribe::new();
    events.sctp_data_io_event = 1;

    let mut client = SctpClientBuilder::new()
        .socket()
        .address(IPV4)
        .port(PORT)
        .events(events)
        .build();

    println!("Client built.");

    client.connect(0);

    let mut buffer: Vec<u8> = "mesaj mare sa se vada".to_string().into_bytes();
    let mut size = buffer.len() as isize;
    let mut addr = client.get_socket_address();

    let x = client.write(&mut buffer[..],size, &mut addr,0,0,0);
    println!("{x:?}");

}
