use std::{io};
use utils::sctp::sctp_api::{SctpPeerBuilder};
use utils::sctp::sctp_proxy::{SctpProxyBuilder, SctpRelayBuilder};
use io::Result;
use utils::config::sctp_proxy_config::SctpProxyConfig;

fn main() -> Result<()> {

    let port = SctpProxyConfig::sctp_port();
    let addresses = SctpProxyConfig::addresses().to_vec();

    if SctpProxyConfig::use_cache() {
        let mut proxy = SctpProxyBuilder::new()
            .port(port)
            .sctp_peer_addresses(addresses)
            .build();

        proxy.start()

    }else{

        let mut proxy = SctpRelayBuilder::new().port(port)
            .sctp_peer_addresses(addresses)
            .build();

        proxy.start()

    }



}