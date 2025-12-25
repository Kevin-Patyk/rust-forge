// In this problem, we will cover producer-consumer with bounded channels

// We will learn about mpsc::sync_channel (bounded channels) and how they provide backpressure to prevent prodcucers from overwhelming consumers

// What we will learn:
// - Bounded versus unbounded channels
// - Backpressure (slowing down fast producers)
// - Producer-consume pattern with flow control
// - Why bounded channels matter in production

// The problem with unbounded channels (what we have used before)
    
    // let (tx, rx) = mpsc::channel();

    // Fast producer
    // for i in 0..1_000_000 {
    //     tx.send(i).unwrap(); // Never blocks! 💀
    // }

// All 1 million items queued in memory immediately
// Could easily run out of memory 

// With a bounded channel (what we will learn):
    
    // let (tx, rx) = mpsc::sync_channel(10); // Buffer of 10
    
    // Fast producer
    // for i in 0..1_000_000 {
    //     tx.send(i).unwrap(); // Blocks when buffer is full! ✅
    // }

// Producer automatically slows down to match consumer speed

// The scenario is that we are building a log processing system
// - Producer: Generates logs quickly (100ms each)
// - Consumer: Processes logs slowly (500ms each)
// - Problem: Producer is 5x faster than consumer

// Without backpressure: Producer generates 1000 logs -> all get queued immediately -> memory explosion
// With backpressure: Producer slows down when the buffer is full -> controlled memory usage

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct LogEntry {
    id: usize,
    message: String,
    timestamp: Instant,
}

fn main() {
    
    // Creating a bounded channel with a capacity of 5
    // Capacity = the maximum number of items that can be stored in the buffer at once
    // The buffer is the storage space for queued items
    // Think of it like a waiting room with limited seats
    // With a capacity of 5, the buffer can hold 5 items max
    // When a producer sends an item, it gets stored in the buffer and a slot gets occupied
    // Producer: send(Item1)
    // Buffer: [Item1] [Empty] [Empty] [Empty] [Empty]
    // When the buffer is full, the sender is blocked (waits)
    let (tx, rx) = mpsc::sync_channel::<LogEntry>(5); // Hold up to 5 items in the buffer before blocking the sender

    // Creating a bounded channel with a capacity of 5
    // Capacity = the maximum number of items that can be stored in the buffer at once

    // How it works:
    // Buffer: [Empty] [Empty] [Empty] [Empty] [Empty]  <- 5 slots
    // 
    // Producer sends 5 logs:
    // Buffer: [Log0] [Log1] [Log2] [Log3] [Log4]  <- FULL!
    //
    // Producer tries to send Log5:
    // ❌ BLOCKED! (waits until consumer frees a slot)
    //
    // Consumer receives Log0:
    // Buffer: [Empty] [Log1] [Log2] [Log3] [Log4]  <- 1 slot free
    // ✅ Producer unblocks, sends Log5
    //
    // This automatic blocking/unblocking is called BACKPRESSURE


    // This is a point in time (like taking a snapshot of a clock)
    // It is for measuring elapsed time
    // What time is it right now?
    // Think of it like pressing "Start" on a stopwatch
    let start = Instant::now();

    // Spawn a producer thread (fast - generates 20 logs very quickly)
    let producer = thread::spawn(move || {
        println!("Producer: Starting...");

        for i in 0..20 {
            let log = LogEntry {
                id: i,
                message: format!("Log entry {}", i),
                timestamp: Instant::now(),
            };

            println!("Producer: Sending log {} (buffer might be full, may block...)", i);

            // Sending the log from this transmitter to the receiver
            // We need to call .unwrap() since .send() returns a Result if the receiver is dropped
            // IMPORTANT: .send() BLOCKS if buffer is full!
            // - If buffer has space: returns immediately (Ok)
            // - If buffer is full: thread SLEEPS until space available
            // - This is different from unbounded channel (mpsc::channel()) which never blocks
            tx.send(log).unwrap();

            // Since we only have one transmitter in this example and it is in this thread, it will be dropped when the thread finishes its work
            // This is why we do not need to drop the transmitter (using drop(tx)); manually later after the threads finish
            // In other examples, we had the original tx and multiple cloned tx, so we had to make sure to drop the original so the rx knew to stop receiving

            // When producer thread ends:
            // 1. tx goes out of scope
            // 2. tx is automatically dropped
            // 3. Channel closes (no more senders)
            // 4. rx.recv() returns Err
            // 5. Consumer loop breaks

            println!("Producer: Log {} sent successfully", i);

            // Simulate log generation time (fast - 100ms)
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Spawn a consumer thread (slow - processes logs slowly)
    let consumer = thread::spawn(move || {
        println!("Consumer: Starting...\n");

        // This will hold the number of processed logs for this specific thread
        let mut processed = 0;

        // We are using an explicit loop for learning purposes to show what is happening
        // The receiving thread does not know how many messages it will receive, that is why we are not using a for loop
        // This loop will continue executing until it reaches the error variant, which means the transmitter is dropped
        loop {
            // .recv() BLOCKS (waits) until a message arrives
            // This is different from .send() blocking:
            // - .send() blocks when buffer FULL (producer waits for space)
            // - .recv() blocks (waits) when buffer is empty (consumer waits for data)
            // Both are blocking operations that put the thread to sleep

            // When we say .recv() blocks when buffer is empty, we mean:
            // - The consumer thread waits (sleeps) until a message arrives
            // - Not that it stops receiving
            // - The thread is paused until the producer sends something
            // - It will resume when data arrives
            match rx.recv() {
                Ok(log) => {
                    println!("  Consumer: Processing log {}", log.id);

                    // Simulate slow processing (500ms)
                    thread::sleep(Duration::from_millis(500));

                    println!("  Consumer: Finished log {}", log.id);

                    processed += 1;
                }
                Err(_) => {
                    break;
                }
            }
        }

        println!("\nConsumer: Finished processing {} logs", processed);

        processed

    });

    // Now, wait for both threads to finish
    producer.join().unwrap();

    // Since the consumer thread produces a variable (processed), we want to assign it to a variable to store the result
    let total_processed = consumer.join().unwrap();

    // After calling Instant::now(), we want to calculate the elapsed time
    // How much time has passed since start?
    // Returns a duration (the time difference)
    // Think of it like pressing "Stop" on a stopwatch
    let elapsed = start.elapsed();

    // We are using this to measure how long the program took from start to finish
    // We call let start = Instant::now(); before creating the threads and doing the work
    // We call let elapsed = start.elapsed(); after allowing the threads to finish all of the work
    // This will give us the duration for how long it took to do all of the work

    // 1. Press "Start" -> Instant::now()
    // 2. Do work
    // 3. Press "Stop" -> start.elapsed()
    // 4. See the time - Duration

    println!("\n=== Results ===");
    println!("Total processed: {}", total_processed);
    println!("Total time: {:.2}s", elapsed.as_secs_f64());
    println!("\nNote: Producer was throttled by bounded channel!");
    println!("Without backpressure, all 20 logs would queue immediately.");
}

// Expected behavior with capacity=5 and this timing:
// - Producer generates logs every 100ms (fast)
// - Consumer processes logs every 500ms (slow - 5x slower)
// - Producer will fill buffer (5 logs) in ~500ms
// - Then producer BLOCKS waiting for consumer to free space
// - Result: Producer is throttled to consumer's pace
// - Total time: ~10 seconds (limited by consumer, not producer)