use std::net::Ipv4Addr;
use std::sync::Arc;
use std::thread;
use utils::cache::lru_cache::TempFileCache;
use utils::constants::BYTE;
use utils::sctp::sctp_api::{SctpEventSubscribe, SctpPeerBuilder};
use utils::sctp::sctp_client::SctpStreamBuilder;

fn main() {

    let mut cache = TempFileCache::new(10 * BYTE);

    for i in 0..9{
        cache.insert(format!("mere{i}").to_string());
        cache.write_append(&format!("mere{i}").to_string(),b"1").unwrap()
    }

    println!("{cache:#?}");
    cache.make_room(10);
    println!("{cache:#?}");
}
