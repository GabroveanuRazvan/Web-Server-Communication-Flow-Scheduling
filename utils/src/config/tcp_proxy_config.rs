use crate::constants::{CONFIG_LOAD_ERROR, DEFAULT_TCP_PROXY_CONFIG_PATH, DEFAULT_TCP_SERVER_CONFIG_PATH};
use std::env;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use crate::config::serialization::load;
use crate::constants::TCP_PROXY_CONFIG_PATH_ENV;

static TCP_PROXY_CONFIG: OnceLock<TcpProxyConfig> = OnceLock::new();

#[derive(Debug,Serialize,Deserialize)]
pub struct TcpProxyConfig{
    
    browser_server_address: Ipv4Addr,
    browser_server_port: u16,
    
    peer_address: Ipv4Addr,
    peer_port: u16,
    
    thread_count: usize,
    // Used in tcp assoc configurations
    stream_count: u8,
    
}

impl TcpProxyConfig{
    
    pub fn get_config() -> &'static Self{
        
        TCP_PROXY_CONFIG.get_or_init(|| {
            
            let tcp_config = match env::var(TCP_PROXY_CONFIG_PATH_ENV){
                Ok(config_path) => load::<TcpProxyConfig,&Path>(Path::new(&config_path)).expect(&format!("{CONFIG_LOAD_ERROR} {}",config_path.as_str())),
                Err(_) => load::<TcpProxyConfig,&Path>(Path::new(DEFAULT_TCP_PROXY_CONFIG_PATH)).expect(&format!("{CONFIG_LOAD_ERROR} {DEFAULT_TCP_SERVER_CONFIG_PATH}")),
            };
            
            tcp_config
            
            
        })
    
    }
    
    pub fn browser_server_address() -> &'static Ipv4Addr {
        &Self::get_config().browser_server_address
    }
    
    pub fn browser_server_port() -> u16 {
        Self::get_config().browser_server_port
    }
    
    pub fn peer_address() -> &'static Ipv4Addr {
        &Self::get_config().peer_address
    }
    
    pub fn peer_port() -> u16 {
        Self::get_config().peer_port
    }
    
    pub fn thread_count() -> usize {
        Self::get_config().thread_count
    }
    
    pub fn stream_count() -> u8 {
        Self::get_config().stream_count
    }
    
}

#[cfg(test)]

mod tests{
    use std::hint::assert_unchecked;
    use super::*;
    
    #[test]
    fn test_tcp_proxy_config1(){
        
        env::set_var(TCP_PROXY_CONFIG_PATH_ENV,"./tests/tcp_proxy_config.json");
        TcpProxyConfig::get_config();
        
    }

    #[test]
    fn test_tcp_proxy_config2(){

        env::set_var(TCP_PROXY_CONFIG_PATH_ENV,"./tests/tcp_proxy_config.json");
        
        assert_eq!(TcpProxyConfig::thread_count(),12);
        assert_eq!(*TcpProxyConfig::peer_address(),Ipv4Addr::new(192,168,1,143));
        assert_eq!(TcpProxyConfig::peer_port(),7878);
        assert_eq!(*TcpProxyConfig::browser_server_address(),Ipv4Addr::UNSPECIFIED);
        assert_eq!(TcpProxyConfig::browser_server_port(),7879);
        assert_eq!(TcpProxyConfig::stream_count(),8);
    }
    
    
}