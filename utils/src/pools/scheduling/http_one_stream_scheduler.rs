use std::fs::OpenOptions;
use memmap2::Mmap;
use crate::http_parsers::{basic_http_response, extract_uri, http_response_to_string};
use crate::libc_wrappers::CStruct;
use crate::sctp::sctp_api::SctpSenderReceiveInfo;
use crate::sctp::sctp_client::SctpStream;

pub struct HttpOneStreamScheduler{

    stream: SctpStream,
    buffer_size: usize,
    packet_size: usize,
}

impl HttpOneStreamScheduler {
    pub fn new(stream: SctpStream, buffer_size: usize, packet_size: usize) -> Self {
        Self {
            stream,
            buffer_size,
            packet_size,
        }
    }
    
    pub fn start(mut self) {
        
        let mut buffer = vec![0u8; self.buffer_size];
        let mut sender_info = SctpSenderReceiveInfo::new();
        
        loop {
            
            match self.stream.read(&mut buffer,Some(&mut sender_info),None){
                
                Err(_error) => break,
                
                Ok(0) => {
                    println!("SCTP connection closed");
                    break;
                }
                
                Ok(bytes_received) => {

                    // TODO better parsing
                    // Extract the first line of the request
                    let new_line_position = buffer.iter().position(|&b| b == b'\n').unwrap();
                    let request_line = String::from_utf8_lossy(&buffer[..new_line_position]).to_string();

                    // Get the server-side file name, the cache side file name and path
                    let file_name = extract_uri(request_line).unwrap();

                    let file_path = match file_name.trim() {
                        "/" => "./index.html".to_string(),
                        _ => {
                            // Remove query operator ? in path
                            String::from(".") + &file_name.trim_end_matches("?").to_string()
                        }
                    };

                    let file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(false)
                        .truncate(false)
                        .open(&file_path);
                    
                    let file = match file{
                       Ok(file) => file,
                       Err(_) => {
                           println!("Could not open {file_path}");
                           break;
                       },
                    };
                    
                    
                    let mmap = unsafe{Mmap::map(&file)};
                    let mmap = match mmap{
                        Ok(mmap) => mmap,
                        Err(_) => {
                            println!("Could not map the file {file_path}");
                            break;
                        }
                        
                    };

                    let http_response = basic_http_response(mmap.len());
                    let string_response = http_response_to_string(http_response);
                    
                    if let Err(error) = self.stream.write_all(string_response.as_bytes(),0,0,0){
                        println!("Error sending response of {file_path}: {error}");
                        break
                    }
                    
                    for chunk in mmap.chunks(self.packet_size){
                        if let Err(error) = self.stream.write_all(&chunk,0,0,0){
                            println!("Error while sending file {file_path}: {error}");
                            break
                        }
                    }
                    
                }
                
            }
            
        }
        
    }
    
}