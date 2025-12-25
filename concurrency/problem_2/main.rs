
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

// This program demonstrates:
// - Arc<Mutex<T>> for shared mutable state across threads
// - Spawning multiple threads that perform different amounts of work
// - Lock acquisition and release in a loop
// - Joining threads to wait for completion
// - Non-deterministic thread execution order
// - Lock contention and why one thread might dominate

fn main() {
    // Creates an account that allows for shared ownership and mutability in a multi-threaded context
    let account: Arc<Mutex<f64>> = Arc::new(Mutex::new(1000.0));

    // This must be mutable (mut to push)
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    // Create a vector of tuples that will house different types of data which we will iterate over
    let transactions = vec![
        (0, "deposit", 100, 10),
        (1, "withdraw", 50, 10),
        (2, "deposit", 200, 5),
        (3, "withdraw", 25, 20),
        (4, "deposit", 150, 10),
    ];

    // In this outer, first loop we spawn all 5 threads very quickly
    // Each thread starts running immediately after being spawned
    // Stores all the handles in the vector
    // The loop exits, but the threads keep working in the background

    // Using a for loop with a tuple
    // This allows us to put several variables into our for loop since we are destructuring
    // We will run the loop a total of 5 times (since the transactions vector has a length of 5)
    for (thread_id, transaction_type, amount, times) in transactions {

        // Clones the Arc -> increments the reference count
        // Creates a new pointer to the same data
        // In each iteration of the loop, the ref count increases each time
        // It is moved into the thread and then the thread owns it
        // 5 threads = 5 reference count
        // As each thread completes, its account_clone is dropped
        // All point to same Mutex<f64> in memory
        // Arc = "How many owners exist?"
        let account_clone = Arc::clone(&account);
        // This only lives in this loop iteration -> move transfers ownership INTO the thread and it can use it as long as it needs
        // Every loop iteration has its own scope, so variables declared inside the loop are dropped at the end of each iteration

        // Now we are spawning a thread
        // A handle represents a running or finished thread
        // You use it to interact with a spawned thread (primarily waiting for it to finish)
        let handle = thread::spawn(move || {
            // We are using move so that the closure takes ownership of account_clone
            // Required because the closure needs to outlive the current scope
            // The spawned thread might run after the loop iteration ends
            // The thread lifetime is independent - it keeps running after the loop iteration ends
            // Moving transfers ownership from the loop scope into the thread's scope
            // Now the thread will own account_clone and can use it as long as it needs

            println!("Thread {} starting: Will {} {}, {} times.", thread_id, transaction_type, amount, times);

            // Each thread will do this 'times' times 
            // The order can vary since each thread can get the lock at the end of this for loop
            for _ in 0..times {

                // Lock the mutex to get access to the data
                // Now, a thread can access and modify the data until the loop ends
                // When the loop ends, the lock is released and another thread can pick it up
                let mut num = account_clone.lock().unwrap();
                // .lock() returns a Result because the lock could be "poisoned" if a thread panicked
                // A mutex becomes poisoned when a thread panics while holding the lock
                // The data might be in an inconsistent state
                // .unwrap() is fine for practice/simple programs -> panics if poisoned

                if transaction_type == "deposit" {
                    *num += amount as f64;
                    println!("Thread {}: deposited {}, new balance: {}.", thread_id, amount, *num);
                } else {
                    *num -= amount as f64;
                    println!("Thread {}: withdrew {}, new balance: {}.", thread_id, amount, *num);
                }
                // Lock releases here (if no more code follows)

                // To see more interleaving, you can explicitly release the lock and then sleep a tiny bit
                drop(num); // This explicitly releases the lock before sleeping 
                // Without drop(), the lock would be held during the sleep 
                // By dropping first, other threads can acquire the lock while this thread sleeps
                thread::sleep(Duration::from_micros(50));  // Sleep in microseconds
            }

            println!("Thread {} finished.", thread_id);
        });

        // Store the handles so we can .join() later
        // This will allow all threads to finish before continuing
        handles.push(handle);
    }

    // At this point: All 5 threads are already running parallel
    // All threads start working immediately when spawned
    // They work in parallel (at the same time)

    // Now, we will wait for all threads to finish before moving on
    // If we did not do this, then the main thread would continue before letting the spawned threads finish
    // When the main thread exits, the entire program exits, even if the spawned threads aren't done
    // When you iterate over the handles, you wait for each thread to finish
    for handle in handles {
        // .join() returns a Result because the thread might have panicked
        // In production code, the error should be handled explicitly
        handle.join().unwrap();
    }

    // Even though all threads are finished, you still need to acquire the lock to read the value inside of the Mutex
    // You can't just read it since it is Arc<Mutex<f64>> not just f64
    // You also simply can't dereference for the same reason
    // The mutex always protects the data - even if no other threads are running, the Mutex still wraps the data 
    // You need to go through the lock mechanism to access it
    let final_balance = account.lock().unwrap();
    println!("Final balance: {}", *final_balance);
}

// In the output, you will notice that the threads seem to spawn at different times, with some threads doing work in-between
// But all threads spawn simultaneously (microseconds apart)
// This is because they all compete for CPU time, mutex lock, console output (printing can be relatively slow)
// Threads can reacquire locks immediately so they do more work before other threads
// This is lock contention - threads are fighting for the lock
// The OS scheduler decides which thread gets CPU time - this order is non-deterministic (unpredictable)
// Even though the order varies, the final result is always correct due to the Mutex preventing data races

// We have a 2-phase pattern:
// Phase 1 (first loop): Spawn all threads quickly - they start working immediately
// Phase 2 (second loop): Wait for all threads to complete 
// This allows true parallelism - all threads work simultaneously
// If we joined inside the first loop, threads would run sequentially (one at a time)

// A handle is a value that represents ownership or control of some resource
// In Rust threading, a JoinHandle is what you get back when you spawn a thread 
// It represents a running thread and lets you interact with it