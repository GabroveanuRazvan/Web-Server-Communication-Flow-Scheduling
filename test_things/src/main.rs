use std::collections::{HashMap, HashSet};
use std::{fs, thread};
use memmap2::{Mmap, MmapMut};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{LazyLock, RwLock};
use path_clean::PathClean;
use utils::html_prefetch_service::HtmlPrefetchService;
use utils::http_parsers::extract_http_paths;
use std::num::Wrapping;
use std::time::Duration;

fn main() {

    // let th1 = thread::spawn(|| {
    //     let file = OpenOptions::new()
    //         .read(true)
    //         .open("./test2.txt").unwrap();
    //
    //     println!("{:#?}", file.metadata().unwrap().len());
    //
    //     let mmap = unsafe{Mmap::map(&file).unwrap()};
    //     println!("{:#?}", mmap.len());
    // });

    let th1 = thread::spawn(move || {

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("./test2.txt").unwrap();

        println!("{:#?}", file.metadata().unwrap().len());

        let mut mmap = unsafe{MmapMut::map_mut(&file).unwrap()};
        println!("{:#?}", mmap.len());

        thread::sleep(Duration::from_secs(5));

    });

    let th2  = thread::spawn(|| {

        thread::sleep(Duration::from_secs(1));
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("./test2.txt").unwrap();

        println!("{:#?}", file.metadata().unwrap().len());

        let mut mmap = unsafe{MmapMut::map_mut(&file).unwrap()};
        println!("{:#?}", mmap.len());

        fs::rename("./test2.txt","./test2.txt.dat").unwrap();

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("./test2.txt.dat").unwrap();

        println!("{:#?}", file.metadata().unwrap().len());

    });

    // th1.join().unwrap();
    th2.join().unwrap();




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