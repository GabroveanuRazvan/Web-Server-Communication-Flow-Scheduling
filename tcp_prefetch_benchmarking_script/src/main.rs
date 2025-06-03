use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant};
use utils::config::serialization::load;
use utils::constants::{KILOBYTE};
use utils::http_parsers::parse_root;

const PEER_ADDRESS: &str = "192.168.50.30:7878";
const RECEIVE_BUFFER_SIZE: usize = 16 * KILOBYTE;
const USER_COUNT: usize = 6;
const DATASET_ROOT : &str = "../benchmarking/raw_dataset";
const REQUESTS_PATH: &str = "../benchmarking/requests/prefetch_requests.json";

const PERSISTENT_CONNECTIONS: bool = false;

fn main() {
    
    
    let requests: VecDeque<String> = load(REQUESTS_PATH).unwrap();
    let requests = Arc::new(Mutex::new(requests));
    let mut threads = Vec::with_capacity(USER_COUNT);
    
    for _ in 0..USER_COUNT{
        let requests = Arc::clone(&requests);
        threads.push(thread::spawn(move || {
            
            if PERSISTENT_CONNECTIONS{
                persistent_connections(requests)
            }
            else{
                non_persistent_connections(requests)
            }
            
        }));
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

fn non_persistent_connections(requests: Arc<Mutex<VecDeque<String>>>) -> (f64,usize,f64){
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

    let throughput = total_size as f64 / total_time ;

    (total_time,total_size,throughput)
}

fn persistent_connections(requests: Arc<Mutex<VecDeque<String>>>) -> (f64,usize,f64){
    let mut buffer = [0u8;RECEIVE_BUFFER_SIZE];

    let mut total_size = 0;
    let mut total_time = 0.0;

    let mut user = TcpStream::connect(PEER_ADDRESS).unwrap();
    
    loop{

        let request = {

            let mut guard = requests.lock().unwrap();
            match guard.pop_back(){
                None => break,
                Some(request) => request
            }

        };

        let start = Instant::now();
        
        let http_get = format!("GET {} HTTP/1.1\r\n\r\n",request);
        user.write(http_get.as_bytes()).unwrap();

        let (response,residue) = get_http_header(&mut user);

        let file_size = extract_content_length(&response).unwrap();
        let mut current_length = residue.len();

        while current_length < file_size {
            let bytes_received = user.read(&mut buffer).unwrap();
            current_length += bytes_received;

        }
        
        let end = start.elapsed().as_secs_f64();

        total_time += end;
        total_size += file_size;

    }

    let throughput = total_size as f64 / total_time ;
    
    (total_time,total_size,throughput)
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