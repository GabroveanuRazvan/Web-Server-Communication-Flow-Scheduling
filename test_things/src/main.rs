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
use utils::lru_cache::{FileCache, MappedFile};

use http::request::Request;
use http::Uri;
use memmap2::{Mmap, MmapMut};

use std::fs::{create_dir,remove_dir,OpenOptions,remove_file};

use chrono::Utc;
use libc::clone;
use utils::temp_file_manager::TempFileManager;

fn main() -> io::Result<()> {

    // let mut manager = TempFileManager::new(Path::new("/tmp/cache"));
    //
    // let file = manager.open("/ceva".to_string()).unwrap();
    // file.set_len(1024).unwrap();
    // let mut mmap = MappedFile::new(file).unwrap();
    //
    // mmap.write(b"ana are mrere");

    let file = OpenOptions::new().write(true).read(true).create(true).open("./test.txt").unwrap();

    let mut mmap = MappedFile::new(file).unwrap();

    mmap.append(b"ceva")?;
    mmap.append(b"\nana are mere")?;

    mmap.flush()?;
    // sleep(Duration::from_secs(20));
    Ok(())
}


