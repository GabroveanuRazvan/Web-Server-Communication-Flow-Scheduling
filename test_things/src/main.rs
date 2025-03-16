
use std::process::{Command, Stdio};
use std::io::Write;
use utils::config::sctp_server_config::{SctpServerConfig};
use utils::sctp::sctp_server::SctpServer;

fn main() {


    println!("{:?}",SctpServerConfig::get_config());

}


