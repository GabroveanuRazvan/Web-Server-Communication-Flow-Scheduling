use std::io::Result;
use std::net::Ipv4Addr;
use utils::sctp::sctp_server::{SctpServer, SctpServerBuilder};
use utils::sctp::sctp_api::{SctpPeerBuilder, SctpEventSubscribeBuilder};
use std::path::Path;
//netstat -lnp | grep sctp

const MAX_CONNECTIONS: u16 = 100;
const PORT: u16 = 7878;
const IPV4: Ipv4Addr = Ipv4Addr::UNSPECIFIED;
const PATH_STR: &str = "./web_files";

fn main() -> Result<()> {
    let events = SctpEventSubscribeBuilder::new().sctp_data_io_event().build();

    let mut server = SctpServerBuilder::new()
        .socket()
        .address(IPV4)
        .port(PORT)
        .max_connections(MAX_CONNECTIONS)
        .events(events)
        .path(Path::new(PATH_STR))
        .build();

    server.bind()
          .listen()
          .options();

    println!("Server started and listening on {IPV4:?}:{PORT}");
    println!("Current directory: {PATH_STR}");
    println!("Connect by: http://127.0.0.1:{PORT}");

    for stream in server.incoming(){

        let stream = stream?;

        SctpServer::handle_client(stream)?

    }



    Ok(())
}
