#![allow(dead_code)]
// In this problem, we will be learning about thread pools and work distribution
// The goal is to build a thread pool that efficiently manages a fixed number of worker threads to execute jobs

// Why use thread pools?
// - Avoid overhead of creating/destroying threads for each task
// - Control resource usage 
// - Reuse threads for multiple tasks
// - Better performance for handling many small tasks

// Without thread pool: Create 1000 threads for 1000 tasks
// With thread pool: Create 4 threads for all 1000 tasks

// We are building a system where:
// 1. Create once: Spawn N workers threads at the start
// 2. Reuse threads: Same threads handle all jobs (no spawning/destroying overhead)
// 3. Job queue: Main thread sends jobs through a channel
// 4. Workers compete: Each worker grabs the next available job
// 5. Track progress: Count how many jobs completed (using atomics)

// The main thread will create a channel for sending jobs
// It will spawn 4 workers threads, which wait for jobs
// Sends 20 jobs into the channel
// We will reuse the same 4 threads for all 20 jobs

use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

// This is a type alias
// It is used as short-hand for a longer type signature
// In this case, it is a boxed closure that:
// - Is in a box -> heap-allocated (size unknown at compile-time)
// - Can be called once (FnOnce())
// - Can be sent between threads (Send)
// - Doesn't reference any short-lived data ('static) - owns all its data or references only globals
type Job = Box<dyn FnOnce() + Send + 'static>;
// For references, &'static str means lives the entire program
// For types T: 'static means doesn't borrow anything with a limited lifetime - doesn't hold references to temporary data
// "Safe to send to another thread because it owns or references only global data"

// 'static is required since work threads run asynchronously 
// Asynchronously = multiple threads executing independently, without one waiting for another to finish, so work can happen in parallel or be overlapped in time
// They might execute AFTER main() exits
// They can't safely borrow stack data that might be gone
// They need to OWN their data or reference any globals

