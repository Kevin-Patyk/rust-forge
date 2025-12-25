// Problem 44: Scoped Threads

// So far, every time we have used threads with shared data, we have needed Arc
// We would wrap the data in Arc, clone the data (increment the reference count), and move it into the thread   
// This works, but it's verbose and has runtime overhead (reference counting)

// Scoped threads let you borrow data directly from the parent thread's stack
    // let data = vec![1, 2, 3, 4];
    // thread::scope(|s| {
    //     s.spawn(|| {
    //         // Directly borrow &data - no Arc needed!
    //     });
    // });

// How do scoped threads work?
// The key insight: thread::scope guarantees all spawned threads finish before the scope ends
    // let mut data = vec![1, 2, 3];

    // thread::scope(|s| {
    //     s.spawn(|| {
    //         println!("{:?}", data); // Borrow &data
    //     });
    //     s.spawn(|| {
    //         println!("{:?}", data); // Another thread also borrows &data
    //     });
    //     // Both threads MUST finish before this scope ends
    // });

// Rust's lifetime checker knows:
// 1. The scope waits for all threads before returning
// 2. Therefore, data outlives all threads
// 3. Therefore, threads can safely borrow &data

// Why scoped threads matter:
// - Cleaner code: No Arc::clone() noise
// - Better performance: No reference counting overhead
// - Simpler ownership: Just use regular borrows
// - Return values: Easy to collect results from threads
// - Real-world use: Parallel computation over existing data

// The problem is to build parallel computation patterns using scoped threads:
// - Parallel sum: Split a large vector, sum chunks in parallel
// - Parallel search: Find elements matching a predicate across threads
// - Parallel map: Transform data in parallel and collect results
// - Mutable access: Let one thread mutate while others read (safely)

use std::thread;

// data is a slice reference, which lets this function operate on
// arrays, vectors, or any contiguous collection without copying
fn parallel_sum(data: &[i32], num_threads: usize) -> i32 {

    // Calculating the chunk size based on the length of the data and number of threads
    // Note: Integer division means we might create more chunks than num_threads if there's a remainder
    // Example: 10 elements / 4 threads = chunk_size of 2, creating 5 chunks (not exactly 4 threads)
    let chunk_size = (data.len() + num_threads - 1) / num_threads; // cieling division

    // thread::scope creates a scope in which:
    // - All spawned threads are GUARANTEED to finish before the scope ends
    // - Threads can borrow local variables (like &data) without Arc
    // - The compiler proves borrows are safe because threads can't outlive the scope
    // - The scope itself returns a value (whatever we return from the closure)
    thread::scope(|s| {

        // Creating a vector to store handles inside of the scope
        // Handles represent running threads and let us retrieve their return values
        let mut handles = Vec::new();

        // Split data into chunks and spawn one thread per chunk
        // .chunks() creates non-overlapping slices of size chunk_size
        // The last chunk may be smaller if data.len() isn't evenly divisible
        for chunk in data.chunks(chunk_size) {

            // Spawn a scoped thread for this chunk
            // 'move' transfers ownership of chunk into the thread
            // Without 'move', chunk wouldn't live long enough (each loop iteration creates a new chunk reference)
            let handle = s.spawn(move || {
                // Each thread sums its chunk independently
                // This is the thread's return value (implicit return)
                chunk.iter().sum::<i32>()
            });

            // Store the handle so we can collect results later
            handles.push(handle);
        }

        // Wait for all threads to finish and collect their partial sums
        // .into_iter() consumes the handles vector
        // .map(|h| h.join().unwrap()) waits for each thread and extracts its return value
        // .sum() adds up all the partial sums into the final result
        // This entire expression is the implicit return value from thread::scope
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    })
}

