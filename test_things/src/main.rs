use std::net::{Ipv4Addr, SocketAddrV4};
use utils::libc_wrappers::{sock_addr_to_c, c_to_sock_addr, debug_sockaddr};
fn main() {

    let mut x = SocketAddrV4::new(Ipv4Addr::new(88,127,100,97), 8080);

    println!("{x:?}");

    let mut y = sock_addr_to_c(&x);

    debug_sockaddr(&y);

    let mut z = c_to_sock_addr(&y);

    println!("{z:?}");

}
