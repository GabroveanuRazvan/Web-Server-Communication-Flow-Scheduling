use inotify::{EventMask, Inotify, WatchMask};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpStream};
use std::path::Path;
use std::{fs, thread};
use std::fs::OpenOptions;
use std::os::unix;
use std::sync::Mutex;
use std::thread::park;
use std::time::Duration;
use utils::libc_wrappers::{debug_sctp_sndrcvinfo, new_sctp_sndrinfo};
use utils::sctp::sctp_api::{SctpEventSubscribeBuilder, SctpPeerBuilder};
use utils::sctp::sctp_client::{SctpStream, SctpStreamBuilder};
use utils::sctp::sctp_server::SctpServerBuilder;
use utils::tcp_proxy::TcpProxy;

fn main(){


    let i: i32 = 256;
    let bytes = i.to_be_bytes();

    println!("{:?}", bytes);


}

// fn main() {
//     let mut inotify = Inotify::init()
//         .expect("Error while initializing inotify instance");
//
//     // Watch for modify and close events.
//     inotify
//         .watches()
//         .add(
//             "/tmp/tmpfs",
//             WatchMask::MODIFY | WatchMask::CREATE | WatchMask::MOVED_TO,
//         )
//         .expect("Failed to add file watch");
//
//     // Read events that were added with `Watches::add` above.
//     let mut buffer = [0; 1024];
//
//     thread::spawn(move || {
//         let addr = "127.0.0.1:7878";
//         let mut stream = TcpStream::connect(addr).unwrap();
//         thread::sleep(Duration::from_secs(2));
//         stream.write_all("/images_4k/4k1.jpg\n".as_ref()).unwrap();
//         stream.write_all("/images_4k/4k2.jpg\n".as_ref()).unwrap();
//         stream.write_all("/images_4k/4k3.jpg\n".as_ref()).unwrap();
//         stream.write_all("/images_4k/4k4.jpg\n".as_ref()).unwrap();
//         stream.write_all("/images_4k/4k5.jpg\n".as_ref()).unwrap();
//     });
//
//     loop {
//         let events = inotify.read_events_blocking(&mut buffer)
//             .expect("Error while reading events");
//
//         for event in events {
//             // if event.mask.contains(EventMask::CREATE) {
//             //     println!("Crated file: {:?}", event.name);
//             // }
//             //
//             // if event.mask.contains(EventMask::MODIFY){
//             //     println!("Mod file: {:?}", event.name);
//             // }
//
//             if event.mask.contains(EventMask::MOVED_TO){
//                 println!("Move file: {:?}", event.name);
//             }
//         }
//     }
// }