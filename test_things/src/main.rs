use std::net::{Ipv4Addr, SocketAddrV4};
use std::thread;
use std::thread::Thread;
use std::time::Duration;
use libc::SCTP_SENDALL;
use utils::libc_wrappers::{sock_addr_to_c, c_to_sock_addr, debug_sockaddr};
use utils::sctp_api::{SctpEventSubscribe, SctpEventSubscribeBuilder, SctpPeer, SctpPeerBuilder};
use utils::sctp_client::SctpClientBuilder;
use utils::sctp_server::{SctpStream, SctpStreamBuilder};

const PORT: u16 = 7878;
const IPV4: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
fn main() {

    let mut events = SctpEventSubscribeBuilder::new().sctp_data_io_event().build();

    let mut sctp_client = SctpStreamBuilder::new()
        .socket()
        .address(IPV4)
        .port(PORT)
        .addresses(vec![IPV4])
        .ttl(0)
        .events(events)
        .build();

    sctp_client.options();
    sctp_client.connect();

    for i in 0..10{
        let mut buffer = format!("mesaj{}",i).to_string().into_bytes();
        sctp_client.write(&mut buffer[..],6,i,0);
    }

    for i in 0..20{
        let mut buffer: Vec<u8> = vec![0; 10];
        sctp_client.read(&mut buffer,None,None);

        println!("{}",String::from_utf8_lossy(&buffer));
    }

    thread::sleep(Duration::from_secs(50));

}
