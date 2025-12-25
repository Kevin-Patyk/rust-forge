use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

// This program demonstrates:
// - RwLock for allowing multiple readers or one writer
// - Arc for shared ownership across threads
// - The difference between Mutex (one at a time) and RwLock (multiple readers)
// - Proper lock scope management (acquire inside loop, not outside)
// - Lock poisoning and why .unwrap() is needed

fn main() {
    // Creating a counter with RwLock instead of Mutex
    // Mutex - only one thread can access data at a time (read OR write) - even if you want to read, you must lock exclusively
    // RwLock - multiple threads can read simultaneously, but only one thread can write (blocks all readers)
    // Better performance when you have many readers, few writers
    let counter = Arc::new(RwLock::new(0));

    // Create a mutable vector to store handles
    // A handle is a way to interact with a thread
    // When you spawn a thread, you get back a JoinHandle
    // It represents a running thread and allows you to interact with it
    let mut handles = Vec::new();

    // Spawn 3 reader threads
    for i in 0..3 {

        // Clones the Arc -> increments the reference count
        // Creates a new pointer to the same data
        // In each iteration of the loop, the ref count increases each time
        // It is then moved into the thread and the thread owns it
        // As each thread completes, its counter_clone will be dropped
        let counter_clone = Arc::clone(&counter);
        // This only lives in this loop iteration -> move transfers ownership INTO the thread and it can use it as long as it needs
        // Every loop iteration has its own scope, so variables declared inside the loop are dropped at the end of each iteration

        // A handle represents a running or finished thread
        // thread::spawn returns a JoinHandle<T> that represents a thread and allows us to interact with it
        let handle = thread::spawn(move || {
            // We are moving counter_clone into the closure so that the thread now owns counter_clone
            // If we did not do this, it would go out of scope after every for loop iteration
            // With move, the thread can use counter_clone as long as it needs

            // If we acquire the lock before the loop, you hold it for all 5 iterations
            // You are reading the same snapshot of the value
            // Writers can't write because readers are holding the lock the entire time
            // If we don't put it in the loop, then it won't be dropped at each iteration and the thread will hold it the entire time (until the loop finishes print 5 times)
            // It will be released when num goes out of scope after the for loop

            // Now, we are reading the value 5 times
            // Each iteration is acquire lock, read, release, sleep (other threads can run)
            // Always acquire the lock for the MINIMUM time necessary
            // Keep the lock scope as small as possible
            for _ in 0..5 {

                // Here, we are using .read() instead of .lock()
                // The .read() method acquires a READ lock
                // It returns a RwLockReadGuard<i32>
                // This is a smart pointer that derefs to i32
                let num = counter_clone.read().unwrap();
                // This has .unwrap() because .read() can return a Result cause of poisoning (thread panics)
                // A lock becomes poisoned when a thread panics while holding the lock

                // We can now read the value by dereferencing
                // We need to dereference since .read() returns a smart pointer that needs to be dereferenced
                println!("Reader {}: Read value {}.", i, *num);

                // When this for loop iteration ends, the READ lock will be released
                // Which means another thread is elgibile to pick it up
                // To show more interleaving, we will put the thread to sleep for 10 microseconds, allowing other threads to pick it up
                // You can also sleep using from_millis() to show more interleaving
                thread::sleep(Duration::from_micros(10));
            }
        });

        handles.push(handle);
    }

    for i in 0..2 {
        
        // Clones the Arc -> increments the reference count
        // Creates a new pointer to the same data
        // In each iteration of the loop, the ref count increases each time
        // It is then moved into the thread and the thread owns it
        // As each thread completes, its counter_clone will be dropped
        let counter_clone = Arc::clone(&counter);
        // This only lives in this loop iteration -> move transfers ownership INTO the thread and it can use it as long as it needs
       // Every loop iteration has its own scope, so variables declared inside the loop are dropped at the end of each iteration
       
        // A handle represents a running or finished thread
        // thread::spawn returns a JoinHandle<T> that represents a thread and allows us to interact with it
        let handle = thread::spawn(move || {
            // We are moving counter_clone into a closure so that the thread now owns counter_clone
            // If we did not do this, it would go out of scope after every loop iteration
            // With move, the thread can use counter_clone as long as it needs

            for _ in 0..3 {
                // Here, we are using .write() instead of .lock()
                // This allows us to mutate (write to) the data, as opposed to just reading it
                // It returns a RwLockWriteGuard<i32> 
                // This is a smart pointer that derefs to i32
                let mut num = counter_clone.write().unwrap();
                // This has .unwrap() because .write() can return a Result cause of poisoning (thread panics)

                // We can modify the value by dereferncing
                // We need to dereference since .write() returns a smart pointer that needs to be dereferenced
                *num += 1;
                
                println!("Writer {}: Incremented to {}.", i, *num);

                // When this for loop iteration ends, the WRITE lock will be released
                // Which means another thread is elgible to pick it up
                // To show more interleaving, we will put the thread to sleep for 50 microseconds, allowing other threads to pick it up
                thread::sleep(Duration::from_micros(50));
            }
        });

        handles.push(handle);
    }

    // As before, all 5 threads (3 readers + 2 writers) will be running in parallel
    // There are 2 spawning phases (3 readers + 2 writers) and they all spawn very fast
    
    // With Mutex, only 1 thread can hold the lock at a time, even if all threads want to just read
    // With RwLock, multiple readers can hold read locks simultaneously
    // But writers still need exclusive access
    // Rules:
    // Multiple readers can hold the read locks at the same time
    // Only one writer can hold a write lock at a time
    // When a writer is writing, NO readers can read
    // When readers are reading, NO writer can write
    // Better performance than Mutex when you have many reads, few writes

    // Reading is safe to do in parallel since multiple threads reading the same data won't corrupt it 
    // No one is modifying, so everyone sees consistent data
    
    // Writing needs exclusivity 
    // If someone is modifying the data, no one else should read it (might see half-written data)
    // If someone is reading data, no one should modify it (readers might see inconsistent state)

    // Now, we will wait for all threads to finish before moving on
    // If we did not do this, the main thread would continue before the other threads finish
    // When the main thread exits, the entire program exits, even if the spawned threads aren't done
    // When you iterate over handles, you let each thread finish
    for handle in handles {
        // .join() returns a Result because a thread might have panicked
        handle.join().unwrap();
    }

    // Even though all threads are finished, we still need to acquire the READ lock to read the value inside of RwLock
    // You can't just read a RwLock<i32>
    // You also can't simply dereference for the same reason
    // RwLock always protects the data - even if no threads are running RwLock, wraps the data
    // Once RwLock is removed with .read().unwrap(), we need to dereference since the value inside final_result is behind a pointer (reference)
    let final_result = counter.read().unwrap();
    println!("Final value is {}.", *final_result);
}
