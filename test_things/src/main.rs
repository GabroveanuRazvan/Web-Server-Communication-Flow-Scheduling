use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use utils::libc_wrappers::safe_dup;
use utils::sctp::sctp_api::SctpPeerBuilder;
use utils::sctp::sctp_client::{SctpStream, SctpStreamBuilder};

fn main(){

    let mut sctp_client = SctpStreamBuilder::new()
        .socket()
        .build();

    println!("{:?}",sctp_client);
    let mut sctp_clone = sctp_client.try_clone();
    println!("{:?}",sctp_clone)

}