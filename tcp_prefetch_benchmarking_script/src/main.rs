use std::fmt::format;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::path::PathBuf;
use std::time::Instant;
use utils::config::serialization::load;
use utils::constants::{KILOBYTE, MEGABYTE};
use utils::tcp::tcp_extended::HtmlReadable;

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
    
    (0..USER_COUNT).for_each(|i| {
        
        let mut user_secs = 0.0;
        let mut user_total_size = 0;
        
        (0..requests_per_user).for_each(|j| {

            
            let start = Instant::now();
            let mut user = TcpStream::connect(PEER_ADDRESS).unwrap();

            let request_index = i * USER_COUNT + j;
            let request = requests[request_index].as_path();
            
            let http_get = format!("GET {} HTTP/1.1\r\n\r\n",request.display());
            user.write(http_get.as_bytes()).unwrap();

            let (response,residue) = get_http_header(&mut user);

            let file_size = extract_content_length(&response).unwrap();
            let mut current_length = residue.len();
            
            while current_length < file_size {
                let bytes_received = user.read(&mut buffer).unwrap();
                current_length += bytes_received;

            }
            
            user.shutdown(Shutdown::Both).unwrap();
            let end = start.elapsed().as_secs_f64();
            
            user_total_size += file_size;
            user_secs += end;
            
        });
        
        average_throughput = user_total_size as f64 / user_secs;
        total_time += user_secs;
        
        println!("User {i} time\t{user_secs}");
        println!("Total user time:\t{total_time}");
        
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