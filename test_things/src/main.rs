use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::cmp::Reverse;
use utils::shortest_job_first_pool::{Job, SjfPool};

fn main() {

    let mut sjf = SjfPool::new(4);


    sjf.schedule_job(IntWrapper(6));

    sjf.schedule_job(IntWrapper(5));

    sjf.schedule_job(IntWrapper(4));

    sjf.schedule_job(IntWrapper(3));

    sjf.schedule_job(IntWrapper(2));

    sjf.schedule_job(IntWrapper(1));

    thread::sleep(std::time::Duration::from_secs(10));

}
use std::cmp::Ordering;

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
    fn execute(&self) {
        thread::sleep(std::time::Duration::from_secs(self.0 as u64));
        println!("{}", self.0);
    }
}