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
use utils::pools::indexed_thread_pool::IndexedTreadPool;

fn main() {

    let pool = IndexedTreadPool::new(3);

    for i in 0..3{
        pool.execute(i,move || {
            println!("Job received {}",i);
        });
    }

}