// This is a function using generics T and F
// T represents the type of elements in the data slice
// F represents the predicate function type
// data is a slice reference that can be any type T, as long as it implements Clone + Send + Sync
// predicate is a function that takes a reference to T (&T) and returns a bool, and also implements Send + Sync + Copy
// Returns a Vec<T> containing all elements that match the predicate
fn parallel_search<T, F>(data: &[T], num_threads: usize, predicate: F) -> Vec<T>
where 
    // Send: allows T values to be moved between threads safely (transferred ownership)
    // Sync: allows &T references to be shared across multiple threads safely (shared access)
    T: Clone + Send + Sync,
    // Fn(&T) -> bool: the predicate must be callable with &T and return bool
    // Send: the function itself can be moved to another thread
    // Sync: the function can be called from multiple threads simultaneously
    // Copy: allows the predicate to be copied into each thread (avoids move issues in the loop)
    F: Fn(&T) -> bool + Send + Sync + Copy
{   
    // Ceiling division ensures we don't lose any elements due to integer truncation
    let chunk_size = (data.len() + num_threads - 1) / num_threads; // cieling division

    // thread::scope creates a scope where spawned threads are guaranteed to finish before scope ends
    // This allows threads to safely borrow 'data' without Arc
    // In the closure parameter, s is the scope handle
    // You use this handle to spawn threads within the scope
    // It is the scope handle that gives you access to scoped thread spawning
    thread::scope(|s| {

        // Vector to store handles for all spawned threads
        // We need these handles to retrieve each thread's results later
        let mut handles = Vec::new();

        // Each chunk is a borrowed slice - a view into a portion of the original data
        for chunk in data.chunks(chunk_size) {

            // Spawn a scoped thread for this chunk
            // 'move' transfers ownership of chunk into the thread
            // Each thread independently filters its chunk
            let handle = s.spawn(move || {

                // Filter elements in this chunk that match the predicate
                // chunk.iter() produces Iterator<Item = &T>
                // .filter() keeps only elements where predicate returns true
                // |&value| uses a reference pattern to dereference &&T to &T for the predicate
                // .cloned() converts &T to T (owned value) by cloning each element
                // .collect::<Vec<T>>() gathers all matching elements into a Vec
                chunk.iter().filter(|&value| predicate(value)).cloned().collect::<Vec<T>>()
                // We need our thread to return a Vec<T> not an iterator, so this is why we are cloning and collecting into a vector
                // We need to clone since chunk.iter() gives us &T and we need to return a vector of owned values

                // .filter() takes a predicate closure that returns a bool

                // If we just ended with .filter(), it would be returning an iterator and not a vector
                // Iterators can't be sent between threads easily
            });

            // Store the handle so we can retrieve this thread's results later
            handles.push(handle);
        }

        // We will now have a vector of handles
        // We are using .into_iter() to take ownership of the handles vector
        // We will iterate over each handle and apply .flat_map()
        // .flat_map() combines .map() and .flatten() in one step, it is the idiomatic choice
        // This will allow each thread to finish (.join()), get its Vec<T> return value (.unwrap()), 
        // and then flatten the Vec<T> into its individual T elements
        // We will then collect all of the T elements from all threads into a single Vec<T>, giving us the result
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()

    })
}

