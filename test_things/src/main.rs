use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Component, Components, Path, PathBuf};
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use indexmap::IndexMap;
use utils::thread_pool;
use utils::thread_pool::ThreadPool;
use utils::lru_cache::FileCache;

use http::request::Request;
use http::Uri;
use memmap2::Mmap;

fn main() {

    let mut cache = FileCache::new(3);

    let path = "./Cargo.toml".to_string();

    let file = File::open(Path::new(&path)).unwrap();

    let mmap = unsafe{Mmap::map(&file).unwrap()};

    cache.insert(path.clone(),mmap);
    cache.insert("./web_files/ceva.html".to_string(),
                 unsafe{Mmap::map(&File::open(Path::new(&"./web_files/ceva.html".to_string())).unwrap()).unwrap()}
    );
    cache.insert("./web_files/hello.html".to_string(),
                 unsafe{Mmap::map(&File::open(Path::new(&"./web_files/hello.html".to_string())).unwrap()).unwrap()}
    );

    cache.insert("./nigga".to_string(),unsafe{Mmap::map(&File::open(Path::new(&"./web_files/hello.html".to_string())).unwrap()).unwrap()});

    let a = cache.get(&"./web_files/ceva.html".to_string());
    println!("{:?}",cache);


}



