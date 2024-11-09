use std::sync::OnceLock;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::cmp::Reverse;
use utils::shortest_job_first_pool::{Job, SjfPool};


static CEVA: OnceLock<i32> = OnceLock::new();
fn main() {
    CEVA.set(2).unwrap();

    println!("{}",CEVA.get());

}
