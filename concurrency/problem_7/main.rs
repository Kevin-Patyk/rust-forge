// In this problem, we we learn about atomic types and lock-free programming
// Atomics are a way to share data between threads without locks

// What are atomics?
// Atomic types provide lock-free, thread safe operations on simple values
// Key idea: Operations happen "atomically" - all at once, indivisibly, with no interruption

// With Mutex, threads need to get the lock, modify the data, then unlock
// With multiple spawned threads and multiple instances of locked data, different threads can acquire different locks 
// which can result is deadlocks
// With Mutex, there is overhead: lock acquisition, context switching, potential deadlocks

// With atomics, there is a direct cpu instruction
// No locks, no blocking, just a single atomic CPU operation

// Use atomics when:
// Simple operations (increment, decrement, swap, compare-and-swap)
// Single values (counters, flags, IDs)
// Performance critical
// Lock-free algorithms

// Use Mutex when:
// Complex operations (need multiple steps)
// Multiple values that must change together
// Need to hold state across multiple operations

#![allow(unused_imports)]
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicUsize, Ordering};
// AtomicBool - true/false flags
// AtomicI32 - Signed 32-bit integer
// AtomicU32 - Unsigned 32-bit integer
// AtomicUsize - Platform-dependent size
// Atomic I64 - Signed 64-bit integer

// Atomics required a "memory ordering" parameter
// Ordering::Relaxed - Fastest, minimal guarantees
// Ordering::Acquire - For Reads
// Ordering::Release - For writes
// Ordering::AcqRel - Both acquire and release
// Ordering::SeqCst - Strongest, easiest to reason about

use std::sync::Arc;
use std::thread;
use std::time::Duration;
use rand::Rng;
use std::thread::JoinHandle;

struct Statistics {
    // For all of our struct fields, we are using the AtomicU32 type
    total_processed: AtomicU32,
    in_progress: AtomicU32,
    success_count: AtomicU32,
    error_count: AtomicU32,
}

impl Statistics {
    // Creating an associated function 
    // An associated function is defined in an impl block and does not take self as a parameter
    fn new() -> Self {
        Self {
            // We are instantiating every field with an AtomicU32 (0)
            total_processed: AtomicU32::new(0),
            in_progress: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            error_count: AtomicU32::new(0),
        }
    }

    fn start_task(&self) {
        // .fetch_add() is an atomic operation that
        // 1. Adds a value to the current value
        // 2. Returns the OLD value (before the addition)
        // 3, Does both steps atomically (as one indivisible operation)
        self.in_progress.fetch_add(1, Ordering::SeqCst);
        // "Fetch" = get the old value before modifying
        // It is called fetch and add because it:
        // 1. Fetches (gets) the current value
        // 2. Adds to it
        // 3. Returns it when fetched
        // It is a single atomic operation that does read-modify-write all at once, with no chance for other threads to interfere
        // Note: If we do not assign the outcome to a variable, the old value is discarded, but the counter still gets incremented
    }

