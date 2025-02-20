use std::collections::{HashMap, HashSet};
use std::fs;
use memmap2::{Mmap, MmapMut};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{LazyLock, RwLock};
use utils::html_prefetch_service::HtmlPrefetchService;
use utils::http_parsers::extract_http_paths;

struct PrefetchService{

    map: HashMap<PathBuf,HashSet<PathBuf>>

}

impl PrefetchService{

    fn new() -> Self{
        Self{map: HashMap::new()}
    }
    fn get_files<T: AsRef<Path>>(&mut self, file_path: T){

        let entry_it = fs::read_dir(file_path).unwrap();

        for entry in entry_it {
            let path = entry.unwrap().path();

            if path.is_dir(){
                self.get_files(&path);
            }

            if let Some(extension) = path.extension()  {

                if extension == "html"{

                    let file = OpenOptions::new()
                        .read(true)
                        .write(false)
                        .create(false)
                        .open(&path).unwrap();

                    let file_parent = path.parent().unwrap();

                    let mmap = unsafe{Mmap::map(&file).unwrap()};
                    let file_content = std::str::from_utf8(&mmap).unwrap();

                    let paths = extract_http_paths(file_content).iter().map(|path| file_parent.join(path)).collect::<HashSet<PathBuf>>();

                    if self.map.contains_key(&path) {
                        panic!("Key should not exist");
                    }

                    if !paths.is_empty(){
                        self.map.insert(path,paths);
                    }


                }

            }

        }

    }
}



fn main() {

    let mut p = HtmlPrefetchService::new();

    p.build_prefetch_links("./web_files").unwrap();

    let map = p.get_links();

    println!("{:#?}", map);



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