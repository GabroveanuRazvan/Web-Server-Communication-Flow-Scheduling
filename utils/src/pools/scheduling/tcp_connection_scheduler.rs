use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs::OpenOptions;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::JoinHandle;
use crate::libc_wrappers::CStruct;
use crate::mapped_file::{MappedFile};
use crate::packets::byte_packet::BytePacket;
use crate::constants::{CHUNK_METADATA_SIZE, METADATA_STATIC_SIZE};
use crate::tcp::tcp_association::TcpAssociation;

pub struct TcpConnectionScheduler {

    heap: Arc<(Mutex<Option<BinaryHeap<Reverse<(MappedFile,String,u32)>>>>,Condvar)>,
    assoc: TcpAssociation,
    workers: Vec<ConnectionWorker>,
    
}

impl TcpConnectionScheduler {

    
    pub fn new(num_workers: usize, assoc: TcpAssociation, packet_size: usize) -> Self{
        assert!(num_workers > 0);
        assert!(packet_size > CHUNK_METADATA_SIZE);

        let mut workers = Vec::with_capacity(num_workers);
        let heap = Arc::new((Mutex::new(Some(BinaryHeap::new())), Condvar::new()));

        for i in 0..num_workers {
            workers.push(ConnectionWorker::new(i, Arc::clone(&heap),assoc.try_clone().unwrap(), packet_size));
        }

        Self{
            heap,
            assoc,
            workers,
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

    
    pub fn start(mut self){
        
        loop {
            
            let mut message_info  = match self.assoc.receive(){
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Ok(message_info) => message_info,
                Err(e) => panic!("{}", e),
            };
            
            
            let path_request = String::from_utf8_lossy(message_info.message.as_slice());

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
            
            
            let mapped_file = MappedFile::new(file).unwrap();
            
            self.schedule_job((mapped_file,path,message_info.ppid));

        }
    }

}

impl Drop for TcpConnectionScheduler
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
    pub fn new(label: usize, heap: Arc<(Mutex<Option<BinaryHeap<Reverse<(MappedFile,String,u32)>>>>,Condvar)>, mut assoc: TcpAssociation, packet_size: usize) -> Self{

        let chunk_size = packet_size - CHUNK_METADATA_SIZE;

        let thread = thread::Builder::new()
            .name(format!("Conn_Th_{}", label))
            .spawn(move || {

                // Get a reference to the mutex and cond var
                let (mutex,cvar) = &*heap;
                let stream_number = label;

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
                        assoc.send(metadata_packet.get_buffer(),stream_number,ppid).unwrap();


                        // Iterate through each chunk and send the packets
                        for chunk in file_buffer.mmap_as_slice().chunks(chunk_size){
                            
                            if let Err(e) = assoc.send(chunk,stream_number,ppid){
                                eprintln!("Send Error: {:?}",e)
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