use std::sync::mpsc;
use std::thread::JoinHandle;
use std::sync::mpsc::{Sender,Receiver};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;
pub struct IndexedTreadPool{

    workers: Vec<(IndexedWorker,Option<Sender<Job>>)>,
    num_workers: usize,

}

impl IndexedTreadPool{

    /// Creates a new thread pool by allocating a number of workers waiting to get jobs.
    pub fn new(num_workers: usize) -> Self{

        assert!(num_workers > 0);

        let mut workers = Vec::with_capacity(num_workers);

        for i in 0..num_workers{
            let (tx,rx) = mpsc::channel();
            let worker = IndexedWorker::new(i,rx);
            workers.push((worker,Some(tx)));
        }

        Self{
            workers,
            num_workers,
        }

    }

    /// Sends a job to a specific indexed worker.
    pub fn execute<F>(&self,index: usize, job: F)
        where F: FnOnce() + Send + 'static{

        assert!(index < self.num_workers);

        let job = Box::new(job);

        match self.workers[index].1{
            Some(ref tx) => {
                tx.send(job).unwrap();
            },
            None => unreachable!(),
        }


    }

}

impl Drop for IndexedTreadPool{

    /// Takes each transmitter, drops it and waits for the worker to finish.
    fn drop(&mut self) {


        for (worker,tx) in &mut self.workers{
            let tx = tx.take();
            drop(tx);

            let thread = worker.thread.take().unwrap();
            thread.join().unwrap();
        }

    }

}

struct IndexedWorker{

    index: usize,
    thread: Option<JoinHandle<()>>,

}

impl IndexedWorker{

    /// Creates a new indexed worker by spawning a thread that waits to receive jobs.
    pub fn new(index: usize,job_rx: Receiver<Job>) -> Self{

        // Create a new labeled thread that gets jobs and calls them
        let thread = thread::Builder::new().name(format!("Indexed worker {index}"))
            .spawn(move || {

                for job in job_rx{

                    job();

                }

                println!("{} disconnected.",thread::current().name().unwrap());

            }).expect("Indexed worker thread failed to create");

        Self{
            index,
            thread: Some(thread),
        }

    }

}


#[cfg(test)]
mod tests{
    use std::num::NonZero;
    use std::sync::Arc;
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    #[should_panic]
    fn test_indexed_thread_pool_1(){
        let pool = IndexedTreadPool::new(0);
    }

    #[test]
    #[should_panic]
    fn test_indexed_thread_pool_2(){
        let pool = IndexedTreadPool::new(3);

        pool.execute(5, move || {
        });

    }

    #[test]
    fn test_indexed_thread_pool_3(){

        let num_cores = thread::available_parallelism().unwrap_or(NonZero::new(6).unwrap()).get();
        let pool = IndexedTreadPool::new(num_cores);

        let counter = Arc::new(AtomicUsize::new(0));

        for i in 0..num_cores{

            let counter = Arc::clone(&counter);

            pool.execute(i,move || {
                counter.fetch_add(1, Ordering::SeqCst);
            })

        }

        drop(pool);

        assert_eq!(counter.load(Ordering::SeqCst), num_cores);

    }

}