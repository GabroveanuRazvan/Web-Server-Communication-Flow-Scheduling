use std::collections::HashMap;
use memmap2::{Mmap, MmapMut};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::process::Command;
use std::sync::RwLock;

fn main() {

    let map : RwLock<HashMap<i32,RwLock<bool>>> = RwLock::new(HashMap::new());

    map.write().unwrap().insert(1,RwLock::new(true));
    let map_guard = map.read().unwrap();
    {
        let mut val = map_guard.get(&1).unwrap().write().unwrap();
        *val = false;
    }


    println!("{:#?}",map_guard.get(&1).unwrap().read().unwrap());

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