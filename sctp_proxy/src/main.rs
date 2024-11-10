use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::{io, mem, thread};
use libc::{sa_family_t, AF_INET};
use utils::libc_wrappers::{debug_sctp_sndrcvinfo, debug_sockaddr, safe_inet_pton, SctpSenderInfo, SockAddrIn};
use utils::sctp::sctp_api::{safe_sctp_recvmsg, safe_sctp_sendmsg, safe_sctp_socket, SctpEventSubscribe, SctpPeerBuilder};
use utils::sctp::sctp_proxy::SctpProxyBuilder;
use io::Result;

const PORT: u16 = 7878;
const SCTP_PEER_IPV4: [Ipv4Addr;1] = [Ipv4Addr::new(127, 0, 0, 1)];
const SCTP_IPV4: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1) ;
const TCP_IPV4: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
fn main() -> Result<()> {

    let mut proxy = SctpProxyBuilder::new()
        .port(PORT)
        .tcp_address(TCP_IPV4)
        .sctp_address(SCTP_IPV4)
        .sctp_peer_addresses(SCTP_PEER_IPV4.to_vec())
        .build();

    proxy.start()

}