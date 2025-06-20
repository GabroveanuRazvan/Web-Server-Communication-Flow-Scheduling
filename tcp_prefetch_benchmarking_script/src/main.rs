use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant};
use utils::config::serialization::load;
use utils::constants::{KILOBYTE};

const PEER_ADDRESS: &str = "192.168.50.30:7878";
const RECEIVE_BUFFER_SIZE: usize = 16 * KILOBYTE;
const USER_COUNT: usize = 4;
const DATASET_ROOT : &str = "../benchmarking/raw_dataset";
const REQUESTS_PATH_PREFIX: &str = "../benchmarking/requests/prefetch_requests_";
const REQUESTS_PATH_SUFFIX: &str = "_5000.json";
const PERSISTENT_CONNECTIONS: bool = false;
const RUNS_COUNT: usize = 6;

fn main() {

    let mut results = Vec::with_capacity(RUNS_COUNT);
    
    (0..RUNS_COUNT).into_iter().for_each(|idx| {
    
        let requests_path = format!("{REQUESTS_PATH_PREFIX}{idx}{REQUESTS_PATH_SUFFIX}");

        let requests: VecDeque<String> = load(requests_path).unwrap();
        let requests = Arc::new(Mutex::new(requests));
        let mut threads = Vec::with_capacity(USER_COUNT);
        let mut total_size = 0;
        
        let start_global = Instant::now();
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
            total_size += size;
            avg_throughput += throughput;
            avg_time += time;
        });
        
        let global_time = start_global.elapsed().as_secs_f64();

        let global_throughput = total_size as f64 / global_time;
        avg_throughput /=  USER_COUNT as f64;
        avg_time /= USER_COUNT as f64;

        println!("Avg time: {avg_time}");
        println!("Avg throughput: {avg_throughput}");
        println!("Global time: {global_time}");
        println!("Global throughput: {global_throughput}");
        
        
        results.push((avg_time, avg_throughput, global_time, global_throughput));
        
    });
    
    println!("{:#?}",results);
    
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

// Persistent
// [
// (
// 298.3819118460002,
// 4648049.430245857,
// 298.719828951,
// 18571104.199815217,
// ),
// (
// 295.9556842622501,
// 4446720.086095891,
// 296.242169253,
// 17769692.796518337,
// ),
// (
// 314.91334875125,
// 4438136.16539985,
// 315.030167076,
// 17745924.464596782,
// ),
// (
// 307.9941888322502,
// 4469149.986441469,
// 308.363884404,
// 17855401.5936134,
// ),
// (
// 285.6976099107501,
// 4763635.911396688,
// 285.815075213,
// 19046803.993606813,
// ),
// (
// 278.5942469975002,
// 4908462.025675666,
// 278.774878705,
// 19621362.25261639,
// ),
// ]

// Persistent

// [
// (
// 244.8020736767501,
// 5665338.357654659,
// 244.920799509,
// 22650412.21946585,
// ),
// (
// 231.21487256300009,
// 5691881.821786297,
// 231.333588218,
// 22755590.23464972,
// ),
// (
// 241.12248840824995,
// 5796335.529452373,
// 241.157942095,
// 23181909.334745105,
// ),
// (
// 239.07315806025008,
// 5757662.864827857,
// 239.20760136,
// 23017500.11996358,
// ),
// (
// 236.50477636400004,
// 5754496.342212795,
// 236.513711703,
// 23017116.75319731,
// ),
// (
// 236.5768387927498,
// 5780278.873819073,
// 236.627469721,
// 23116263.248934865,
// ),
// ]

//non persistent

// [
// (
// 264.28994013474994,
// 5247535.339842389,
// 264.595500677,
// 20966180.663714595,
// ),
// (
// 252.13169988650003,
// 5219567.085878214,
// 252.388758639,
// 20857237.736683283,
// ),
// (
// 266.8688669475,
// 5237114.6164556835,
// 266.962819411,
// 20941124.166033015,
// ),
// (
// 260.426994681,
// 5285518.386598385,
// 260.72254579,
// 21118085.420333374,
// ),
// (
// 262.0456017490002,
// 5193659.518134609,
// 262.177549126,
// 20764034.65570476,
// ),
// (
// 264.28434277125,
// 5174379.8681954425,
// 264.458774716,
// 20683537.114146143,
// ),
// ]

