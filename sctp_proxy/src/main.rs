use std::net::{Ipv4Addr};
use std::{io};
use utils::sctp::sctp_api::{SctpPeerBuilder};
use utils::sctp::sctp_proxy::SctpProxyBuilder;
use io::Result;
use utils::config::sctp_proxy_config::SctpProxyConfig;

fn main() -> Result<()> {

    let port = SctpProxyConfig::sctp_port();
    let addresses = SctpProxyConfig::addresses().to_vec();

    let mut proxy = SctpProxyBuilder::new()
        .port(port)
        .sctp_peer_addresses(addresses)
        .build();

    proxy.start()

}