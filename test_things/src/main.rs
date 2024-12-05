use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use libc::{O_CREAT, O_RDWR};
use utils::libc_wrappers::{safe_shm_open, safe_shm_unlink};

fn main() -> std::io::Result<()> {

    let res = safe_shm_open("/anamereare",O_RDWR | O_CREAT, 0o666);
    println!("res: {:?}", res);

    let res = safe_shm_unlink("/anamereare");

    Ok(())
}
