use std::env;
use std::net::{Ipv4Addr};
use std::sync::{OnceLock};
use serde::{Deserialize, Serialize};
use crate::config::serialization::load;
use crate::constants::{CONFIG_LOAD_ERROR, DEFAULT_TCP_SERVER_CONFIG_PATH, TCP_SERVER_CONFIG_PATH_ENV};
use std::path::{Path, PathBuf};

static TCP_SERVER_CONFIG: OnceLock<TcpServerConfig> = OnceLock::new();

#[derive(Debug,Serialize,Deserialize)]
pub struct TcpServerConfig{
    
    address: Ipv4Addr,
    port: u16,
    thread_count: usize,
    server_root: PathBuf,
    file_packet_size: usize,
    
}

impl TcpServerConfig {
    
    pub fn get_config() -> &'static Self{
        
        TCP_SERVER_CONFIG.get_or_init(|| {
            
            let tcp_config = match env::var(TCP_SERVER_CONFIG_PATH_ENV){
                Ok(config_path) => load::<TcpServerConfig,&Path>(Path::new(&config_path)).expect(&format!("{CONFIG_LOAD_ERROR} {}",config_path.as_str())),
                Err(_) => load::<TcpServerConfig,&Path>(Path::new(DEFAULT_TCP_SERVER_CONFIG_PATH)).expect(&format!("{CONFIG_LOAD_ERROR} {DEFAULT_TCP_SERVER_CONFIG_PATH}")),
             };
            
            tcp_config
            
        })
        
    }
    
    pub fn address() -> &'static Ipv4Addr{
        &Self::get_config().address
    }
    pub fn port() -> u16{
        Self::get_config().port
    }
    pub fn thread_count() -> usize {
        Self::get_config().thread_count
    }
    
    pub fn server_root() -> &'static Path{
        Self::get_config().server_root.as_path()
    }
    pub fn file_packet_size() -> usize {
        Self::get_config().file_packet_size
    }
    
}

#[cfg(test)]

mod tests{
    use crate::config::sctp_proxy_config::SctpProxyConfig;
    use crate::constants::KILOBYTE;
    use super::*;
    
    #[test]
    fn test_tcp_server_config1(){
        env::set_var(TCP_SERVER_CONFIG_PATH_ENV,"./tests/tcp_server_config.json");
        let config = SctpProxyConfig::get_config();
    }
    
    #[test]
    fn test_tcp_server_config2(){
        env::set_var(TCP_SERVER_CONFIG_PATH_ENV,"./tests/tcp_server_config.json");
        let config = SctpProxyConfig::get_config();
        
        assert_eq!(TcpServerConfig::address().clone(),Ipv4Addr::UNSPECIFIED);
        assert_eq!(TcpServerConfig::port(),7878);
        assert_eq!(TcpServerConfig::server_root(),Path::new("./web_files"));
        assert_eq!(TcpServerConfig::file_packet_size(),16 * KILOBYTE);
        assert_eq!(TcpServerConfig::thread_count(),12);
    }
}