use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::ops::DerefMut;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

/// Trait used mainly by SjfPool.
pub trait Job{
    /// Each Job should be able to be executed.
    fn execute(&self);
}

/// Shortest Job First Thread Pool.
///
/// Each pool is initialized with a number of workers and uses the schedule_job method to add a job to be executed by shortest job first scheduler.
///
pub struct SjfPool<J>
where
    J: Job + Ord + Send + 'static,
{
    heap: Arc<(Mutex<Option<BinaryHeap<Reverse<J>>>>, Condvar)>,
    workers: Vec<SjfWorker>,
}

impl <J> SjfPool<J>
where
    J: Job + Ord + Send + 'static,
{

    /// Creates a new SJF Pool with a nonnegative size.
    ///
    pub fn new(size: usize) -> Self {

        assert!(size > 0);

        // create the mutex protected min-heap and its condition variable
        let heap = Arc::new((Mutex::new(Some(BinaryHeap::new())),Condvar::new()));
        let mut workers = Vec::with_capacity(size);

        // create the workers and pass them the heap
        for i in 0..size{
            workers.push(SjfWorker::new(i,Arc::clone(&heap)));
        }

        Self{
            heap,
            workers,
        }

    }

    /// Adds a new job to be processes by an available worker.
    ///
    pub fn schedule_job(&mut self, job: J){

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

}

/// Drop trait used to gracefully shut down all worker threads.
///
impl<J> Drop for SjfPool<J>
where
    J: Job + Ord + Send + 'static,
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

/// Worker struct to be used by Shortest Job First Scheduler.
///
pub struct SjfWorker
{
    label: usize,
    thread: Option<JoinHandle<()>>,
}

impl SjfWorker
{
    /// Creates a new worker, with a passed heap and condition variable.
    ///
    pub fn new<J>(label: usize, heap:Arc<(Mutex<Option<BinaryHeap<Reverse<J>>>>,Condvar)>) -> Self
    where
        J: Job + Ord + Send + 'static,
    {


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
                    job.execute();
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

