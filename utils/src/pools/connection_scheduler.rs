use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;
use memmap2::Mmap;
use path_clean::PathClean;
use crate::constants::BYTE;
use crate::html_prefetch_service::HtmlPrefetchService;
use crate::packets::byte_packet::BytePacket;
use crate::packets::chunk_type::FilePacketType;
use crate::sctp::sctp_client::SctpStream;

pub struct ConnectionScheduler {

    num_workers: usize,
    packet_size: usize,
    receive_buffer_size: usize,
    stream: Arc<SctpStream>,
    ppid_counter: u32,
    worker_threads: Vec<SchedulerWorker>,

}

impl ConnectionScheduler {

    pub fn new(stream: SctpStream, num_workers: usize, packet_size: usize, receive_buffer_size: usize) -> Self {

        assert!(packet_size > 0);
        assert!(num_workers > 0);

        Self{
            num_workers,
            packet_size,
            receive_buffer_size,
            stream: Arc::new(stream),
            ppid_counter: 0,
            worker_threads: Vec::with_capacity(num_workers),
        }

    }

    /// Returns the current ppid. Increments it with intentional overflow behaviour.
    fn next_ppid(&mut self)-> u32{

        let ppid = self.ppid_counter;
        self.ppid_counter = self.ppid_counter.wrapping_add(1);
        ppid

    }

    pub fn start(mut self) {

        // Init a prefetch service and use it to process the html files
        let mut prefetch_service = HtmlPrefetchService::new();

        // The current working directory should be set to the server root
        let server_root =  PathBuf::from("./");
        prefetch_service.build_prefetch_links(server_root).expect("HTML prefetch service build error");
        let html_links = prefetch_service.get_links();

        // Prepare and create the workers
        let stream = Arc::clone(&self.stream);
        let (job_tx,job_rx) = mpsc::channel();

        let job_rx = Arc::new(Mutex::new(job_rx));

        for worker_stream in 0..self.num_workers {

            let stream = Arc::clone(&stream);
            let job_rx = Arc::clone(&job_rx);
            let worker = SchedulerWorker::new(worker_stream as u16,stream,self.packet_size,job_rx);
            self.worker_threads.push(worker);

        }

        let mut buffer = vec![0u8; self.receive_buffer_size];
        let empty_vec = Vec::new();

        loop{
            match stream.read(buffer.as_mut_slice(),None,None){

                Ok(0) => {
                    println!("Connection closed");
                    break;
                }

                Err(e) => {
                    panic!("Error reading from sctp stream: {}", e);
                }

                Ok(bytes_read) =>{

                    // Read the requested path and clean it
                    let path_request = String::from_utf8_lossy(&buffer[..bytes_read]);

                    let path = match path_request.trim() {
                        "/" => "./index.html".to_string(),
                        _ => {
                            // Remove query operator ? in path
                            String::from(".") + &path_request.trim_end_matches("?")
                        }
                    };

                    let path = PathBuf::from(path).clean();
                    let ppid = self.next_ppid();

                    // Get the dependencies if they exist and send the request to the workers to be processed
                    let dependencies = html_links.get(&path).unwrap_or(&empty_vec);

                    job_tx.send((path,ppid)).unwrap();

                    for prefetched_path in dependencies {
                        let ppid = self.next_ppid();
                        job_tx.send((prefetched_path.clone(),ppid)).unwrap();
                    }

                }

            }


        }


    }


}


const FILE_CHUNK_METADATA_SIZE: usize = 3 * BYTE;
const FILE_METADATA_PACKET_SIZE: usize = 11 * BYTE;

struct SchedulerWorker{
    stream_number: u16,
    thread: Option<JoinHandle<()>>,
}

impl SchedulerWorker {
    fn new(stream_number: u16,stream: Arc<SctpStream>,packet_size: usize, file_receiver: Arc<Mutex<Receiver<(PathBuf,u32)>>>) -> Self {

        let thread = thread::spawn(move || {
            loop{

                // Get a new job, or end the loop if the sender disconnected
                let file = file_receiver.lock().unwrap().recv();
                let (file_path,ppid) = match file{
                    Ok(file_data) => file_data,
                    Err(_) => break,
                };

                let file = OpenOptions::new()
                    .read(true)
                    .write(false)
                    .create(false)
                    .truncate(false)
                    .open(&file_path)
                    .expect(format!("Failed to open file {}",file_path.display()).as_str());

                let mmap = unsafe{
                    Mmap::map(&file).unwrap()
                };

                // Prepare the first packet metadata:
                // packet_type + chunk_count + file_size + client_side_file_path

                let client_side_file_path = PathBuf::from("/").join(&file_path);
                let file_path = client_side_file_path.to_string_lossy();
                let file_bytes = file_path.as_bytes();

                let packet_type = FilePacketType::Metadata;
                let file_size = mmap.len();
                let file_chunk_size = packet_size - FILE_CHUNK_METADATA_SIZE;
                let chunk_count = (file_size + file_chunk_size - 1) / file_chunk_size;

                let mut packet_buffer = BytePacket::new(FILE_METADATA_PACKET_SIZE + file_bytes.len());

                packet_buffer.write_u8(u8::from(packet_type)).unwrap();
                packet_buffer.write_u16(chunk_count as u16).unwrap();
                packet_buffer.write_u64(file_size as u64).unwrap();

                unsafe{
                    packet_buffer.write_buffer(file_bytes).unwrap();
                }

                println!("Sending metadata...");

                stream.write_all(packet_buffer.get_buffer(),stream_number,ppid,0).unwrap();

                // Prepare to send each file packet:
                // packet_type + chunk_index + file_chunk
                println!("Sending chunks...");
                for (chunk_index,chunk) in mmap.chunks(file_chunk_size).enumerate() {

                    let mut chunk_packet = BytePacket::new(packet_size);

                    // Get the packet type based on the chunk index
                    let packet_type = if chunk_index == chunk_count - 1{
                        FilePacketType::LastChunk
                    }else{
                        FilePacketType::Chunk
                    };

                    let chunk_index: u16 = chunk_index as u16;

                    chunk_packet.write_u8(u8::from(packet_type)).unwrap();
                    chunk_packet.write_u16(chunk_index).unwrap();
                    unsafe{
                        chunk_packet.write_buffer(chunk).unwrap();
                    }

                    stream.write_all(chunk_packet.get_buffer(),stream_number,ppid,0).unwrap();

                }

            }
        });

        Self{
            stream_number,
            thread: Some(thread),
        }

    }
}