    fn complete_task(&self, success: bool) {
        // .fetch_sub() works the same as .fetch_add() but for subtraction
        // So, it subtracts from the current value and returns the old value before subtraction
        // If you do not assign the outcome to a variable, the old value is discarded, but the counter still gets decremented
        self.in_progress.fetch_sub(1, Ordering::SeqCst);

        self.total_processed.fetch_add(1, Ordering::SeqCst);

        if success {
            self.success_count.fetch_add(1, Ordering::SeqCst);
        } else {
            self.error_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn print_stats(&self) {
        // .load() is an atomic operation that allows us to load (read) the current value
        // .store() is an atomic operation that allows us to store (write) a new value
        let total_processed = self.total_processed.load(Ordering::SeqCst);
        let in_progress = self.in_progress.load(Ordering::SeqCst);
        let success_count = self.success_count.load(Ordering::SeqCst);
        let error_count = self.error_count.load(Ordering::SeqCst);
        println!("Current statistics:");
        println!("Total Processed: {}", total_processed);
        println!("In Progress: {}", in_progress);
        println!("Success Count: {}", success_count);
        println!("Error Count: {}", error_count);
    }
}

fn main() {
    
    // Creating a new instance of the statistics struct wrapped in Arc
    // Arc is atomic reference count
    // It keeps track of the number of references to an object in a multi-threaded context
    // It creates a new pointer to the same data
    // It gets automatically dereferenced
    // We do not need Mutex here since we are working with atomic types
    let stats = Arc::new(Statistics::new()); // This is the shared data 
    // When we need multiple threads to interact with the same data, we use Arc
    // Multiple owners can share this data across threads

    // Why atomics work here:
    // Each field is modified independently -> Each field can be changed without caring about the values of the other fields/no relationship between fields during modification
    // Operations are simple (increment/decrement)
    // No need to maintain consistency between multiple fields at once
    // Good use case for lock free atomic operations

    // Atomics don't work when fields are dependent because:
    // Can't make multiple fields change atomically together
    // Other threads can observe in-between states
    // Need Mutex to group multiple operations together

    // Making an empty vector to store the handles in
    // We will later call .join() on each handle in this vector to allow the threads to finish executing 
    // before continuining with the main thread
    // A handle represents a spawned thread and allows us to interact with it
    let mut handles: Vec<JoinHandle<()>> = vec![];

    for worker_id in 0..5 {

        // Making a clone of stats
        // This is incrementing the reference count
        // This will be moved into the thread so that it can continue using it after this loop iteration ends
        // Each loop iteration has its own scope, so this would be dropped at the end of the loop iteration if we did not move it into the thread
        let stats_clone = Arc::clone(&stats);

        // Spawning a thread here
        // In total, we will spawn 5 threads
        // We are moving ownership of stats_clone into the thread since the thread's lifetime is independent
        // It will still continue running after the spawning loop is finished and it needs to be able to use stats_clone
        let handle = thread::spawn(move || {

            // This is from the rand crate
            // It must be mutable since RNG changes internal state each time you use it
            // rng() is a function that creates a random number generator 
            // It is a thread-local random number generator
            // Thread-local means it creates RNG that is specific to the current thread
            // Each thread gets its own independent RNG
            let mut rng = rand::rng();

            // Each worker thread will process 100 tasks
            for task_num in 0..100 {

                // Start the task
                stats_clone.start_task();

                // Simulate work
                thread::sleep(Duration::from_millis(1));

                // Generate a random success
                // This generates a random number between 0.0 and 1.0
                let random_num: f64 = rng.random();
                let success = random_num < 0.8;

                 stats_clone.complete_task(success);

                // Every 20 tasks, we will print the progress of the thread
                // The % operator is the modulo (remainder) operator
                // It gives you the remainder after division
                // It is good for checking even or odd
                // The remainder when a is divided by b
                 if task_num % 20 == 0 {
                    println!("Worker {}: Processed {} tasks", worker_id, task_num);
                 }
            }

        });

        handles.push(handle);
    }

    // Bonus challenge 
    // Add a progress monitor thread that prints stats every 100ms while workers are running
    // We spawn the monitor thread after the workers because we need work to have started so it doesn't exit immediately
    let monitor_stats = Arc::clone(&stats);
    let monitor = thread::spawn(move || {

        loop {
            thread::sleep(Duration::from_millis(100));

            let in_progress = monitor_stats.in_progress.load(Ordering::SeqCst);

            if in_progress == 0 {
                break;
            }

            println!("Progress Update:");
            monitor_stats.print_stats();
        }

    });

    // Now, we will allow all spawned threads to finish their work
    // If we did not allow the spawned threads to "join" the main thread, then the main thread would finish before spawned threads did
    // .unwrap() is called if a thread panics
    for handle in handles {
        handle.join().unwrap();
    }

    // Wait for monitor to finish
    monitor.join().unwrap();

    // Printing the final statistics
    println!("Final Stats:");
    stats.print_stats();

    // .fetch_add() example to demonstrate how it returns the old value
    let counter = AtomicU32::new(5);

    let old_value = counter.fetch_add(3, Ordering::SeqCst);

    println!("Old value: {}", old_value); // Prints 5 (before addition)
    println!("New value: {}", counter.load(Ordering::SeqCst)); // Prints 8 (after addition)
}

// This problem demonstrated lock-free concurrent programming with atomics
// - 5 threads all incrementing the same counters without locks
// - No data races due to atomic operations
// - Used AtomicU32 for thread-safe counters
// - Used atomic operations
// - Used Ordering::SeqCst for strongest memory ordering guarantees
// - Demonstrated live progress monitoring with a separate thread
// - Final count is 500 (5 works * 100 tasks)

// In Series proptest, we do:

// A global, thread-safe counter that will be used to ensure unique column names when the Series are created
// This is especially useful for when the Series strategies are combined to create a DataFrame strategy

// static COUNTER: AtomicUsize = AtomicUsize::new(0);

    // fn next_column_name() -> String {
    //     format!("col_{}", COUNTER.fetch_add(1, Ordering::Relaxed))
    // }

// Create a static (global) variable (lives for the entire program) named COUNTER
// We assign a thread-safe unsigned integer to it starting at 0
// Each call to next_column_name() will increment COUNTER
// Each call the next_column_name() will increment the COUNTER +1 and return the old value for naming 
// First call = returns "col_0" and increments to 1
// Second call = returns "col_1" and increments to 2
// This is thread safe since proptest often runs tests in parallel (multiple threads)
// No need for Mutex here, since it would be overkill
// We are not using Arc here since static variables are already globally shared - they don't need Arc because they're not owned by any particular thread or scope