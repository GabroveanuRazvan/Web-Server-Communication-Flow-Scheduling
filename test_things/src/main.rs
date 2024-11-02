use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::{remove_dir_all, File};
use std::io::{Read, Write};
use std::path::{Component, Components, Path, PathBuf};
use std::rc::Rc;
use std::{fs, io, thread};
use std::thread::{sleep, Thread};
use std::time::Duration;
use indexmap::IndexMap;
use utils::thread_pool;
use utils::thread_pool::ThreadPool;
use utils::lru_cache::{TempFileCache, MappedFile};

use http::request::Request;
use http::Uri;
use memmap2::{Mmap, MmapMut};

use std::fs::{create_dir,remove_dir,OpenOptions,remove_file};

use chrono::Utc;
use libc::clone;
use utils::temp_file_manager::TempFileManager;

fn main() -> io::Result<()> {

    let mut cache = TempFileCache::new(3);

    cache.insert("/ana are mere".to_string());
    cache.insert("/ana are mere".to_string());
    cache.insert("/ana be here2".to_string());
    cache.insert("/ana be here3".to_string());

    cache.insert("/1".to_string());
    cache.insert("/2".to_string());
    cache.insert("/3".to_string());

    let map = cache.get(&"/1".to_string()).unwrap();

    let mut map = map.borrow_mut();
    map.write_append(b"mereajhishuiadsijowefjioewfjioewfjiooewjiojiwe");

    println!("{:#?}", map);
    sleep(Duration::from_secs(20));

    Ok(())
}


