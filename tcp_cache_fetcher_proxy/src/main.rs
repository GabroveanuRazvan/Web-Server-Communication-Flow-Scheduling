use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use utils::tcp_proxy::TcpProxyBuilder;
use io::Result;
fn main() -> Result<()> {

    let mut tcp_proxy = TcpProxyBuilder::new()
        .port(7879)
        .tcp_address(Ipv4Addr::UNSPECIFIED)
        .sctp_proxy_ipv4(Ipv4Addr::new(127, 0, 0, 1))
        .sctp_proxy_port(7878)
        .build();

    tcp_proxy.start()?;
    Ok(())
}
