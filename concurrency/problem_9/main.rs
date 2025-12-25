// In this problem, we will introduce Barrier synchronization 
// This will teach us coordinating multiple threads to reach synchronization points together

// We will learn about std::sync::Barrier for coordinating threads at synchronization points
// and build a multi-phase data processing pipeline

// We will learn:
// - Barrier synchronization (threads wait for each other)
// - Multi-phase parallel processing
// - Combining barriers with channels and atomics
// - Pipeline pattern with synchronized stages

// Scenario: Build a data processing pipeline with 3 phases:
// 1. Fetch phase: Workers fetch raw data
// 2. Process phase: Workers transform the data
// 3. Aggregate phase: Workers combine results

// All workers must complete each phase before ANY worker can proceed to the next phase
// Key challenge: Use Barrier to ensure phase synchronization - no workers start phase 2 until ALL workers finish phase 1

use std::sync::{Arc, Barrier, mpsc};
use std::sync::atomic::{AtomicU32, Ordering}; // Atomic data types are data types that allow safe concurrent access to shared data across multiple threads without using locks
// They rely on hardware-level atomic instructions
// When multiple threads read and write the same variable, you can get data races
// Atomic types prevent data races, guarantee indivisible operations, and are faster than mutexes for simple shared state
use std::thread;
use std::time::Duration;
use rand::Rng;

struct DataItem {
    id: u32,
    raw_value: u32,
    processed_value: Option<u32>,
}

#[derive(Debug)]
struct PipelineStats {
    phase1_complete: AtomicU32,
    phase2_complete: AtomicU32,
    phase3_complete: AtomicU32,
    total_processed: AtomicU32,
}

impl PipelineStats {
    // Associated function
    // Does not take self as an input parameter
    // Does not need an instance of the struct to work
    // Creates a new instance of the struct
    fn new() -> Self {
        Self {
            phase1_complete: AtomicU32::new(0),
            phase2_complete: AtomicU32::new(0),
            phase3_complete: AtomicU32::new(0),
            total_processed: AtomicU32::new(0),
        }
    }
}

