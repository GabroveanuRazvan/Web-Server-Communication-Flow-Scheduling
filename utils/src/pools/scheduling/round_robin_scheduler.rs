use std::fs::OpenOptions;
use std::sync::Arc;
use memmap2::Mmap;
use crate::pools::indexed_thread_pool::IndexedThreadPool;
use crate::sctp::sctp_client::SctpStream;
use crate::constants::PACKET_METADATA_SIZE;
use crate::libc_wrappers::CStruct;
use crate::packets::byte_packet::BytePacket;
use crate::sctp::sctp_api::SctpSenderReceiveInfo;

/// Round Robin scheduler for a Sctp Stream.
pub struct RoundRobinScheduler {

    stream: Arc<SctpStream>,
    packet_size: usize,
    buffer_size: usize,
    num_workers: usize,
    worker_pool: IndexedThreadPool,
    round_robin_counter: usize,
}

impl RoundRobinScheduler {

    /// Creates a worker pool of given size and takes a Sctp Stream.
    pub fn new(num_workers: usize, stream: SctpStream, buffer_size: usize, packet_size: usize) -> Self{
        assert!(packet_size > PACKET_METADATA_SIZE);

        let worker_pool = IndexedThreadPool::new(num_workers);

        Self{
            stream: Arc::new(stream),
            packet_size,
            buffer_size,
            num_workers,
            worker_pool,
            round_robin_counter: 0,
        }

    }

    /// Pushes on the scheduler min-heap a new MappedFile as a job.
    pub fn schedule_job(&mut self, job: (Mmap,u32)){

        let job_index = self.round_robin_counter;
        self.round_robin_counter = (self.round_robin_counter + 1) % self.num_workers;

        // 4 bytes coming from the leading chunk index + total chunks
        let chunk_size = self.packet_size - PACKET_METADATA_SIZE;
        let packet_size = self.packet_size;
        let stream = Arc::clone(&self.stream);

        self.worker_pool.execute(job_index, move || {
            let (file_buffer,ppid) = job;

            let file_size = file_buffer.len();

            // Ceil formula for integers
            let chunk_count = (file_size + chunk_size - 1) / chunk_size;

            // Iterate through each chunk and send the packets
            for (chunk_index,chunk) in file_buffer.chunks(chunk_size).enumerate() {

                // Build the file chunk packet consisting of: current chunk index + total chunk count + chunk size + chunk data
                let mut chunk_packet = if chunk_index != chunk_count - 1 {
                    BytePacket::new(packet_size)
                } else {
                    BytePacket::new(chunk.len() + PACKET_METADATA_SIZE)
                };

                chunk_packet.write_u16(chunk_index as u16).unwrap();
                chunk_packet.write_u16(chunk_count as u16).unwrap();

                unsafe {
                    chunk_packet.write_buffer(chunk).unwrap();
                }

                // Send the chunk
                match stream.write_all(chunk_packet.get_buffer(), job_index as u16, ppid, chunk_index as u32) {
                    Ok(_bytes) => (),
                    Err(e) => println!("Write Error: {:?}", e)
                }
            }

        });

    }

    /// Starts and consumes the scheduler.
    /// Each request will be assigned to a worker by Round Robin scheduling.
    pub fn start(mut self){

        let mut buffer = vec![0u8; self.buffer_size];
        let mut sender_info = SctpSenderReceiveInfo::new();

        loop{

            let bytes_read = self.stream.read(&mut buffer,Some(&mut sender_info),None).unwrap();
            let ppid = sender_info.sinfo_ppid;

            if bytes_read == 0 {
                break;
            }

            let path_request = String::from_utf8_lossy(&buffer[..bytes_read]);

            let path = match path_request.trim() {
                "/" => "./index.html".to_string(),
                _ => {
                    // Remove query operator ? in path
                    String::from(".") + &path_request.trim_end_matches("?")
                }
            };

            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(false)
                .truncate(false)
                .open(&path);

            let file = file.unwrap_or_else(|_|{
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(false)
                    .truncate(false)
                    .open("./404.html").unwrap()
            });

            let mmap = unsafe{Mmap::map(&file).unwrap()};

            self.schedule_job((mmap,ppid));

        }

    }

}