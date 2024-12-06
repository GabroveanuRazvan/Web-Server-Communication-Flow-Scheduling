use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::fd::FromRawFd;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use libc::{O_CREAT, O_EXCL, O_RDWR};
use utils::constants::KILOBYTE;
use utils::libc_wrappers::{safe_shm_open, safe_shm_unlink, ModeBuilder};
use utils::shared_memory::SharedMemory;

fn main() -> std::io::Result<()> {

    let mode = ModeBuilder::new()
        .user_execute()
        .user_read()
        .user_write()
        .build();

    let mut shm = SharedMemory::recreate("me2re",mode,1 * KILOBYTE)?;

    println!("{shm:?}");


    thread::sleep(Duration::from_secs(15));
    Ok(())
}
