use std::io::Result;
use std::num::NonZero;
use utils::sctp::sctp_server::{SctpServerBuilder};
use utils::sctp::sctp_api::{SctpPeerBuilder, SctpEventSubscribeBuilder};
use std::thread;
use utils::config::sctp_server_config::SctpServerConfig;
use utils::constants::{MAX_CONNECTIONS};

//netstat -lnp | grep sctp
fn main() -> Result<()> {

    let addresses = SctpServerConfig::addresses();
    let port = SctpServerConfig::port();
    let server_root = SctpServerConfig::root();
    let outgoing_streams = SctpServerConfig::default_outgoing_streams();
    let incoming_streams = SctpServerConfig::max_incoming_streams();

    let events = SctpEventSubscribeBuilder::new().sctp_data_io_event().build();
    
    let mut server = SctpServerBuilder::new()
        .socket()
        .addresses(addresses.to_vec())
        .port(port)
        .max_connections(MAX_CONNECTIONS)
        .events(events)
        .root(server_root)
        .set_outgoing_streams(outgoing_streams)
        .set_incoming_streams(incoming_streams)
        .build();

    server.bind()
          .listen()
          .set_events();


    server.start()?;



    Ok(())
}

//For flamegraph to work: echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid