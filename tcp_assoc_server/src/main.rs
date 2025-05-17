use utils::config::tcp_assoc_server_config::TcpAssocServerConfig;
use utils::tcp::tcp_assoc_server::TcpAssocServerBuilder;
use std::io::Result;

fn main() -> Result<()> {
    
    let address = TcpAssocServerConfig::address();
    let port = TcpAssocServerConfig::port();
    let server_root = TcpAssocServerConfig::root();
    let stream_count = TcpAssocServerConfig::stream_count();
    
    
    let server = TcpAssocServerBuilder::new()
        .stream_count(stream_count)
        .ipv4(*address)
        .port(port)
        .server_root(server_root)
        .build();
    
    server.start()?;
    
    Ok(())
}
