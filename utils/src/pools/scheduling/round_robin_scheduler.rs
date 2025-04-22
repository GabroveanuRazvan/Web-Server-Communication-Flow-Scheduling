use std::fs::OpenOptions;
use std::sync::Arc;
use memmap2::Mmap;
use crate::pools::indexed_thread_pool::IndexedThreadPool;
use crate::sctp::sctp_client::SctpStream;
use crate::constants::{CHUNK_METADATA_SIZE, METADATA_STATIC_SIZE};
use crate::libc_wrappers::CStruct;
use crate::packets::byte_packet::BytePacket;
use crate::packets::chunk_type::FilePacketType;
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
        assert!(packet_size > CHUNK_METADATA_SIZE);

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
    pub fn schedule_job(&mut self, job: (Mmap,String,u32)){

        let job_index = self.round_robin_counter;
        self.round_robin_counter = (self.round_robin_counter + 1) % self.num_workers;

        // Get owned metadata variables
        let chunk_size = self.packet_size - CHUNK_METADATA_SIZE;
        let stream = Arc::clone(&self.stream);

        self.worker_pool.execute(job_index, move || {

            let (file_buffer,path,ppid) = job;
            let path_bytes = &path.as_bytes()[1..];
            let file_size = file_buffer.len();
            let stream_number = job_index as u16;



            // Send a metadata packet made out of file_size + file_path
            let mut metadata_packet = BytePacket::new(METADATA_STATIC_SIZE + path_bytes.len());
            metadata_packet.write_u64(file_size as u64).unwrap();
            unsafe{metadata_packet.write_buffer(&path_bytes).unwrap();}

            stream.write_all(metadata_packet.get_buffer(),stream_number,ppid,0).unwrap();


            // Iterate through each chunk and send the packets
            for chunk in file_buffer.chunks(chunk_size){

                // Just send the raw chunk
                match stream.write_all(chunk,stream_number,ppid,0){
                    Ok(_bytes) => (),
                    Err(e) => eprintln!("Write Error: {:?}",e)
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

            self.schedule_job((mmap,path,sender_info.sinfo_ppid));

        }

    }

}