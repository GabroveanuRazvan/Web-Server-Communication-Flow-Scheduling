use std::collections::{HashMap, HashSet};
use std::{fs, thread};
use std::cell::RefCell;
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
use utils::pools::indexed_thread_pool::IndexedTreadPool;

fn main(){


    let mut map: RwLock<HashMap<i32,RefCell<Option<File>>>> = RwLock::new(HashMap::new());

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("random.txt").unwrap();

    let mut map_lock = map.write().unwrap();
    map_lock.insert(1,RefCell::new(Some(file)));
    drop(map_lock);

    let map = map.read().unwrap();

    let mut file_ref = map.get(&1).unwrap().borrow_mut();
    let file = file_ref.as_mut().unwrap();

    file.write_all(b"Hello, world!").unwrap();

    file_ref.take();

    println!("{:?}",file_ref);

    let path = PathBuf::from("/tmp/tmpfs/ceva.txt.tmp");
    let mut path = path.with_extension("");
    println!("{:?}",path);
    path.set_extension(".tmp");
    println!("{:?}",path);


}
