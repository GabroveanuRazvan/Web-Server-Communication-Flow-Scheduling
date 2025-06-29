use std::env;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use crate::config::serialization::load;
use crate::constants::{KILOBYTE,DEFAULT_SERVER_CONFIG_PATH,SERVER_CONFIG_PATH_ENV,CONFIG_LOAD_ERROR};
use crate::pools::scheduling::scheduling_policy::SchedulingPolicy;

static SCTP_SERVER_CONFIG: OnceLock<SctpServerConfig> = OnceLock::new();

#[derive(Debug,Serialize,Deserialize)]
pub struct SctpServerConfig {
    addresses: Vec<Ipv4Addr>,
    port: u16,
    root: PathBuf,
    default_outgoing_streams: u16,
    scheduling_policy: u8,
    max_incoming_streams: u16,
    file_packet_size: usize,
}

impl Default for SctpServerConfig {

    fn default() -> Self {

        Self{
            addresses: Vec::new(),
            port: 0,
            root: PathBuf::default(),
            default_outgoing_streams: 10,
            max_incoming_streams: 10,
            scheduling_policy: 0,
            file_packet_size: 32 * KILOBYTE,
        }

    }
}

impl SctpServerConfig {

    pub fn get_config() -> &'static SctpServerConfig {

        SCTP_SERVER_CONFIG.get_or_init(||{
            match env::var(SERVER_CONFIG_PATH_ENV){
                Ok(config_path) => load::<SctpServerConfig,&Path>(Path::new(&config_path)).expect(&format!("{} {}", CONFIG_LOAD_ERROR, config_path.as_str())),
                Err(_) => load::<SctpServerConfig,&Path>(Path::new(DEFAULT_SERVER_CONFIG_PATH)).expect(&format!("{} {}",CONFIG_LOAD_ERROR, DEFAULT_SERVER_CONFIG_PATH)),
            }
        })

    }

    pub fn scheduling_policy() -> SchedulingPolicy {Self::get_config().scheduling_policy.into()}
    pub fn addresses() -> &'static [Ipv4Addr] {
        Self::get_config().addresses.as_slice()
    }
    pub fn port() -> u16 {
        Self::get_config().port.clone()
    }
    pub fn root() -> &'static Path {
        Self::get_config().root.as_path()
    }
    pub fn default_outgoing_streams() -> u16 {
        Self::get_config().default_outgoing_streams
    }
    pub fn max_incoming_streams() -> u16 {
        Self::get_config().max_incoming_streams
    }
    pub fn file_packet_size() -> usize {
        Self::get_config().file_packet_size
    }


}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_sctp_server_config1() {
        env::set_var(SERVER_CONFIG_PATH_ENV,"./tests/server_config.json");
        let config = SctpServerConfig::get_config();
    }

    #[test]
    fn test_sctp_server_config2() {
        env::set_var(SERVER_CONFIG_PATH_ENV,"./tests/server_config.json");
        let config = SctpServerConfig::get_config();

        assert_eq!(SctpServerConfig::max_incoming_streams(),10);
        assert_eq!(SctpServerConfig::file_packet_size(),65536);
        assert_eq!(SctpServerConfig::addresses()[0],Ipv4Addr::UNSPECIFIED);
        assert_eq!(SctpServerConfig::addresses()[1],Ipv4Addr::new(192,168,0,1));
        assert_eq!(SctpServerConfig::scheduling_policy(), SchedulingPolicy::RoundRobin);
    }

}