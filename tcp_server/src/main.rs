use std::env;
use utils::config::tcp_server_config::TcpServerConfig;
use utils::tcp::tcp_server::TcpServerBuilder;
use std::io::Result;
use utils::constants::TCP_SERVER_CONFIG_PATH_ENV;

fn main() -> Result<()>{
    
    env::set_var(TCP_SERVER_CONFIG_PATH_ENV,"./server_config.json");
    
    let server = TcpServerBuilder::new()
        .ipv4_address(TcpServerConfig::address().clone())
        .port(TcpServerConfig::port())
        .worker_count(TcpServerConfig::thread_count())
        .root(TcpServerConfig::server_root())
        .file_packet_size(TcpServerConfig::file_packet_size())
        .build();
    
    server.start()
    
}
