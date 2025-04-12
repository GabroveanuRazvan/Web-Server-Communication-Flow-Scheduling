use std::collections::BinaryHeap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize)-> Self{
        assert!(size > 0);

        let (sender,receiver) = channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for label in 0..size{
            workers.push(Worker::new(label,Arc::clone(&receiver)));
        }

        Self{
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self,f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender
            .as_ref()
            .unwrap()
            .send(job)
            .unwrap();
    }

}

impl Drop for ThreadPool
{

    fn drop(&mut self){

        drop(self.sender.take());

        for worker in &mut self.workers{

            println!("Shutting down worker {}", worker.label);

           if let Some(thread) = worker.thread.take() {
               thread.join().unwrap();
           }

        }
    }

}

struct Worker
{
    label: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new(label: usize,receiver: Arc<Mutex<Receiver<Job>>>) -> Self{

        let thread = thread::Builder::new()
            .name(format!("Th_{label}"))
            .spawn(move || {
                loop{

                    let message = receiver
                        .lock()
                        .unwrap()
                        .recv();

                    match message{
                        Ok(job) => {
                            // println!("Worker labeled with {label} got a job.");
                            job()
                        }

                        Err(_) => {
                            println!("Worker labeled with {label} disconnected.");
                            break;
                        }

                    }
                }
            }).unwrap();

        Self{
            label,
            thread: Some(thread),
        }
    }
}