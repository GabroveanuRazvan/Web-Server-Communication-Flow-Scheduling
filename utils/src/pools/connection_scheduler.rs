use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs::OpenOptions;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::JoinHandle;
use crate::constants::BYTE;
use crate::libc_wrappers::CStruct;
use crate::mapped_file::{MappedFile};
use crate::packets::byte_packet::BytePacket;
use crate::sctp::sctp_api::SctpSenderReceiveInfo;
use crate::sctp::sctp_client::SctpStream;


const PACKET_METADATA_SIZE: usize = 4 * BYTE;

/// Shortest Job First scheduler for a Sctp Stream.
///
pub struct ConnectionScheduler{

    heap: Arc<(Mutex<Option<BinaryHeap<Reverse<(MappedFile,u32)>>>>,Condvar)>,
    stream: Arc<SctpStream>,
    workers: Vec<ConnectionWorker>,
    packet_size: usize,
    buffer_size: usize,

}

impl ConnectionScheduler{

    /// Creates a worker pool of size and takes a Sctp Stream.
    ///
    pub fn new(size: usize, stream: SctpStream, buffer_size: usize, packet_size: usize) -> Self{
        assert!(size > 0);
        assert!(packet_size > PACKET_METADATA_SIZE);

        let mut workers = Vec::with_capacity(size);
        let stream = Arc::new(stream);
        let heap = Arc::new((Mutex::new(Some(BinaryHeap::new())), Condvar::new()));

        for i in 0..size{
            workers.push(ConnectionWorker::new(i, Arc::clone(&heap), Arc::clone(&stream), packet_size));
        }

        Self{
            heap,
            stream,
            workers,
            packet_size,
            buffer_size,
        }

    }

    /// Pushes on the scheduler min-heap a new MappedFile as a job.
    pub fn schedule_job(&self,job: (MappedFile,u32)){
        // get a reference to the heap and condition variable
        let (mutex,cvar) = &*self.heap;

        // acquire the heap and push the new Job
        let mut heap_guard = mutex.lock().unwrap();

        heap_guard.as_mut()
            .unwrap()
            .push(Reverse(job));

        // unlock the heap and notify one of the workers
        drop(heap_guard);
        cvar.notify_one();
    }

    /// Method that consumes and starts the scheduler.
    /// Reads the requests from the Sctp Stream, process them and schedule them.
    ///
    pub fn start(self){
        let mut buffer: Vec<u8> = vec![0;self.buffer_size];
        let mut sender_info: SctpSenderReceiveInfo = SctpSenderReceiveInfo::new();


        loop {

            let bytes_read = self.stream.read(&mut buffer, Some(&mut sender_info), None).unwrap();
            let ppid = sender_info.sinfo_ppid as u32;

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
                println!("Not exists: {}",path);
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(false)
                    .truncate(false)
                    .open("./404.html").unwrap()
            }

            );

            let mapped_file = MappedFile::new(file).unwrap();

            self.schedule_job((mapped_file,ppid));

        }
    }

}

/// Drop trait used to gracefully shut down all worker threads.
///
impl Drop for ConnectionScheduler
{
    fn drop(&mut self){

        // acquire the mutex
        let (mutex,cvar) = &*self.heap;
        let mut heap_guard = mutex.lock().unwrap();

        // take the heap out and notify all the threads
        heap_guard.take();

        cvar.notify_all();

        // drop the guard
        drop(heap_guard);

        // join all worker threads
        for worker in &mut self.workers{

            let handle = worker.thread.take().unwrap();

            handle.join().expect("Failed to join worker thread");

        }

    }
}

/// Worker for the ConnectionScheduler.
///
pub struct ConnectionWorker{
    label: usize,
    thread: Option<JoinHandle<()>>,
}

impl ConnectionWorker{
    /// Starts the worker thread.
    ///
    pub fn new(label: usize, heap: Arc<(Mutex<Option<BinaryHeap<Reverse<(MappedFile,u32)>>>>,Condvar)>, stream: Arc<SctpStream>, packet_size: usize) -> Self{


        // 4 bytes coming from the leading chunk index + total chunks
        let chunk_size = packet_size - PACKET_METADATA_SIZE;

        let thread = thread::Builder::new()
            .name(format!("Connection Worker {}", label))
            .spawn(move || {

                // Get a reference to the mutex and cond var
                let (mutex,cvar) = &*heap;
                let stream_number = label as u16;

                loop{

                    // Acquire the mutex
                    let mut heap_guard = mutex.lock().unwrap();

                    // Shut down case 1: after the pool drop is called the mutex is released and a thread might try to get a new job
                    // if the thread gets this mutex in the first instance we need to check if the heap still exits
                    if heap_guard.is_none(){
                        break;
                    }

                    // While the heap exists and is empty wait
                    while let Some(heap) = heap_guard.as_mut(){

                        // if the heap is not empty then there is a new job to be processed
                        if !heap.is_empty(){
                            break;
                        }

                        heap_guard = cvar.wait(heap_guard).unwrap();
                    }

                    // Shut down case 2: while the worker was waiting for a new job, he gets notified by the pool drop and ends the while loop as the heap is None now
                    // So, check if the heap is none to know when to stop
                    if heap_guard.is_none(){
                        break;
                    }

                    // When the heap is not empty extract the job release the mutex and execute the job

                    println!("Worker thread labeled {label} got a new job.");
                    if let Some(Reverse(job_pair)) = heap_guard.as_mut().and_then(|heap| heap.pop()){
                        drop(heap_guard);

                        let (job,ppid) = job_pair;
                        let file_size = job.mmap_as_slice().len();

                        // Ceil formula for integers
                        let chunk_count = (file_size + chunk_size - 1) / chunk_size;

                        // Iterate through each chunk and send the packets
                        for (chunk_index,chunk) in job.mmap_as_slice().chunks(chunk_size).enumerate(){

                            // Build the file chunk packet consisting of: current chunk index + total chunk count + chunk size + chunk data
                            let mut chunk_packet = if chunk_index != chunk_count - 1 {
                                BytePacket::new(packet_size)

                            }
                            else{
                                BytePacket::new(chunk.len() + PACKET_METADATA_SIZE)
                            };

                            chunk_packet.write_u16(chunk_index as u16).unwrap();
                            chunk_packet.write_u16(chunk_count as u16).unwrap();

                            unsafe{
                                chunk_packet.write_buffer(chunk).unwrap();
                            }

                            // Send the chunk
                            match stream.write_all(chunk_packet.get_buffer(),stream_number,ppid,chunk_index as u32){
                                Ok(_bytes) => (),
                                Err(e) => println!("Write Error: {:?}",e)
                            }

                        }

                    }

                }

                println!("Worker thread labeled {label} shutting down.")
            }).unwrap();

        Self{
            label,
            thread: Some(thread),
        }

    }
}