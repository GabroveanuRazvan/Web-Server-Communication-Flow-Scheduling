use std::io::{Cursor, Write};
use std::thread::sleep;
use std::time::Duration;
use utils::pools::thread_pool::ThreadPool;

fn main(){

    let bytes = vec![0u8;1];
    let mut cursor = Cursor::new(bytes);
    cursor.write(&32u64.to_ne_bytes()).unwrap();

    println!("{:?}",cursor);
}

