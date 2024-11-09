use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::JoinHandle;
use crate::http_parsers::{basic_http_response, http_response_to_string};
use crate::mapped_file::{MappedFile, MappedFileJob};
use crate::sctp_client::SctpStream;
use crate::shortest_job_first_pool::{Job, SjfPool};

pub struct ConnectionScheduler{

    heap: Arc<(Mutex<Option<BinaryHeap<Reverse<MappedFile>>>>,Condvar)>,
    stream: Arc<SctpStream>,
    workers: Vec<ConnectionWorker>,

}

impl ConnectionScheduler{

    pub fn new(size: usize, stream: SctpStream)-> Self{
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let stream = Arc::new(stream);
        let heap = Arc::new((Mutex::new(Some(BinaryHeap::new())), Condvar::new()));

        for i in 0..size{
            workers.push(ConnectionWorker::new(i,Arc::clone(&heap),Arc::clone(&stream)));
        }

        Self{
            heap,
            stream,
            workers,
        }

    }

}

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

pub struct ConnectionWorker{
    label: usize,
    thread: Option<JoinHandle<()>>,
}

impl ConnectionWorker{
    pub fn new(label: usize,heap: Arc<(Mutex<Option<BinaryHeap<Reverse<MappedFile>>>>,Condvar)>, stream: Arc<SctpStream>) -> Self{

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

                    let mut response_bytes = http_response_to_string(basic_http_response(job.file_size())).into_bytes();
                    let stream_number = label as u16;

                    match stream.write_all(&mut response_bytes,stream_number,0){
                        Ok(bytes) => println!("Wrote {bytes}"),
                        Err(e) => println!("Write Error: {:?}",e)
                    }

                    // send the body of the response
                    match stream.write_chunked(&job.mmap_as_slice(),2048,stream_number,0){
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