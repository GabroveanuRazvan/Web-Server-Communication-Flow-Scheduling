use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs::OpenOptions;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::JoinHandle;
use crate::http_parsers::{basic_http_response, http_response_to_string, string_to_http_request};
use crate::libc_wrappers::{debug_sctp_sndrcvinfo, new_sctp_sndrinfo, SctpSenderInfo};
use crate::mapped_file::{MappedFile};
use crate::sctp_client::SctpStream;

/// Shortest Job First scheduler for a Sctp Stream.
///
pub struct ConnectionScheduler{

    heap: Arc<(Mutex<Option<BinaryHeap<Reverse<MappedFile>>>>,Condvar)>,
    stream: Arc<SctpStream>,
    workers: Vec<ConnectionWorker>,
    chunk_size: usize,
    buffer_size: usize,

}

impl ConnectionScheduler{

    /// Creates a worker pool of size and takes a Sctp Stream.
    ///
    pub fn new(size: usize, stream: SctpStream,buffer_size: usize,chunk_size: usize)-> Self{
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let stream = Arc::new(stream);
        let heap = Arc::new((Mutex::new(Some(BinaryHeap::new())), Condvar::new()));

        for i in 0..size{
            workers.push(ConnectionWorker::new(i,Arc::clone(&heap),Arc::clone(&stream),chunk_size));
        }

        Self{
            heap,
            stream,
            workers,
            chunk_size,
            buffer_size,
        }

    }

    /// Pushes on the scheduler min-heap a new MappedFile as a job.
    pub fn schedule_job(&self,job: MappedFile){
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
        let mut sender_info: SctpSenderInfo = new_sctp_sndrinfo();


        loop {

            let bytes_read = self.stream.read(&mut buffer, Some(&mut sender_info), None).unwrap();

            if bytes_read == 0 {
                break;
            }

            println!("Read {bytes_read} bytes");

            debug_sctp_sndrcvinfo(&sender_info);

            let request = string_to_http_request(&String::from_utf8(buffer.clone()).unwrap());

            println!("{} {}", request.method().to_string(), request.uri().to_string());

            let mut method = request.method().to_string();
            let mut path = request.uri().path().to_string();

            if method == "GET" {
                path = match path.as_str() {
                    "/" => "./index.html".to_string(),
                    _ => {
                        String::from(".") + &path
                    }
                }
            } else {
                path = "./404.html".to_string();
            }

            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(false)
                .truncate(false)
                .open(path);

            let file = file.unwrap_or_else(|_|
                OpenOptions::new()
                .read(true)
                .write(true)
                .create(false)
                .truncate(false)
                .open("./404.html").unwrap()
            );

            let mapped_file = MappedFile::new(file).unwrap();

            self.schedule_job(mapped_file);

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
    pub fn new(label: usize,heap: Arc<(Mutex<Option<BinaryHeap<Reverse<MappedFile>>>>,Condvar)>, stream: Arc<SctpStream>,chunk_size: usize) -> Self{

        let thread = thread::spawn(move||{

            // get a reference to the mutex and cond var
            let (mutex,cvar) = &*heap;

            loop{

                // acquire the mutex
                let mut heap_guard = mutex.lock().unwrap();

                // Shut down case 1: after the pool drop is called the mutex is released and a thread might try to get a new job
                // if the thread gets this mutex in the first instance we need to check if the heap still exits
                if heap_guard.is_none(){
                    break;
                }

                // while the heap exists and is empty wait
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

                // when the heap is not empty extract the job release the mutex and execute the job

                println!("Worker thread labeled {label} got a new job.");
                if let Some(Reverse(job)) = heap_guard.as_mut().and_then(|heap| heap.pop()){
                    drop(heap_guard);

                    // create the response and send it
                    let mut response_bytes = http_response_to_string(basic_http_response(job.file_size())).into_bytes();
                    let stream_number = label as u16;

                    match stream.write_all(&mut response_bytes,stream_number,0){
                        Ok(bytes) => println!("Wrote {bytes}"),
                        Err(e) => println!("Write Error: {:?}",e)
                    }

                    // send the body of the response
                    match stream.write_chunked(&job.mmap_as_slice(),chunk_size,stream_number,0){
                        Ok(bytes) => println!("Wrote {bytes}"),
                        Err(e) => println!("Write Error: {:?}",e)
                    }

                    // send a null character to mark the end of the message
                    match stream.write_null(stream_number,0){
                        Ok(bytes) => println!("Wrote {bytes}"),
                        Err(e) => println!("Write Error: {:?}",e)
                    }

                }

            }

            println!("Worker thread labeled {label} shutting down.")

        });

        Self{
            label,
            thread: Some(thread),
        }

    }
}