use std::io::Write;
use std::net::{SocketAddrV4, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Instant;
use utils::config::serialization::load;

const REQUESTS_PATH: &str = "./requests_list_10000.json";
const PEER_ADDRESS: &str = "192.168.50.251:7878";


fn main() {

    let requests: Vec<PathBuf> = load(REQUESTS_PATH).unwrap();
    let num_requests = requests.len();
    let mut events = vec![LocustEvent::default(); num_requests];
    let mut tcp_client = TcpStream::connect(PEER_ADDRESS).unwrap();

    for (idx,request) in requests.iter().enumerate() {

        let http_header = HttpGetHeader(request);

        let start = Instant::now();
        
        tcp_client.write_all(http_header.as_bytes()).unwrap();
        
        let end = start.elapsed().as_secs_f64();
    }

}

#[derive(Debug,Clone)]
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
    format!("GET {} HTTP/1.1\r\nHost: rust",file_path.as_ref().to_str().unwrap())
}