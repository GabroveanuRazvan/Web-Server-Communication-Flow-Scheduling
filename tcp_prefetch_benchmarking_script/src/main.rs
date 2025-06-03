use std::fmt::format;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use utils::config::serialization::load;
use utils::constants::{KILOBYTE, MEGABYTE};
use utils::http_parsers::parse_root;
use utils::tcp::tcp_extended::HtmlReadable;

const PEER_ADDRESS: &str = "192.168.50.30:7878";
const RECEIVE_BUFFER_SIZE: usize = 16 * KILOBYTE;
const USER_COUNT: usize = 6;
const DATASET_ROOT : &str = "../benchmarking/raw_dataset";

fn main() {
    
    let requests = Arc::new(Mutex::new(parse_root(DATASET_ROOT).unwrap()));
    println!("{:#?}",requests);
    thread::sleep(Duration::from_secs(10));
    let mut threads = Vec::with_capacity(USER_COUNT);
    
    for _ in 0..USER_COUNT{
        
        let requests = Arc::clone(&requests);
        threads.push(thread::spawn(move || {

            let mut buffer = [0u8;RECEIVE_BUFFER_SIZE];
            
            let mut total_size = 0;
            let mut total_time = 0.0;
            
            loop{
                
                let request = {
                    
                    let mut guard = requests.lock().unwrap();
                    match guard.pop_back(){
                        None => break,
                        Some(request) => request
                    }
                    
                };

                let start = Instant::now();
                let mut user = TcpStream::connect(PEER_ADDRESS).unwrap();
                
                let http_get = format!("GET {} HTTP/1.1\r\n\r\n",request);
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
                
                total_time += end;
                total_size += file_size;
                
            }
            return (total_time,total_size);
        }))
    }
    
    let mut total_time = 0.0;
    let mut total_size = 0;
    threads.into_iter().for_each(|thread| {
        let (time,size) = thread.join().unwrap();
        total_time += time;
        total_size += size;
    });
    
    let throughput = total_size as f64 / total_time ;
    
    println!("Total time: {}",total_time);
    println!("Total throughput: {}",throughput);
    
    
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