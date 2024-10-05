
mod modules;

use std::net::Ipv4Addr;
use crate::modules::sctp_server::SctpServerBuilder;

fn main() {

    let s = SctpServerBuilder::new()
        .descriptor()
        .address(Ipv4Addr::new(127, 0, 0, 1))
        .port(7878)
        .build();

    println!("{:?}",s);

}
