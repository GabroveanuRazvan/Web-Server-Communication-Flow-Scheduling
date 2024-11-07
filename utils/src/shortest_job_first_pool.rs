use std::cmp::Reverse;
use std::collections::BinaryHeap;
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
    heap: Arc<(Mutex<BinaryHeap<Reverse<J>>>, Condvar)>,
    workers: Vec<SjfWorker<J>>,
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
        let heap = Arc::new((Mutex::new(BinaryHeap::new()),Condvar::new()));
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
        heap_guard.push(Reverse(job));

        // unlock the heap and notify one of the workers
        drop(heap_guard);
        cvar.notify_one();

    }

}

/// Worker struct to be used by Shortest Job First Scheduler
pub struct SjfWorker<J>
where
    J: Job + Ord + Send + 'static,
{
    label: usize,
    thread: Option<JoinHandle<()>>,
}

impl<J> SjfWorker<J>
where
    J: Job + Ord + Send + 'static,
{

    pub fn new(label: usize, heap:Arc<(Mutex<BinaryHeap<Reverse<J>>>,Condvar)>) -> Self{

        
        let thread = thread::spawn(move||{

            let (mutex,cvar) = &*heap;

            loop{

                let mut heap_guard = mutex.lock().unwrap();

                while heap_guard.is_empty(){
                    heap_guard = cvar.wait(heap_guard).unwrap();
                }

                let job = heap_guard.pop().unwrap().0;
                drop(heap_guard);

                job.execute();

            }

        });

        Self{
            label,
            thread: Some(thread),
        }

    }

}

