use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex}; // Multiple Producer, Single Consumer
use std::thread::JoinHandle;
use std::thread;
use std::time::Duration;

// In this problem, we are combining what we learned: use channels for task distribution AND Arc<Mutex> for tracking progress
// We are building a simple task processing system:
// Main thread sends tasks through a channel
// 3 worker threads receive and process tasks
// Track total completed tasks using shared state

fn main() {

    // Create a channel
    // tx = transmitter
    // rx = receiver
    // mpsc = multiple producer, single consumer
    // multiple threads can send (multiple producer)
    // only one thread can receive (single consumer)
    let (tx, rx) = mpsc::channel::<String>();

    // Create a shared counter
    // Creates a counter that allows for shared ownership and mutability in a multi-threaded context
    // Arc = Atomic Reference Count
    // Mutex only allows for one thread at a time to access data - prevents data races
    let counter: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));

    // Creating an empty vector that will hold JoinHandle from our spawned threads
    // Handles represent a running or finished thread - they allow us to interact with spawned threads
    // We will later call .join() on these in a for loop to let the threads finish before letting the main thread continue
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    // We cannot clone the receiver (rx) since, with mpsc, there's only one receiver
    // We will move the receiver (rx) 
    // We are creating it outside the loop - create once, share with all workers
    // The receiver already has internal synchronization for receiving messages safely from multiple threads, but the type is NOT sync
    // Arc<T> requires T: Sync (T must be safely shareable across threads)
    // Receiver<T> does not implement Sync
    // Mutex<T> adds Sync to any T: Send
    let receiver: Arc<Mutex<Receiver<String>>> = Arc::new(Mutex::new(rx));

    // Mutex serves 2 purposes:
    // 1. Makes Receiver shareable (Sync) - satisfies Rust type system
    // 2. Coordinates access between workers - only one worker can receive at a time
    // The Receiver already has internal synchronization, but Rust's rules require the Mutex wrapper

    // Spawn 3 worker threads
    for worker_id in 0..3 {

        // In each loop, we are cloning the counter and receiver
        // This will increment the reference count
        // It creates a new pointer to the same data
        // In this case, it will increase the reference count by 3
        // Since every for loop iteration has its own scope, this will be dropped at the end of the iteration if not moved
        let counter_clone = Arc::clone(&counter);
        let rx_clone = Arc::clone(&receiver);

        // Here, we are spawning a worker thread 
        // We are moving counter_clone and rx_clone into the thread
        // This will allow the thread to take ownership of them and keep using it after the for loop iteration ends
        // If we did not, then counter_clone and rx_clone would be dropped at the end of the iteration
        // The thread's lifetime is independent - it keeps running after the loop iteration ends
        // Now the thread will own these so it can keep using them as long as it needs
        let handle = thread::spawn(move || {
            
            // We are using loop {} instead of a for {} loop because there is an unknown number of messages
            // The worker doesn't know how many messages it will receive 
            // With 3 workers, messages are distributed among workers
            // It is non-deterministic - you can't predict which workers gets which messages
            // We will exit when .recv() returns Err (all senders dropped)

            // Workers compete for each message
            // Worker locks -> receive one message -> unlocks -> processes
            // While one worker processes, others can lock and receive their own messages
            loop {
                // Since rx_clone is wrapped in Mutex, we need to acquire the lock
                // The Arc is automatically dereferenced
                let task = rx_clone.lock().unwrap().recv(); // Lock acquired and immediately released
                // In our previous problems, the lock would be dropped at the end the loop but here it is dropped after .recv()
                // because the MutexGuard goes out of scope
                // In our previous problems, we assigned the MutexGuard to num and as long as num exists, the lock held
                // But num went out of scope at the end of the loop
                // Task holds the Result, not the MutexGuard
                // Lock is released at the end of the statement, not the loop iteration
                // This is why different threads can then pick up the lock
                
                // The .lock().unwrap() creates a TEMPORARY MutexGuard
                // The temporary is NOT stored in a variable, so it drops immediately after .recv()
                // If we did let guard = rx_clone.lock().unwrap(); - the lock would be held longer
                // By not storing the guard, we ensure minimal lock time

                // We acquire the lock for the MINIMUM time necessary
                // Lock -> do the critical operation -> unlock immediately
                // Don't hold locks while doing slow operations, otherwise other threads can't access the data
                // The lock will be dropped immediately, so that other threads can acquire it

                // .unwrap() is called on .lock() since the thread can be poisoned, so it will panic

                // .recv() BLOCKS (waits until a message arrives)
                // The thread pauses execution while waiting
                // When a message arrives, the thread wakes up and continues
                match task {
                    Ok(message) => {
                        println!("Worker: {} Processing {:?}", worker_id, message);
                        thread::sleep(Duration::from_millis(100));

                        *counter_clone.lock().unwrap() += 1;
                    }
                    Err(_) => break,
                }
            }
            
        });

        // Pushing each handle to the vector so that they can be joined later
        handles.push(handle);

    }

    // Sending 10 tasks through the transmitter (tx)
    for i in 0..10 {
        // We use .unwrap() on .send() since it returns a Result because sending can fail if the receiver has been dropped
        // If the receiver is dropped, then there is no one to receive the message
        // In our code: "If the receiver is gone, panic (crash the thread)."
        tx.send(format!("Task {}", i)).unwrap();
    }

    // Dropping the transmitter so that the receiver knows to stop receiving
    // The receiver will only stop receiving when there are no more senders (transmitters)
    // If you do not drop the transmitter, then the receiver will wait forever
    drop(tx);

    // Allow all of the threads to finish before continuing
    // If we did not do this, the main thread would continue even if the spawned threads are running
    // When you iterate over the handles and call .join(), you allow all of the running threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // We need to call .lock() since the Mutex always protects the data, even when no threads are running
    // This is why you also just can't simply dereference it
    // You need to go through the lock mechanism to access it
    let final_result = counter.lock().unwrap();
    println!("Final result: {}", *final_result);
    // Here, final_result will be dropped, so the lock will also be dropped
}

// 1. All 3 workers spawn and start their loops
// 2. Worker 0 wins first - locks, receives Task 0, unlocks, processes
// 3. Worker 1 wins second - locks, receives Task 1, unlocks, processes
// 4. Worker 2 wins third - lock, receives Task 2, unlocks, processes
// 5. And so on...

// Comparison with previous problems:
// Previous problem: One receiver (main thread), multiple senders (spawned threads) - simple channel usage
// Current problem: Multiple receivers (spawned threads), one sender (main thread) - need Arc<Mutex<Receiver>> 