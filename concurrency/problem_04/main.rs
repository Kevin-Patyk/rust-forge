use std::sync::mpsc; // Multiple Producer, Single Consumer
use std::thread;
use std::time::Duration;

// In this practice problem, we will learn about message passing
// This is a completely different approach to concurrency than shared state (Arc/Mutex/RwLock)

// Instead of sharing memory between threads, you send messages between them
// "Don't communicate by sharing memory; share memory by communicating"
// Channel = A pipe for sending data from one thread to another

// When using Arc/Mutex/RwLock, we are used a shared-state approach
// Multiple threads will access the same memory location
// Threads share a single piece of data via Arc
// Locks (Mutex/RwLock) prevent simultaneous access
// Threads compete for a lock
// When we access the lock, we are accessing the same shared memory

// In the message passing approach, threads have their own data and send copies to each other
// Thread 1 -> Sends Message (Data) -> Channel -> Thread 2
// Each thread owns its own data
// There is no shared memory -> data is moved or copied through messages
// Threads communicate by sending messages through a channel
// No locks needed - ownership transfers through messages

// Real-world analogy

// Shared State = Shared Whiteboard
// 10 people need to write on the same whiteboard
// The whiteboard is the shared memory -> everyone sees the same whiteboard
// Only 1 person can write at a time (lock)
// Others must wait their turn

// Message Passing = Passing Notes
// Person 1 writes a note and hands it to Person 2
// Person 1 no longer has the note (ownership transferred)
// Person 2 writes a response and hands it to Person 3
// No one is waiting for access - data flows through the system

// Use Arc/Mutex when multiple threads need to modify the same data structure
// Use message passing (channels) for producer-consumer pattern/pipeline or workflow processing/decoupled thread communication

// Instead of everyone fighting over one piece of data (shared state), you let threads pass data between themselves (message passing)

// Key Differences:
// Shared State (Arc/Mutex):
// - Multiple threads access SAME memory
// - Locks prevent simultaneous access
// - Threads compete/wait for locks
// - Good for: Counters, shared config, caches

// Message Passing (Channels):
// - Threads have OWN data, send copies
// - No locks needed
// - Threads wait for messages, not access
// - Good for: Producer-consumer, pipelines, task queues

fn main() {
    // Create a channel
    // tx is the transmitter - sends messages
    // rx is the receiver - receives messages
    // mpsc = multiple producer, single consumer
        // multiple threads can send (producer)
        // only one thread receives (consumer)
    let (tx, rx) = mpsc::channel();

    // Creating an empty vector where all of the handles will go
    // Handles represent running threads and allow us to interact with them
    let mut handles = Vec::new();

    // We are spawning 3 producer threads here
    for i in 0..3 {

        // We are cloning the transmitter
        let tx_clone = tx.clone();

        // A handle represents a finish or running thread
        // When you spawn a thread it returns a JoinHandle that represents a thread and allows us to interact with it
        let handle = thread::spawn(move || {

            // We are moving tx_clone here so that the thread takes ownership of it and can use it after the loop iteration is finished
            // Otherwise, tx_clone would go out of scope when the loop iteration ends
            // Every loop iteration has its own scope

            // Here, we are assigning the result of an if else statement to determine what the prefix should be
            let prefix = if i == 0 {
                "Job"
            } else if i == 1 {
                "Task"
            } else {
                "Work"
            };

            // Since each thread needs to send 3 different messages, we are using another for loop
            for j in 0..3 {
                // Sending a message
                // Messages can be any type that implements Send (most types do)
                // When you send through a channel, ownership is transferred (moved) not cloned (this depends on the type but usually ownership transfer)
                tx_clone.send(format!("{} {}", prefix, j)).unwrap();
                // We use .unwrap() on .send() since it returns a Result because sending can fail if the receiver has been dropped
                // If the receiver is dropped, then there is no one to receive the message
                // In our code: "If the receiver is gone, panic (crash the thread)."

                // Once the thread sends the message, we are putting it to sleep to simulate work
                thread::sleep(Duration::from_millis(100));
            }
        });

        // Pushing the handle to the vector so that we can use .join() later to allow all running threads to finish their work
        handles.push(handle);
    }

    // We must drop the original transmitter or the receiver loop never ends
    // The receiver keeps waiting as long as ANY transmitter exists
    // If you don't drop the original tx, the loop waits forever
    drop(tx);

    // Receive and print all messages
    // The receiver (rx) implements the Iterator trait, so you can loop over it just like a vector
    // It calls rx.recv() to get the next message
    // If a message arrives, assigns it to received and runs the loop body
    // If all senders are dropped, the iterator ends and the loop exits
    for received in rx {
        println!("Received: {}", received);
    }

    // The loop automatically ends when all senders (tx) are dropped
    // Timeline:
    // Original tx is dropped with drop(tx)
    // Each thread finishes -> its tx_clone is dropped
    // When the last tx_clone is dropped -> no senders remain
    // rx.recv() returns Err -> loops exits
    
    // The above loop is equivalent to:
        // loop {
        //     match rx.recv() {
        //         Ok(received) => {
        //             println!("Received: {}", received);
        //         }
        //         Err(_) => {
        //             // All senders dropped, exit loop
        //             break;
        //         }
        //     }
        // }

    // .recv() waits until a message arrives
    // Returns Ok(message) of message is received
    // Returns Err(_) if all senders are dropped (no more messages possible)

    // The loop finishes when all senders are dropped 
    // If you don't drop the original tx, it stays in the main thread and the receiver will think "there's a sender out there"
    // and the loop waits forever for messages that never come, so the program hangs

    // You can also do:
        // for _ in 0..9 {
        //     let received = rx.recv().unwrap();
        //     println!("Received: {}", received);
        // }

    // As a note, .recv() BLOCKS (waits) until a message arrives - the thread pauses execution
    // This is different from locks - with channels you wait for DATA, not ACCESS
    
    // When loop calls rx.recv(), it blocks (waits) until a message arrives
    // When the message arrives, loop body runs
    // The loop repeats -> calls rx.recv() again and the process repeats
    // blocking = thread pauses and waits (frozen until message arrives) -> CPU can do other work while thread is blocked
    // In our loop, main thread waits between each message

    // Now we will wait for all threads to finish before moving on
    // If we did not do this, the main thread would continue before the other threads finished
    // When the main thread exits, the entire program exists, regardless of running spawned threads
    // When you iterate over handles and call .join(), you let all threads finish before moving on
    for handle in handles {
        handle.join().unwrap();
    }
}

    // println!("About to receive...");
    // let msg = rx.recv().unwrap();  // ← Thread STOPS HERE and WAITS
    // println!("Received: {}", msg);  // ← Only runs AFTER message arrives

// Main Thread:
// ├─ Prints "About to receive..."
// ├─ Calls rx.recv()
// │  └─ BLOCKS (waits, does nothing, frozen)
// │     ... waiting ...
// │     ... still waiting ...
// │     [Message arrives from another thread!]
// │  └─ Unfreezes, returns the message
// ├─ Prints "Received: ..."
// └─ Continues

// In our example, the main thread is the receiver thread
// The receiver thread can be any thread (main or spawned)
// Only ONE receiver allowed (Single Consumer)
// Multiple senders allowed (Multiple Producer)
// Common pattern: Dedicate receiver thread for background processing

// In Rust, loop is a control-flow construct that runs indefinitely until you explicitly stop it using break or return from inside it
// loop runs infinitely until stopped manually