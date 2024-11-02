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
use utils::lru_cache::FileCache;

use http::request::Request;
use http::Uri;
use memmap2::Mmap;

use std::fs::{create_dir,remove_dir,OpenOptions,remove_file};

use chrono::Utc;
use libc::clone;

fn main() -> io::Result<()> {

    let mut manager = TempFileManager::new(&Path::new("/tmp").join(TempFileManager::unique_name()));

    let key1 = "/index.html".to_string();

    manager.add(key1.clone())?;

    let key2 = "/index2.html".to_string();

    manager.add(key2.clone())?;

    let file = manager.open("/cevarandom.html".to_string())?;

    println!("{file:?}");



    sleep(Duration::from_secs(20));

    Ok(())
}


