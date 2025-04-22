

/// /////////////////////////
/// Memory representation ///
/// /////////////////////////

pub const BYTE: usize = 1;
pub const KILOBYTE: usize = 1024 * BYTE;
pub const MEGABYTE: usize = KILOBYTE * 1024;


/// /////////////////////////////
/// Sctp server configuration ///
/// /////////////////////////////

pub const DEFAULT_SERVER_CONFIG_PATH: &str = "./server_config.json";
pub const SERVER_CONFIG_PATH_ENV: &str = "SERVER_CONFIG_PATH";
pub const CONFIG_LOAD_ERROR: &str = "Failed to load config file";
pub const MAX_CONNECTIONS: u16 = 100;
pub const SERVER_RECEIVE_BUFFER_SIZE: usize = 32 * KILOBYTE;

/// ////////////////////////////
/// Sctp proxy configuration ///
/// ////////////////////////////

pub const SCTP_PROXY_CONFIG_PATH_ENV: &str = "SCTP_PROXY_CONFIG_PATH";
pub const DEFAULT_SCTP_PROXY_CONFIG_PATH: &str = "./sctp_proxy_config.json";
pub const PACKET_BUFFER_SIZE: usize = 64 * KILOBYTE;


/// ///////////////////////////
/// Tcp proxy configuration ///
/// //////////////////////////
pub const REQUEST_BUFFER_SIZE: usize = 4 * KILOBYTE;
pub const INOTIFY_BUFFER_SIZE: usize = 16 * KILOBYTE;
pub const BROWSER_CHUNK_SIZE: usize = 32 * KILOBYTE;

/// //////////////////////
/// Schedulers macros ///
/// ////////////////////


pub const METADATA_STATIC_SIZE: usize = 8 * BYTE;
pub const CHUNK_METADATA_SIZE: usize = 0 * BYTE;