use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddrV4, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use utils::constants::{KILOBYTE};
use utils::http_parsers::parse_root;
use utils::sctp::sctp_api::SctpPeerBuilder;
use utils::sctp::sctp_client::{SctpStreamBuilder};

const PEER_ADDRESS: &str = "192.168.50.30:7878";
const RECEIVE_BUFFER_SIZE: usize = 16 * KILOBYTE;
const USER_COUNT: usize = 6;
const DATASET_ROOT : &str = "../benchmarking/raw_dataset";

fn main() {

    let requests = Arc::new(Mutex::new(parse_root(DATASET_ROOT).unwrap()));
    let mut threads = Vec::with_capacity(USER_COUNT);

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

    for idx in 0..USER_COUNT {
        
        let user = user.try_clone().unwrap();
        let stream_number = idx as u16;
        let mut buffer = [0u8; RECEIVE_BUFFER_SIZE];

        let mut total_size = 0;
        let mut total_time = 0.0;
        let requests = Arc::clone(&requests);
        
        threads.push(
            thread::spawn(move || {
                
                loop{
                    
                    let request = {
                        let mut guard = requests.lock().unwrap();

                        match guard.pop_back(){
                            Some(request) => request,
                            None => break,
                        }

                    };

                    let start = Instant::now();
                    
                    let http_get = format!("GET {} HTTP/1.1\r\n\r\n",request);
                    user.write_all(http_get.as_bytes(),stream_number,0,0).unwrap();

                    let header_size = user.read(&mut buffer,None,None).unwrap();
                    println!("{}",String::from_utf8_lossy(&buffer[..header_size]));
                    let file_size = extract_content_length(&buffer[..header_size]).unwrap();
                    let mut current_length = 0;

                    while current_length < file_size {
                        let bytes_received = user.read(&mut buffer,None,None).unwrap();
                        current_length += bytes_received;
                    }

                    let end = start.elapsed().as_secs_f64();

                    total_time += end;
                    total_size += file_size;
                }

                let throughput = total_size as f64 / total_time ;

                (total_time,total_size,throughput)
                
            })
        );
    }
        let mut avg_throughput = 0.0;
        let mut avg_time = 0.0;

        
        threads.into_iter().for_each(|thread| {
            let (time,size,throughput) = thread.join().unwrap();
            avg_throughput += throughput;
            avg_time += time;
        });

        avg_throughput /=  USER_COUNT as f64;
        avg_time /= USER_COUNT as f64;

        println!("Avg time: {avg_time}");
        println!("Avg throughput: {avg_throughput}");
        
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