// This is a function using generics T, U, and F
// T represents the type of elements in the data slice (input type)
// F represents the function type that does the transformation
// U represents the output type - the type we are transforming each element to
// data is a slice reference that can be any type T, as long as it implements Send + Sync
fn parallel_map<T, U, F>(data: &[T], num_threads: usize, func: F) -> Vec<U>
// We are using U in the output type because we are mapping (transforming)
// For parallel_search, we were just filtering, so we were returning the same type (T -> T), 
// whereas mapping can return a different type (T -> U)
// U is necessary when you want to allow transformations like i32 -> String or String -> usize
where
    // Send: allows T values to be moved between threads safely (transferred ownership)
    // Sync: allows &T references to be shared across multiple threads safely (shared access)
    T: Send + Sync,
    U: Send,
    // Fn(&T) -> U: the function must be callable with &T and return U (transforming it)
    // Send: the function itself can be moved to another thread
    // Sync: the function can be called from multiple threads simultaneously
    // Copy: allows the function to be copied into each thread (avoids move issues in the loop)
    F: Fn(&T) -> U + Send + Sync + Copy,
{
    let chunk_size = (data.len() + num_threads - 1) / num_threads;

    // We use s to spawn threads within the scope
    thread::scope(|s| {
        
        // Vector to store handles for all spawned threads
        // We need these handles to retrieve each thread's results later
        let mut handles = Vec::new();

        // Split data into chunks and process each chunk in a separate thread
        // Each chunk is a borrowed slice - a view into a portion of the original data
        for chunk in data.chunks(chunk_size) {
            
            // Spawn a scoped thread for this chunk
            // 'move' transfers ownership of chunk into the thread
            // Each thread independently maps (transforms) its chunk
            let handle = s.spawn(move || {

                // Transform each element in this chunk using the provided function
                // chunk.iter() produces Iterator<Item = &T>
                // .map(|value| func(value)) calls func on each &T, producing U
                // .collect::<Vec<U>>() gathers all transformed elements into a Vec
                //
                // We do NOT need .cloned() here because:
                // - func already returns owned values (U), not references
                // - In parallel_search, .filter() kept the &T references, so we needed .cloned()
                // - In parallel_map, func produces new U values, already owned
                chunk.iter().map(|value| func(value)).collect::<Vec<U>>()

                // .map() takes a function that transforms each element

                // If we just ended with .map(), it would be returning an iterator and not a vector
                // Iterators can't be sent between threads easily
            });

            // Store the handle so we can retrieve this thread's results later
            handles.push(handle);
        }

        // We will now have a vector of handles
        // We are using .into_iter() to take ownership of the handles vector
        // We will iterate over each handle and apply .flat_map()
        // .flat_map() combines .map() and .flatten() in one step, it is the idiomatic choice
        // This will allow each thread to finish (.join()), get its Vec<T> return value (.unwrap()), 
        // and then flatten the Vec<T> into its individual T elements
        // We will then collect all of the T elements from all threads into a single Vec<T>, giving us the result
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect() // handles is still inside of the scoped thread closure, so we can use it
    })
}

fn main() {
    println!("=== Test 1: parallel_sum ===");
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let sum = parallel_sum(&numbers, 4);
    println!("Sum of {:?}: {}", numbers, sum);
    println!("Expected: 55\n");

    println!("=== Test 2: parallel_search ===");
    let ages = vec![15, 22, 18, 35, 42, 19, 50, 28, 33];
    let adults = parallel_search(&ages, 3, |age| *age >= 18);
    println!("Ages: {:?}", ages);
    println!("Adults (>= 18): {:?}", adults);
    println!("Expected: [22, 18, 35, 42, 19, 50, 28, 33]\n");

    println!("=== Test 3: parallel_map ===");
    let words = vec!["rust", "parallel", "scoped", "threads"];
    let uppercase = parallel_map(&words, 2, |word| word.to_uppercase());
    println!("Original: {:?}", words);
    println!("Uppercase: {:?}", uppercase);
    println!("Expected: [\"RUST\", \"PARALLEL\", \"SCOPED\", \"THREADS\"]\n");
}

// Random notes:

// What we have been working on is concurrency (and potentially parallelism when those threads actually run simultaneously on multiple cores)

// Threading = Concurrency mechanism
// - Creating multiple threads to handle tasks concurrently
// - Threads can run in parallel if you have multiple CPU cores

// Working stealing = Concurrency optimization
// - A scheduling technique where idle threads "steal" work from busy threads
// - Helps balance load across concurrent workers
// - Common in thread pools and async runtimes

// Mutex = Concurrency synchronization
// - Protects shared data from race conditions when multiple threads access it
// - Ensures only one thread can access protected data at a time
// - Essential for safe concurrent programming

// The full picture:
// - You're writing concurrent code (multiple threads making progress)
// - It becomes parallel when threads actually run simultaneously on different cores
// - You use synchronization primitives (Mutex, channels, etc.) to coordinate the concurrent threads safely

// You write the concurrent structure (threads, tasks)
// The OS/runtime handles the parallel execution (scheduling on cores)
// You can't directly control which core runs which thread (that's the OS's job)

// What you CAN control:

// Number of threads (often set to number of CPU cores)
// How work is divided among threads
// Synchronization between threads