// dyn FnOnce() is still a trait object -> it represents some type that implements this trait, but we don't know which specific type at compile time
// dyn = dynamic dispatch = trait object
// We need Box<> since different closures have different sizes, compiler doesn't know which closure we will use, Rust requires all types to have a known size

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>, // the transmitter, which will be main in our case. We will have one sender and multiple receivers (possible through mutex since you can't clone a receiver)
    completed_count: Arc<AtomicU32>,
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl ThreadPool {
    // This associated function is for making the threadpool
    // It will spawn size number of threads
    fn new(size: usize) -> Self {
        // We are creating a transmitter (rx) and a receiver (rx)
        // That will be sending and receiving Jobs
        // mpsc = Multiple Producer, Single Consumer
        // Multiple threads can send (tx can be cloned) but only one can receive (rx cannot be cloned)
        // This is why we wrap rx in Arc<Mutex<>> below
        let (tx, rx) = mpsc::channel::<Job>(); // We are creating a channel that will send jobs between threads (main -> worker)

        // We are wrapping the receiver in Arc and Mutex
        // Arc = Multiple ownership (each worker gets a clone pointing to same receiver)
        // Mutex = Mutual exclusion (only one worker can recv() at a time)
        // This pattern allows multiple workers to share a single receiver
        // Pattern: Arc<Mutex<Receiver>> is standard for multi-consumer work queues
        let rx = Arc::new(Mutex::new(rx));

        // Creating a new Arc-wrapped AtomicU32 (starts at 0)
        // We need Arc here since it is not globally accessible (like 'static)
        // Arc allows multiple workers to share the same counter
        let completed_count = Arc::new(AtomicU32::new(0));

        // This will house our vector of Worker structs
        let mut workers = Vec::new();

        // Loop size times to spawn size workers
        for worker_id in 0..size {

                // Making new pointers to the same data
                let rx_clone = Arc::clone(&rx);
                let completed_clone = Arc::clone(&completed_count);

                // We are spawning a thread
                // Handles allow us to interact with spawned threads
                let handle = thread::spawn(move || {
                    // We are moving the captured variables into the closure
                    // This is so the thread can continue to use them after the loop iteration ends

                    // We are just using loop here since we do not know the number of messages each thread will receive
                    // Worker pattern: loop forever until channel closes
                    // 1. Try to receive a job 
                    // 2. Execute the job
                    // 3. Update the completed count
                    // 4. Loop back (repeat until channel closes)
                    loop {
                        
                        // The thread can acquire the lock, allowing it to receive a message
                        // We use .unwrap() in case the thread panics
                        // .recv() returns a Result, this is why we need to match
                        // After receiving, the thread immediately releases the lock so another thread can pick it up
                        let recv = rx_clone.lock().unwrap().recv(); // Lock is acquired and dropped here
                        // Minimal lock scope -> lock released at semicolon

                        // If we put rx_clone.lock().unwrap().recv() directly instead of the intermediate recv variable in the match expression
                        // the first spawned thread would hold the lock for all of the work and the other threads wouldn't be able to pick up the lock
                        // This is because temporaries in match expressions live until end of the match
                        // So the MutexGuard would not be dropped until an Err was encountered and the loop breaks
                        // Which, if that happens, there is nothing else being sent, since all the work is already finished
                        match recv {
                            // If the receiver gets a message (Ok(job)) it unwraps and it assigns it to job
                            Ok(job) => {
                                // We print some information
                                println!("Worker {} executing job", worker_id);
                                // We execute the job (since it is a closure)
                                job();
                                // We increment the counter but discard the old value
                                completed_clone.fetch_add(1, Ordering::SeqCst);
                            }
                            // Channel closed (sender was dropped)
                            // No more jobs coming, so exit the loop and let thread finish
                            Err(_) => break,
                        }
                    }
                });
                
                // Pushing size Worker structs to the vector
                workers.push(Worker {
                    id: worker_id,
                    thread:handle,
                })
        } 

        // Returning the ThreadPool
        // It can now be used with ThreadPool::new(4)
        Self {
            workers, // vector of Worker structs
            sender: tx,
            completed_count, 
        }

        // It is common to match the number of CPU cores when making a new threadpool
            // let num_cpus = std::thread::available_parallelism()
            //     .map(|n| n.get())
            //     .unwrap_or(4);
    }

    // This function takes a generic as input
    // In this case, f needs to be:
    // 1. Callable once (implement the FnOnce() trait)
    // 2. Be able to be sent across threads (Send)
    // 3. Does not reference any short-lived data ('static)
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {   
        // Since f has an unknown size at compile time, it has to be put in a Box
        // Box the closure to make it a Job (Box<dyn FnOnce() + Send + 'static>)
        let job = Box::new(f);

        // Now we are using the sender (tx) to send jobs to the receiver (rx)
        // Send the job through the channel to the workers
        // One of the workers will receive and execute it
        // This does not wait for the job to complete - it queues it - it does not wait for the job to actually finish running
        // The main thread continues immediately after sending -> returns immediately
        // This is important so you can submit many jobs quickly and they run concurrently, the main thread doesn't freeze, and allows for throughput
        self.sender.send(job).unwrap();
        // Fire and forget
        // Difference between queuing work and doing work
    }

    fn completed(&self) -> u32 {
        // Since self.completed_count is Arc<AtomicU32>, not u32, we need to load it
        // When we load the atomic value, it will give us a u32 in return
        self.completed_count.load(Ordering::SeqCst)
    }

    // self is consumed here so ThreadPool can't be used after .join() is called
    // This is by design - once we have waited for all workers to finish and shut down, the pool is no longer functional (channel is closed, workers exited)
    // Consuming self prevents accidentally trying to use a shutdown pool
    fn join(self) {

        // We need to drop the sender so that the receivers know to stop receiving
        // They will always keep receiving if they know a sender is out there
        // This prevents them from waiting forever
        drop(self.sender);

        // Using .join() so all the spawned threads join the main thread
        // This allows all the spawned threads to finish before the main thread continues
        for worker in self.workers {
            worker.thread.join().unwrap();
        }
    }

    // Graceful shutdown pattern:
    // Step 1: Drop the sender so workers know no more jobs are coming -> .recv() returns Err, causing workers to break from the loop
    // Step 2: Wait for each worker thread to finish its current job and exit
}

fn main() {

    // Spawn a pool with 4 worker threads
    let pool = ThreadPool::new(4);

    // Submit 20 jobs
    for i in 0..20 {
        pool.execute(move || {
            println!("Job {} starting", i);
            thread::sleep(Duration::from_millis(500));
            println!("Job {} complete", i);
        });
    }

    pool.join();
}

// Arc:
// - Allows multiple owners of one object
// - All owners have pointers to the same data
// - The data lives in one place (heap), pointers point to it
// - Cloning just increments the counter + new pointer
// - Data stays in one place 
// - Last owner drops = data deallocated

// Mutex:
// - Mutual Exclusion
// - Allows mutability for multiple owners but only one at a time
// - Lock/unlock mechanism
// - Automatic lock release (when guard goes out of scope/drops)