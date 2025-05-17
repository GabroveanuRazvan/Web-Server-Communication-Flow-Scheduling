use std::env;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use crate::config::serialization::load;
use crate::constants::{KILOBYTE,DEFAULT_SERVER_CONFIG_PATH,SERVER_CONFIG_PATH_ENV,CONFIG_LOAD_ERROR};
use crate::pools::scheduling::scheduling_policy::SchedulingPolicy;

static TCP_ASSOC_SERVER_CONFIG: OnceLock<TcpAssocServerConfig> = OnceLock::new();

#[derive(Debug,Serialize,Deserialize)]
pub struct TcpAssocServerConfig {
    address: Ipv4Addr,
    port: u16,
    root: PathBuf,
    stream_count: u8,
    scheduling_policy: u8,
    file_packet_size: usize,
}

impl Default for TcpAssocServerConfig {

    fn default() -> Self {

        Self{
            address: Ipv4Addr::UNSPECIFIED,
            port: 0,
            root: PathBuf::default(),
            stream_count: 0,
            scheduling_policy: 0,
            file_packet_size: 32 * KILOBYTE,
        }

    }
}

impl TcpAssocServerConfig {

    pub fn get_config() -> &'static TcpAssocServerConfig {

        TCP_ASSOC_SERVER_CONFIG.get_or_init(||{
            match env::var(SERVER_CONFIG_PATH_ENV){
                Ok(config_path) => load::<TcpAssocServerConfig,&Path>(Path::new(&config_path)).expect(&format!("{} {}", CONFIG_LOAD_ERROR, config_path.as_str())),
                Err(_) => load::<TcpAssocServerConfig,&Path>(Path::new(DEFAULT_SERVER_CONFIG_PATH)).expect(&format!("{} {}",CONFIG_LOAD_ERROR, DEFAULT_SERVER_CONFIG_PATH)),
            }
        })

    }

    pub fn scheduling_policy() -> SchedulingPolicy {Self::get_config().scheduling_policy.into()}
    pub fn address() -> &'static Ipv4Addr {&Self::get_config().address}
    pub fn port() -> u16 {
        Self::get_config().port.clone()
    }
    pub fn root() -> &'static Path {
        Self::get_config().root.as_path()
    }
    pub fn  stream_count() -> u8 {Self::get_config().stream_count}
    pub fn file_packet_size() -> usize {
        Self::get_config().file_packet_size
    }


}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_tcp_assoc_server_config1() {
        env::set_var(SERVER_CONFIG_PATH_ENV,"./tests/tcp_assoc_server_config.json");
        let config = TcpAssocServerConfig::get_config();
    }

    #[test]
    fn test_tcp_assoc_server_config2() {
        env::set_var(SERVER_CONFIG_PATH_ENV,"./tests/tcp_assoc_server_config.json");
        let config = TcpAssocServerConfig::get_config();
        
        assert_eq!(TcpAssocServerConfig::scheduling_policy(), SchedulingPolicy::RoundRobin);
        assert_eq!(TcpAssocServerConfig::stream_count(), 12);
        assert_eq!(TcpAssocServerConfig::file_packet_size(), 65536);
        assert_eq!(config.address, Ipv4Addr::new(127,0,0,1));
    }

}