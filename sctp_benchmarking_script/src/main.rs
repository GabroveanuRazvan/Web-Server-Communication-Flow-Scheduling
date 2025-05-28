use std::net::{SocketAddrV4};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use utils::config::serialization::{load, save};
use utils::constants::{KILOBYTE, MEGABYTE};
use utils::libc_wrappers::SocketBuffers;
use utils::sctp::sctp_api::SctpPeerBuilder;
use utils::sctp::sctp_client::{SctpStreamBuilder};

const REQUESTS_PATH_TEMPLATE: &str = "../benchmarking/requests/requests_";
const EVENTS_PATH_TEMPLATE: &str = "./events_list_";

const PEER_ADDRESS: &str = "192.168.50.30:7878";
const RECEIVE_BUFFER_SIZE: usize = 1 * MEGABYTE;


fn main() {

    for i in 0..12{
        
        let requests_path = format!("{REQUESTS_PATH_TEMPLATE}{i}.json");
        let events_path = format!("{EVENTS_PATH_TEMPLATE}{i}.json");

        let mut buffer = [0u8;64 * KILOBYTE];
        let requests: Vec<PathBuf> = load(requests_path).unwrap();
        let num_requests = requests.len();
        let mut events = vec![LocustEvent::default(); num_requests];

        let socket_address: SocketAddrV4 = PEER_ADDRESS.parse().unwrap();

        // Create the sctp client and connect
        let mut sctp_client = SctpStreamBuilder::new()
            .socket()
            .address(socket_address.ip().clone())
            .port(socket_address.port())
            .set_incoming_streams(10)
            .set_outgoing_streams(10)
            .ttl(0)
            .build();

        sctp_client.connect();


        sctp_client.set_receive_buffer_size(RECEIVE_BUFFER_SIZE).unwrap();

        println!("Receive buffer size: {}",sctp_client.get_receive_buffer_size().unwrap());

        thread::sleep(Duration::from_secs(5));

        let mut total_time = 0f64;
        let mut total_size = 0usize;

        for (idx,request) in requests.iter().enumerate() {
            let http_header = HttpGetHeader(request);

            // Send the request
            sctp_client.write_all(http_header.as_bytes(), 0, 0, 0).unwrap();

            // Get the response
            sctp_client.read(&mut buffer, None, None).unwrap();

            let file_size = extract_content_length(&buffer).expect("Failed to extract content");
            let mut current_size = 0;

            let start = Instant::now();

            while current_size < file_size {
                let bytes_received = sctp_client.read(&mut buffer, None, None).unwrap();
                current_size += bytes_received;
            }

            let end = start.elapsed().as_secs_f64();

            // Store the request data
            total_time += end;
            total_size +=  file_size;

            events[idx] = LocustEvent::new(String::from("SCTP"), format!("GET {}", request.display()), end, file_size);
            println!("{idx}");
        }

        // Compute the throughput and store the data as json files
        let throughput = total_size as f64 / total_time;

        let data = LocustData::new(events, total_time,throughput);
        save(data,events_path).unwrap();
        
    }
    

}

#[derive(Debug,Clone,Serialize,Deserialize)]
struct LocustData{
    total_time: f64,
    throughput: f64,
    events: Vec<LocustEvent>,
    
}

impl LocustData{
    fn new(events: Vec<LocustEvent>, total_time: f64,throughput: f64)->Self{
        Self{
            total_time,
            throughput,
            events,
        }
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
struct LocustEvent{

    request_type: String,
    name: String,
    response_time: f64,
    response_length: usize,

}

impl Default for LocustEvent{
    fn default() -> Self{
        Self{
            request_type: String::new(),
            name: String::new(),
            response_time: 0.0,
            response_length: 0,
        }
    }
}

impl LocustEvent{
    pub fn new(request_type: String, name: String, response_time: f64, response_length: usize) -> Self{
        Self{
            request_type,
            name,
            response_time,
            response_length,
        }
    }
}


pub fn HttpGetHeader(file_path: impl AsRef<Path>) -> String{
    format!("GET {} HTTP/1.1\r\nHost: rust\r\n\r\n",file_path.as_ref().to_str().expect("Http Header"))
}

pub fn extract_content_length(buffer: &[u8]) -> Option<usize>{

    let text = String::from_utf8_lossy(buffer);

    for line in text.lines(){

        if let Some(rest) = line.strip_prefix("content-length: ") {
            return rest.trim().parse::<usize>().ok();
        }

    }

    None

}

//sudo sysctl -w net.core.rmem_max=1048576
//sudo sysctl -w net.core.wmem_max=1048576