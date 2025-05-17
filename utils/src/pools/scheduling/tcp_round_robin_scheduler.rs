use std::fs::OpenOptions;
use memmap2::Mmap;
use crate::pools::indexed_thread_pool::IndexedThreadPool;
use crate::constants::{CHUNK_METADATA_SIZE, METADATA_STATIC_SIZE};
use crate::packets::byte_packet::BytePacket;
use crate::tcp::tcp_association::TcpAssociation;

/// Round Robin scheduler for a Sctp Stream.
pub struct TcpRoundRobinScheduler {

    assoc: TcpAssociation,
    packet_size: usize,
    worker_count: u8,
    worker_pool: IndexedThreadPool,
    round_robin_counter: u8,
}

impl TcpRoundRobinScheduler {
    
    /// Create a new thread pool of the size of the stream count of the provided association.
    pub fn new(assoc: TcpAssociation, packet_size: usize) -> Self{
        assert!(packet_size > CHUNK_METADATA_SIZE);
        let worker_count = assoc.stream_count();
        
        let worker_pool = IndexedThreadPool::new(worker_count as usize);

        Self{
            assoc,
            worker_count,
            packet_size,
            worker_pool,
            round_robin_counter: 0,
        }

    }

    /// Pushes on the scheduler min-heap a new MappedFile as a job.
    pub fn schedule_job(&mut self, job: (Mmap,String,u32)){

        let job_index = self.round_robin_counter;
        self.round_robin_counter = (self.round_robin_counter + 1) % self.worker_count;

        // Get owned metadata variables
        let chunk_size = self.packet_size - CHUNK_METADATA_SIZE;
        let mut assoc = self.assoc.try_clone().unwrap();

        self.worker_pool.execute(job_index as usize, move || {

            let (file_buffer,path,ppid) = job;
            let path_bytes = &path.as_bytes()[1..];
            let file_size = file_buffer.len();
            let stream_number = job_index as usize;

            
            // Send a metadata packet made out of file_size + file_path
            let mut metadata_packet = BytePacket::new(METADATA_STATIC_SIZE + path_bytes.len());
            metadata_packet.write_u64(file_size as u64).unwrap();
            unsafe{metadata_packet.write_buffer(&path_bytes).unwrap();}

            assoc.send(metadata_packet.get_buffer(),stream_number,ppid).unwrap();


            // Iterate through each chunk and send the packets
            for chunk in file_buffer.chunks(chunk_size){

                // Just send the raw chunk
                if let Err(e) = assoc.send(chunk,stream_number,ppid){
                    eprintln!("Write Error: {:?}",e) 
                }

            }


        });

    }

    /// Starts and consumes the scheduler.
    /// Each request will be assigned to a worker by Round Robin scheduling.
    pub fn start(mut self){
        
        loop{

            let message_info = match self.assoc.receive(){
                Ok(message_info) => message_info,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => panic!("{:?}", e),
            };
            
            
            let path_request = String::from_utf8_lossy(&message_info.message);

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

            self.schedule_job((mmap,path,message_info.ppid));

        }

    }

}