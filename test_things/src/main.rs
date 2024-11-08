use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::cmp::Reverse;
use utils::shortest_job_first_pool::{Job, SjfPool};

fn main() {

    let file1 = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .truncate(false)
        .open("test1.txt").unwrap();

    let file2 = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .truncate(false)
        .open("test1.txt").unwrap();

    let mapped_file = MappedFile::new(file1).unwrap();

    let job_file = MappedFileJob::new(mapped_file,Box::new(||{println!("Yey")}));

    let mapped_file2 = MappedFile::new(file2).unwrap();

    let mut job_file2 = MappedFileJob::new(mapped_file2,Box::new(||{println!("Yey")}));

    println!("{:?}",job_file2.cmp(&job_file));

    job_file.execute();

}
use std::cmp::Ordering;
use std::fs::{File, OpenOptions};
use utils::mapped_file::{MappedFile, MappedFileJob};

#[derive(Debug, Clone, Copy)]
struct IntWrapper(i32);

// Implementăm PartialEq pentru a permite compararea egalității
impl PartialEq for IntWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// Implementăm Eq, care este necesar pentru a folosi Ord
impl Eq for IntWrapper {}

// Implementăm PartialOrd pentru a permite compararea ordonată parțial
impl PartialOrd for IntWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Implementăm Ord pentru a permite compararea ordonată complet
impl Ord for IntWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Job for IntWrapper {
    fn execute(self) {
        thread::sleep(std::time::Duration::from_secs(self.0 as u64));
        println!("{}", self.0);
    }
}