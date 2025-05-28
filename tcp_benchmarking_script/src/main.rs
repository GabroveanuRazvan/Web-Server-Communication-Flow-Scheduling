use std::io::{Read, Write};
use std::net::{TcpStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use utils::config::serialization::{load, save};
use utils::constants::{KILOBYTE, MEGABYTE};
use utils::libc_wrappers::SocketBuffers;

const REQUESTS_PATH_TEMPLATE: &str = "../benchmarking/requests/requests_";
const EVENTS_PATH_TEMPLATE: &str = "./events_list_";
const PEER_ADDRESS: &str = "192.168.50.30:7878";

const RECEIVE_BUFFER_SIZE: usize = 1 * MEGABYTE;


fn main() {

    for i in 0 .. 12{

        let requests_path = format!("{REQUESTS_PATH_TEMPLATE}{i}.json");
        let events_path = format!("{EVENTS_PATH_TEMPLATE}{i}.json");
        
        
        let requests: Vec<PathBuf> = load(requests_path).unwrap();
        let num_requests = requests.len();
        let mut events = vec![LocustEvent::default(); num_requests];
        let mut tcp_client = TcpStream::connect(PEER_ADDRESS).unwrap();

        tcp_client.set_receive_buffer_size(RECEIVE_BUFFER_SIZE).unwrap();

        println!("Receive buffer size: {}",tcp_client.get_receive_buffer_size().unwrap());

        let mut total_time = 0f64;
        let mut total_size = 0usize;

        thread::sleep(Duration::from_secs(1));

        for (idx,request) in requests.iter().enumerate() {

            let http_header = HttpGetHeader(request);
            
            tcp_client.write_all(http_header.as_bytes()).unwrap();

            let (response,residue) = get_http_header(&mut tcp_client);

            let file_size = extract_content_length(&response).unwrap();
            let mut current_length = residue.len();

            let mut buffer = [0u8;16 * KILOBYTE];

            let start = Instant::now();

            while current_length < file_size {

                let bytes_received = tcp_client.read(&mut buffer).unwrap();
                current_length += bytes_received;

            }

            let end = start.elapsed().as_secs_f64();

            total_time += end;
            total_size +=  file_size;

            events[idx] = LocustEvent::new(String::from("TCP"), format!("GET {}", request.display()), end, file_size);
            println!("{idx}");
        }

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
    format!("GET {} HTTP/1.1\r\nHost: rust\r\n\r\n",file_path.as_ref().to_str().unwrap())
}

pub fn extract_content_length(buffer: &[u8]) -> Option<usize>{

    let text = String::from_utf8_lossy(buffer);

    for line in text.lines(){

        if let Some(rest) = line.to_ascii_lowercase().strip_prefix("content-length: ") {
            return rest.trim().parse::<usize>().ok();
        }

    }

    None

}

pub fn get_http_header(stream: &mut TcpStream) -> (Vec<u8>,Vec<u8>){
    let mut header = Vec::new();
    let mut buffer = [0u8; 4 * KILOBYTE];

    let needle = b"\r\n\r\n";

    while !buffer.windows(needle.len()).any(|x| x == needle){

        let bytes_received = stream.read(&mut buffer).unwrap();
        header.extend_from_slice(&buffer[..bytes_received]);
    }

    let pos = header.windows(needle.len()).position(|x| x == needle).unwrap();
    let body = header.split_off(pos +  needle.len());

    (header,body)
}