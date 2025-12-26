// Concurrency -> multiple tasks making progress during overlapping time periods
// "Dealing with many things at once"
// Tasks can start, pause, and resume - they don't necessarily run simultaneously
// Example: A chef preparing multiple dishes -> they switch between chopping vegetables, stirring a pot, and checking the oven
// Only one task happens at any instance, but all are progressing

// Parallelism -> multiple tasks running at the exact same moment
// "Doing many things at once"
// Requires multiple CPU cores - tasks execute simultaneously
// Example: Multiple chefs preparing their own dish at the same time

// Concurrency is about structure (how you organize code) while parallelism is about execution (what the hardware does)

// Multi-threading
// A thread is an independent sequence of execution within a program
// It is the smallest unit of execution that can be scheduled by the OS
// Threads within the same process share memory but have their own stack

// Single-threaded:
// Sequential (one after another): Task A -> Task B -> Task C

// Multi-threaded: 
// Thread 1: Task A
// Thread 2: Task B
// Thread 3: Task C
// (they can run concurrently/in parallel)

// Benefits of multi-threading:
// Utilize multiple CPU cores
// Keep UI responsive while doing background work
// Resource sharing: Threads share memory, making communication easier than separate processes

// Challenges:
// Race conditions: multiple threads accessing shared data simultaneously
// Deadlocks: threads waiting for each other indefinitely
// Data races: Undefined behavior when accessing shared mutable data
// Debugging complexity: Hard to reproduce behavior

// In this problem, we are creating a program where multiple threads increment a shared counter
// This will teach us the fundamentals of Arc<Mutex<T>>

use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

fn main() {
    // Arc = Atomic reference count - it allows multiple ownership
    // This is multiple ownership across threads - in a multi-threaded context
    // Allows immutable access
    // Rc is reference count in a single-threaded context
    
    // Mutex allows only one thread at a time to access some data - protects data from concurrent access - prevents data races
    // Mutex allows for the modification of data in a multi-threaded context
    // Mutex = mutual exclusion
    // RefCell is allows for modification of data in a single-threaded context
    let counter: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
    // Together, Arc<Mutex>> allows for shared ownership + mutation

    // When you spawn a thread with std::thread::spawn(), it returns a JoinHandle
    // JoinHandle<T> where T is the return type of the thread's closure
    // It allows you to wait for the thread to finish and get its result

    // When we spawn multiple threads, you need to keep track of them so you can wait for all of them to complete before continuing
    // You need the main thread to wait for all the child threads to complete before continuing
    // This is a convenient way to manage multiple threads
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    // Spawn 5 threads
    // Each thread will increment the counter 1000 times
    for _ in 0..5 {
        // Clones the arc - increments the reference count
        // Each thread needs its own clone pointing to the same data
        let counter_clone = Arc::clone(&counter);

        // A handle represents a running (or finished) thread 
        // You use it to interact with a spawned thread - most importantly, wait for it to finish
        // When you spawn a thread, you get back JoinHandle<T>
        // JoinHandle is not copyable - you must move it or store it somewhere
        let handle = thread::spawn(move || {
            // We are using move so the closure takes ownership of counter_clone
            // Required because the closure needs to outlive the current scope
            // The spawned thread might run after the loop iteration ends
            // Without move, the closure would try to borrow counter_clone 
            // but counter_clone goes out of scope at the end of each loop iteration

            // The threads lifetime is independnet - it keeps running after the loop iteration ends
            // move transfers ownership from the loop scope into the thread's scope
            // Now the thread owns counter_clone and can use it as long as it needs

            // Each thread will increment the counter 1000 times
            for _ in 0..1000 {
                // Lock the mutex to get access to the data
                // .lock() returns Result<MutexGuard<T>, PoisonError>
                // MutexGuard is a smart pointer that derefs to T
                // Lock mutex to get exclusive mutable access to the data for the current thread
                // .lock() returns a Result because the lock could be "poisoned" if a thread panicked while holding the lock
                let mut num = counter_clone.lock().unwrap();

                // Dereference and increment
                // Dereference and modify
                *num += 1;

                // Mutex automatically unlocks when num goes out of scope
                // Lock automatically releases when num goes out of scope
                // num goes out of scope when the loop ends - it releases after each increment, not when the entire thread is done
                // Lock is acquired and released 1000 times - other threads can interleave and get the lock between iterations
                // In a loop, each iteration has its own scope

                // All 5 threads are spawned and running in parallel
                // Each thread is doing a for loop 1000 times
                // Only ONE thread can hold the lock at a time
                // After each increment, the lock drops
                // Another thread can immediately get the lock

                // Imagine 5 people (threads) who all need to speak into the same microphone (counter)
                // Only 1 person can hold the microphone at a time
                // After saying 1 word (incrementing once), they put the microphone down
                // Another person immediately grabs it and says their word
                // This happens very fast - thousands of times per second
                // Eventually everyone gets to say all their words - complete all iterations
            }
        });

        // Store the handle so we can join later
        handles.push(handle);
    }

    // At this point: All 5 threads are already running parallel
    // All threads start working immediately when spawned
    // They work in parallel (at the same time)

    // Wait for all threads to finish/complete
    // You need the vector of handles to wait for all threads to finish before using the result
    // Without it, your program has a race condition where the main thread reads the counter before the worker threads finish incrementing it
    // The main thread is the thread that runs your main() function - it is automatically created when the program starts - we don't spawn it - it's already running
    // Main thread = the original thread that starts when the program runs
    // Spawned threads = child threads that you create with thread::spawn()
    // When the main thread exits, the entire program exits (even if spawned threads aren't done)
    // That's why we need .join() - to make the main thread wait for spawned threads
    // when you iterate over the vector of handles and call .join(), you wait for each thread to finish (they are already running in parallel)
    for handle in handles {
        // .join() blocks until the thread completes
        // .join() waits for a thread that's already running to finish
        // .join() makes the main thread wait for each spawned thread to finish
        handle.join().unwrap();
    }

    // Lock the mutex one final time to read the value
    let final_value = counter.lock().unwrap();
    println!("Final counter value: {}", *final_value);
}

// 1. We make a thread-safe counter that can be owned by multiple objects and mutated
// 2. We make an empty vector to store all of the handles
// 3. When we want to spawn a thread, we clone the counter
// 4. Then, we spawn a thread and inside define the work it does
// 5. Each thread uses .lock() to get exclusive access to modify the counter
// 6. When the variable we assigned .lock() to goes out of scope, the lock releases
// 7. Then we push the handle to the vector of handles
// 8. We then iterate through the handles and call .join() on each - .join() blocks the main thread until that spawned thread completes

// Think of .join() like checking completion tickets
// 1. Spawning phase - you send all 5 workers to do their jobs simultaneously
// 2. All the workers start working right away in parallel 
// 3. Join phase - you stand at the finish line and collect completion tickets 
    // Worker 1's ticket (wait if they haven't arrived)
    // Worker 2's ticket (wait if they haven't arrived)
    // ...
// If worker 2 arrives before worker 1, they have to wait at the finish line
// When you get to worker 2's ticket, you collect it instantly because they're done
// The threads are all working in parallel from the moment they're spawned - .join() just makes the main thread wait for each one to be done