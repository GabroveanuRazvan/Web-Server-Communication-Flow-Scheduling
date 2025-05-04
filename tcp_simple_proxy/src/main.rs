use utils::config::tcp_proxy_config::TcpProxyConfig;
use utils::tcp::tcp_simple_proxy::{TcpSimpleProxy, TcpSimpleProxyBuilder};
use std::io::Result;

fn main() -> Result<()> {
    
    let tcp_proxy = TcpSimpleProxyBuilder::new()
        .worker_count(TcpProxyConfig::thread_count())
        .browser_ipv4(*TcpProxyConfig::browser_server_address())
        .browser_port(TcpProxyConfig::browser_server_port())
        .peer_connection_ipv4(*TcpProxyConfig::peer_address())
        .peer_connection_port(TcpProxyConfig::peer_port())
        .build();
    
    tcp_proxy.start()
    
}
