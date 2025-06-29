use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;
use crate::libc_wrappers::CStruct;
use crate::mapped_file::{MappedFile};
use crate::packets::byte_packet::BytePacket;
use crate::sctp::sctp_api::SctpSenderReceiveInfo;
use crate::sctp::sctp_client::SctpStream;
use crate::constants::{CHUNK_METADATA_SIZE, METADATA_STATIC_SIZE};
use crate::logger::Logger;

/// Shortest Job First scheduler for a Sctp Stream.
pub struct ConnectionScheduler{

    heap: Arc<(Mutex<Option<BinaryHeap<Reverse<(MappedFile,String,u32)>>>>,Condvar)>,
    stream: Arc<SctpStream>,
    workers: Vec<ConnectionWorker>,
    buffer_size: usize,

}

impl ConnectionScheduler{

    /// Creates a worker pool of given size and takes a Sctp Stream.
    pub fn new(num_workers: usize, stream: SctpStream, buffer_size: usize, packet_size: usize) -> Self{
        assert!(num_workers > 0);
        assert!(packet_size > CHUNK_METADATA_SIZE);

        let mut workers = Vec::with_capacity(num_workers);
        let stream = Arc::new(stream);
        let heap = Arc::new((Mutex::new(Some(BinaryHeap::new())), Condvar::new()));

        for i in 0..num_workers {
            workers.push(ConnectionWorker::new(i, Arc::clone(&heap), Arc::clone(&stream), packet_size));
        }

        Self{
            heap,
            stream,
            workers,
            buffer_size,
        }

    }

    /// Pushes on the scheduler's min-heap a new MappedFile as a job.
    pub fn schedule_job(&self,job: (MappedFile,String,u32)){
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

    /// Starts and consumes the scheduler.
    /// Each request will be assigned to a worker by SJF scheduling.
    pub fn start(self){
        let mut buffer = vec![0u8;self.buffer_size];
        let mut sender_info: SctpSenderReceiveInfo = SctpSenderReceiveInfo::new();
        
        // let mut logger = Logger::new("/tmp/Breakdown/MappedFiles").unwrap();
        
        loop {

            let bytes_read = self.stream.read(&mut buffer, Some(&mut sender_info), None).unwrap();

            // Connection closed
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
            
            // let start = Instant::now();
            
            let mapped_file = MappedFile::new(file).unwrap();
            
            // let end = start.elapsed().as_micros();
            // logger.writeln(format!("Map file: {end} us").as_ref()).unwrap();
            
            self.schedule_job((mapped_file,path,sender_info.sinfo_ppid));

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
pub struct ConnectionWorker{
    label: usize,
    thread: Option<JoinHandle<()>>,
}

impl ConnectionWorker{

    /// Starts the worker thread.
    pub fn new(label: usize, heap: Arc<(Mutex<Option<BinaryHeap<Reverse<(MappedFile,String,u32)>>>>,Condvar)>, stream: Arc<SctpStream>, packet_size: usize) -> Self{

        let chunk_size = packet_size - CHUNK_METADATA_SIZE;

        let thread = thread::Builder::new()
            .name(format!("Conn_Th_{}", label))
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

                    // println!("Worker thread labeled {label} got a new job.");
                    if let Some(Reverse(job)) = heap_guard.as_mut().and_then(|heap| heap.pop()){
                        drop(heap_guard);

                        let (file_buffer,path,ppid) = job;
                        let path_bytes = &path.as_bytes()[1..];
                        let file_size = file_buffer.mmap_as_slice().len();


                        // Send a metadata packet made out of packet file_size + file_path
                        let mut metadata_packet = BytePacket::new(METADATA_STATIC_SIZE + path_bytes.len());
                        metadata_packet.write_u64(file_size as u64).unwrap();
                        unsafe{metadata_packet.write_buffer(&path_bytes).unwrap();}
                        stream.write_all(metadata_packet.get_buffer(),stream_number,ppid,0).unwrap();


                        // Iterate through each chunk and send the packets
                        for chunk in file_buffer.mmap_as_slice().chunks(chunk_size){

                            // Just send the raw chunk
                            match stream.write_all(chunk,stream_number,ppid,0){
                                Ok(_bytes) => (),
                                Err(e) => eprintln!("Write Error: {:?}",e)
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


/// Worker for the ConnectionScheduler.
pub struct BenchmarkConnectionWorker{
    label: usize,
    thread: Option<JoinHandle<()>>,
}

impl BenchmarkConnectionWorker{
    /// Starts the worker thread.
    pub fn new(label: usize, heap: Arc<(Mutex<Option<BinaryHeap<Reverse<(MappedFile,String,u32)>>>>,Condvar)>, stream: Arc<SctpStream>, packet_size: usize) -> Self{

        let chunk_size = packet_size - CHUNK_METADATA_SIZE;
        let thread_name = format!("Conn_Th_{}", label);
        let logger_file_path = PathBuf::from("/tmp/Breakdown").join(&thread_name);

        let thread = thread::Builder::new()
            .name(thread_name)
            .spawn(move || {

                // Get a reference to the mutex and cond var
                let (mutex,cvar) = &*heap;
                let stream_number = label as u16;

                let mut logger = Logger::new(logger_file_path.as_path()).expect("Failed to create logger");

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

                    // println!("Worker thread labeled {label} got a new job.");
                    if let Some(Reverse(job)) = heap_guard.as_mut().and_then(|heap| heap.pop()){
                        drop(heap_guard);
                        
                        let (file_buffer,path,ppid) = job;
                        let path_bytes = &path.as_bytes()[1..];
                        let file_size = file_buffer.mmap_as_slice().len();

                        
                        // Track metadata packet build time
                        let start_metadata_packet = Instant::now();

                        // Send a metadata packet made out of packet file_size + file_path
                        let mut metadata_packet = BytePacket::new(METADATA_STATIC_SIZE + path_bytes.len());
                        metadata_packet.write_u64(file_size as u64).unwrap();
                        unsafe{metadata_packet.write_buffer(&path_bytes).unwrap();}

                        let end_metadata_packet = start_metadata_packet.elapsed().as_micros();
                        logger.writeln(format!("(1) Metadata packet build time: {} us",end_metadata_packet).as_ref()).unwrap();

                        // Track sending the metadata packet time
                        let start_send_metadata = Instant::now();

                        stream.write_all(metadata_packet.get_buffer(),stream_number,ppid,0).unwrap();

                        let end_send_metadata_packet = start_send_metadata.elapsed().as_micros();
                        logger.writeln(format!("(2) Send metadata packet: {} us",end_send_metadata_packet).as_ref()).unwrap();


                        // Track sending the requested file
                        let mut send_chunks_time : u128 = 0;

                        // Iterate through each chunk and send the packets
                        for chunk in file_buffer.mmap_as_slice().chunks(chunk_size){

                            let start_send_chunk = Instant::now();

                            // Just send the raw chunk
                            match stream.write_all(chunk,stream_number,ppid,0){
                                Ok(_bytes) => (),
                                Err(e) => eprintln!("Write Error: {:?}",e)
                            }

                            send_chunks_time += start_send_chunk.elapsed().as_millis();

                        }

                        logger.writeln(format!("(3) Send complete file: {} ms",send_chunks_time).as_ref()).unwrap();

                        let chunk_count = file_buffer.mmap_as_slice().len() as f64;
                        let send_chunk_average_time = send_chunks_time as f64 / chunk_count;
                        logger.writeln(format!("(4) Average chunk sending time: {:.3} ms of {} chunks\n",send_chunk_average_time,chunk_count).as_ref()).unwrap()

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