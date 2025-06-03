use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddrV4, TcpStream};
use std::path::PathBuf;
use std::time::Instant;
use utils::config::serialization::load;
use utils::constants::{MEGABYTE};
use utils::sctp::sctp_api::SctpPeerBuilder;
use utils::sctp::sctp_client::{SctpStream, SctpStreamBuilder};

const PEER_ADDRESS: &str = "192.168.50.30:7878";
const RECEIVE_BUFFER_SIZE: usize = 1 * MEGABYTE;
const USER_COUNT: usize = 6;
const REQUESTS_PATH : &str = "../benchmarking/requests/requests_0.json";

fn main() {

    let requests: Vec<PathBuf> = load(REQUESTS_PATH).unwrap();
    let request_count = requests.len();
    let requests_per_user =  request_count / USER_COUNT;

    let mut buffer = [0u8;RECEIVE_BUFFER_SIZE];

    let mut total_time = 0.0;
    let mut average_throughput = 0.0;

    let socket_address: SocketAddrV4 = PEER_ADDRESS.parse().unwrap();
    let mut user = SctpStreamBuilder::new()
        .socket()
        .address(socket_address.ip().clone())
        .port(socket_address.port())
        .set_incoming_streams(10)
        .set_outgoing_streams(10)
        .ttl(0)
        .build();
    
    user.connect();
    
    (0..USER_COUNT).for_each(|i| {

        let mut user_secs = 0.0;
        let mut user_total_size = 0;

        (0..requests_per_user).for_each(|j| {

            let start = Instant::now();

            let request_index = i * USER_COUNT + j;
            let request = requests[request_index].as_path();

            let http_get = format!("GET {} HTTP/1.1\r\n\r\n",request.display());
            user.write_all(http_get.as_bytes(),i as u16,0,0).unwrap();
                
            let header_size = user.read(&mut buffer,None,None).unwrap();

            let file_size = extract_content_length(&buffer[..header_size]).unwrap();
            let mut current_length = 0;

            while current_length < file_size {
                let bytes_received = user.read(&mut buffer,None,None).unwrap();
                current_length += bytes_received;
            }

            let end = start.elapsed().as_secs_f64();

            user_total_size += file_size;
            user_secs += end;

        });

        let user_throughput = user_total_size as f64 / user_secs;
        average_throughput += user_throughput;
        total_time += user_secs;

        println!("User {i} time\t{user_secs}");
        println!("User throughput:\t{user_throughput}");

    });

    average_throughput /= USER_COUNT as f64;

    println!("Total test time:\t{total_time}");
    println!("Average throughput:\t{average_throughput}");

}

fn extract_content_length(buffer: &[u8]) -> Option<usize>{

    let text = String::from_utf8_lossy(buffer);

    for line in text.lines(){

        if let Some(rest) = line.to_ascii_lowercase().strip_prefix("content-length: ") {
            return rest.trim().parse::<usize>().ok();
        }

    }

    None

}


// Total test time:        1399.9743759190005
// Average throughput:     8539839.710247297
//
//
// Total test time:        1618.7891264680002
// Average throughput:     7389004.508294408
