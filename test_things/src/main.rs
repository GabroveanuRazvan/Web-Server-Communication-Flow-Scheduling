use inotify::{EventMask, Inotify, WatchMask};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpStream};
use std::path::Path;
use std::thread;
use std::time::Duration;
use utils::libc_wrappers::{debug_sctp_sndrcvinfo, new_sctp_sndrinfo};
use utils::sctp::sctp_api::{SctpEventSubscribeBuilder, SctpPeerBuilder};
use utils::sctp::sctp_client::{SctpStream, SctpStreamBuilder};
use utils::sctp::sctp_server::SctpServerBuilder;

fn main() {

    let addr = "127.0.0.1:7878";

    let mut stream = TcpStream::connect(addr).unwrap();

    stream.write_all("/images_4k/4k4.jpg".as_ref()).unwrap();

}
