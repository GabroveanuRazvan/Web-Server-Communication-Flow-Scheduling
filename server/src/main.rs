use std::io::Result;
use std::num::NonZero;
use utils::sctp::sctp_server::{SctpServer, SctpServerBuilder};
use utils::sctp::sctp_api::{SctpPeerBuilder, SctpEventSubscribeBuilder};
use std::thread;
use utils::config::sctp_server_config::SctpServerConfig;
use utils::constants::{MAX_CONNECTIONS};

//netstat -lnp | grep sctp
fn main() -> Result<()> {

    let ipv4 = SctpServerConfig::ipv4();
    let port = SctpServerConfig::port();
    let server_root = SctpServerConfig::root();
    let outgoing_streams = SctpServerConfig::default_outgoing_streams();
    let incoming_streams = SctpServerConfig::max_incoming_streams();

    let events = SctpEventSubscribeBuilder::new().sctp_data_io_event().build();
    let num_cpus = thread::available_parallelism().unwrap_or(NonZero::new(outgoing_streams as usize).unwrap()).get();



    let mut server = SctpServerBuilder::new()
        .socket()
        .address(ipv4)
        .port(port)
        .max_connections(MAX_CONNECTIONS)
        .events(events)
        .root(server_root)
        .set_outgoing_streams(num_cpus as u16)
        .set_incoming_streams(incoming_streams)
        .build();

    server.bind()
          .listen()
          .set_events();


    println!("Server started and listening on {ipv4:?}:{port}");
    println!("Current directory: {}",server_root.display());

    for stream in server.incoming(){

        let stream = stream?;
        server.handle_client(stream)?

    }



    Ok(())
}
