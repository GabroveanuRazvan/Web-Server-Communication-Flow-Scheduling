use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use utils::tcp::tcp_proxy::TcpProxyBuilder;
use io::Result;
use utils::config::sctp_proxy_config::SctpProxyConfig;

fn main() -> Result<()> {

    let mut tcp_proxy = TcpProxyBuilder::new()
        .port(SctpProxyConfig::browser_server_port())
        .tcp_address(SctpProxyConfig::browser_server_address().clone())
        .build();

    tcp_proxy.start()?;
    Ok(())
}
