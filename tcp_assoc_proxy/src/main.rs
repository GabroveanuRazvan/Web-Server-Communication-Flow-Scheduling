use std::io;
use utils::config::tcp_proxy_config::TcpProxyConfig;
use utils::tcp::tcp_assoc_proxy::TcpAssocRelayBuilder;
use io::Result;

fn main() -> Result<()>{

    let stream_count = TcpProxyConfig::stream_count();
    let thread_count = TcpProxyConfig::thread_count();
    let browser_ipv4 = *TcpProxyConfig::browser_server_address();
    let browser_port = TcpProxyConfig::browser_server_port();
    let peer_ipv4 = *TcpProxyConfig::peer_address();
    let peer_port = TcpProxyConfig::peer_port();

    let tcp_assoc_proxy = TcpAssocRelayBuilder::new()
        .stream_count(stream_count)
        .worker_count(thread_count)
        .peer_port(peer_port)
        .peer_ipv4(peer_ipv4)
        .browser_port(browser_port)
        .browser_ipv4(browser_ipv4)
        .build();
    
    tcp_assoc_proxy.start()

}
