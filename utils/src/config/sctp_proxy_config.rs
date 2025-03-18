use std::env;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use crate::config::serialization::load;
use crate::constants::{CONFIG_LOAD_ERROR, DEFAULT_SCTP_PROXY_CONFIG_PATH, SCTP_PROXY_CONFIG_PATH_ENV};

static SCTP_PROXY_CONFIG: OnceLock<SctpProxyConfig> = OnceLock::new();

#[derive(Debug,Serialize,Deserialize)]
pub struct SctpProxyConfig{
    addresses: Vec<Ipv4Addr>,
    port: u16,
    tcp_address: Ipv4Addr,
    cache_path: PathBuf,
    download_suffix: String,
    default_outgoing_streams: u16,
    max_incoming_streams: u16,
}

impl SctpProxyConfig {

    pub fn get_config() -> &'static SctpProxyConfig {

        let mut sctp_config = match env::var(SCTP_PROXY_CONFIG_PATH_ENV){
            Ok(config_path) => load::<SctpProxyConfig,&Path>(Path::new(&config_path)).expect(&format!("{} {}", CONFIG_LOAD_ERROR, config_path.as_str())),
            Err(_) => load::<SctpProxyConfig,&Path>(Path::new(DEFAULT_SCTP_PROXY_CONFIG_PATH)).expect(&format!("{} {}",CONFIG_LOAD_ERROR, DEFAULT_SCTP_PROXY_CONFIG_PATH)),
        };

        // Add a "." at the start of the download suffix if it was not provided
        if !sctp_config.download_suffix.starts_with("."){
            sctp_config.download_suffix = format!(".{}",sctp_config.download_suffix)
        }

        SCTP_PROXY_CONFIG.get_or_init(||{
          sctp_config
        })

    }

    pub fn addresses() -> &'static [Ipv4Addr] {
        Self::get_config().addresses.as_slice()
    }

    pub fn port() -> u16{
        Self::get_config().port
    }

    pub fn tcp_address() -> &'static Ipv4Addr{
        &Self::get_config().tcp_address
    }

    pub fn cache_path() -> &'static Path {
        Self::get_config().cache_path.as_path()
    }
    pub fn download_suffix() -> &'static str {
        Self::get_config().download_suffix.as_str()
    }

    pub fn default_outgoing_streams() -> u16 {
        Self::get_config().default_outgoing_streams
    }
    pub fn max_incoming_streams() -> u16 {
        Self::get_config().max_incoming_streams
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_sctp_proxy_config1() {
        env::set_var(SCTP_PROXY_CONFIG_PATH_ENV,"./tests/sctp_proxy_config.json");
        let config = SctpProxyConfig::get_config();
    }

    #[test]
    fn test_sctp_proxy_config2() {
        env::set_var(SCTP_PROXY_CONFIG_PATH_ENV,"./tests/sctp_proxy_config.json");
        let config = SctpProxyConfig::get_config();

        assert_eq!(SctpProxyConfig::addresses().len(), 1);
        assert_eq!(SctpProxyConfig::port(),7878);
        assert_eq!(*SctpProxyConfig::tcp_address(),Ipv4Addr::UNSPECIFIED);
        assert_eq!(SctpProxyConfig::cache_path(),PathBuf::from("/tmp/tmpfs"));
        assert_eq!(SctpProxyConfig::download_suffix(),".tmp");
        assert_eq!(SctpProxyConfig::default_outgoing_streams(),10);
        assert_eq!(SctpProxyConfig::max_incoming_streams(),30);

    }

}