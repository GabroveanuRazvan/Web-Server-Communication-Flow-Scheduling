use std::net::Ipv4Addr;
use std::sync::OnceLock;

pub static THREAD_POOL_SIZE: OnceLock<usize> = OnceLock::new();
pub static CACHE_CAPACITY: OnceLock<usize> = OnceLock::new();
pub static CHUNK_SIZE: OnceLock<usize> = OnceLock::new();
pub static BUFFER_SIZE: OnceLock<usize> = OnceLock::new();
pub static SCTP_PEER_IPV4: OnceLock<Vec<Ipv4Addr>> = OnceLock::new();
pub static SCTP_IPV4: OnceLock<Ipv4Addr> = OnceLock::new();
pub static TCP_IPV4: OnceLock<Ipv4Addr> = OnceLock::new();
pub static SERVER_PATH: OnceLock<String> = OnceLock::new();
pub static SERVER_IPV4: OnceLock<Ipv4Addr> = OnceLock::new();
pub static MAX_CONNECTIONS: OnceLock<u16> = OnceLock::new();
pub static CACHE_MANAGER_PATH: OnceLock<String> = OnceLock::new();


pub const BYTE: usize = 1;
pub const KILOBYTE: usize = 1024 * BYTE;
pub const MEGABYTE: usize = KILOBYTE * 1024;