fn main() {

    // This demonstrates the PipelinePattern:
    // - Multiple workers process data in synchronized phases
    // - Barriers ensure no worker gets ahead (phase synchronization)
    // - Channel collects final results
    // - Atomics track progress without locks
    // Real-world use: ETL pipelines, batch processing, map-reduce

    // Creating a barrier for 4 workers
    // barrier.wait() blocks until ALL threads call it
    // Barrier needs to be in Arc because multiple threads need to share ownership of the same Barrier
    // Arc allows multiple owners of the same barrier
    // Each thread gets a pointer (Arc clone) to the same Barrier
    // When threads call .wait() on the same barrier, it releases them all
    let barrier = Arc::new(Barrier::new(4));

    // Note: The number you pass to Barrier::new(N) must match the number of threads that will call .wait() on it
    // If mismatched, threads will deadlock (hang forever)
    // The barrier resets after each synchronization, so it can be resused for multiple phases

    // When the 4th (last) threads calls .wait(), ALL threads are released simultaneously
    // This ensures no thread proceeds to the next phase until everyone finishes the current phase

    // This does not need to mutable since AtomicU32 provides interior mutability - you can mutate through a shared reference
    // It can be modified through &self using atomic CPU instructions
    // We need to wrap this in Arc since it is not globally available
    let stats = Arc::new(PipelineStats::new());

    // We are creating a transmitter and receiver to send messages through
    // There can be multiple transmitters (can clone them) but only one receiver 
    // We can have "multiple" receivers, though, if we put rx in Arc and Mutex
    let (tx, rx) = mpsc::channel();

    // Creating a mutable vector to store handles in
    // A handle represents a running or finished thread
    // It is a way to interact with threads
    // We will .join() on each handle later to allow the threads to finish running before the main thread continues
    let mut handles = Vec::new();

    for worker_id in 0..4 {

        // Cloning the barrier
        // Incrementing the reference count
        // Making a new pointer to the same data
        // This will be moved into the thread, since the thread's lifetime is independent and it will continue after the loop iteration ends
        let barrier_clone = Arc::clone(&barrier);

        // Cloning the stats
        // Incrementing the reference count
        // Making a new pointer to the same data
        // This will also be moved into the thread
        let stats_clone = Arc::clone(&stats);

        // Cloning the transmitter
        // The transmitter will be moved into the thread so it can send messages through a channel to the receiver (main thread)
        let tx_clone = tx.clone();

        // We are moving the variables captured from the surrounding scope into the thread
        // This is so the thread now owns the data
        // If we did not move it, it would be dropped at the end of the for loop iteration
        // If we only referenced it, Rust would reject the code at compile-time
        // because the thread might outlive the reference (no dangling references allowed)
        let handle = thread::spawn(move || {

            // Start phase 1
            println!("Worker {}: Phase 1 starting,", worker_id);

            // Create an empty vector that will store DataItem
            let mut items: Vec<DataItem> = Vec::new();
            // Start the random number generator
            let mut rng = rand::rng();

            // Create 10 random values and store them in DataItem structs
            // We are creating 10 DataItem structs per thread
            // We will now have a vector of 10 DataItem
            for item_id in 0..10 {
                // Each worker uses IDs 0-9, so IDs will repeat across workers
                // In production, you would use unique IDs
                let random_value = rng.random_range(1..100);
                items.push(DataItem {
                    id: item_id,
                    raw_value: random_value,
                    processed_value: None,
                });
            }

            println!("Worker {}: Phase 1 complete (generated {} items)", worker_id, items.len());

            // Now, we will increment the phase 1 counter in the stats struct
            // We need to use .fetch_add(), which fetches the old value and increments the counter
            // In this case, we are discarding the old value and just incrementing the counter
            // This is an atomic operation
            stats_clone.phase1_complete.fetch_add(1, Ordering::SeqCst);

            // SeqCst provides the strongest memory ordering guarantees
            // It ensures all threads see operations in the same order
            // For learning, SeqCst is safest - in production, weaker orderings like Relaxed might suffice

            barrier_clone.wait(); // Wait for all workers to finish Phase 1 before continuing

            // Start phase 2
            println!("Worker {}: Phase 2 starting", worker_id);

            // We are iterating over a vector of DataItem structs
            for item in &mut items {
                // Simulate processing time
                thread::sleep(Duration::from_millis(10));

                // We are updating the processed_value field by multiplying the raw value by 2
                item.processed_value = Some(item.raw_value * 2);
            }

            println!("Worker {}: Phase 2 complete (processed {} items)", worker_id, items.len());

            stats_clone.phase2_complete.fetch_add(1, Ordering::SeqCst);

            barrier_clone.wait(); // Wait for all workers to finish Phase 2 before continuing

            println!("Worker {}: Phase 3 starting", worker_id);

            // We are iterating over a vector of DataItem structs
            for item in items {
                // We are sending each DataItem in the vector through the channel to the receiver (main thread)
                tx_clone.send(item).unwrap();

                // Incrementing the total_processed by 1 after each send
                // You can also increment by 10 at the end (all items from the worker) after the loop
                stats_clone.total_processed.fetch_add(1, Ordering::SeqCst);
            }

            println!("Worker {}: Phase 3 complete (sent {} items)", worker_id, 10);

            stats_clone.phase3_complete.fetch_add(1, Ordering::SeqCst);

            barrier_clone.wait(); // Wait for all workers to finish Phase 3 before continuing
        });

        // Pushing the handle to the vector so we can call .join() on them later and allow all of them to finish
        handles.push(handle);
    }

    // Order matters:
    // 1. Drop tx (signal no more messages coming)
    // 2. Receive all 40 items from channel
    // 3. Join handles (wait for workers to finish cleanup/exit)
    // 4. Print final stats
    
    // Drop the original transmitter so the receivers (main thread) know when to stop
    // Without this, the channel never closes
    // The receiving loop would wait forever
    // The transmitters in the threads will drop on their own, but we manually need to drop the original
    drop(tx);
    
    // Now we will receive before joining
    // We use an explicit loop + match instead of a for loop to show the pattern clearly
    // The loop continues until all senders are dropped (Err from recv)
    // Alternative: for received in rx {} -> does same thing implicitly
    loop {
        match rx.recv() {
            Ok(received) => {
                println!("Received item {}: raw={}, processed={:?}", received.id, received.raw_value, received.processed_value);
            }
            // All senders dropped, exit the loop
            Err(_) => {
                break;
            }
        }
    }

    // Allow all of the threads to finish before the main thread continues
    for handle in handles {
        // We are allowing the spawned threads to "join" the main thread
        // We call .unwrap() since a thread can panic
        handle.join().unwrap();
    }

    // We do not need to acquire a lock for stats since we do not have a Mutex
    // We are using lock-free programming due to atomics
    // AtomicU32.load() safely reads the value without locks
    // All threads have finished (joined) so there's no more concurrent access anyway
    println!("Final stats {:?}", stats);
}
