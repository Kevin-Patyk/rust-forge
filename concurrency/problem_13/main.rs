// In this problem, we will learn about map-reduce patterns and build a simple parallel iterator 
// that splits work across threads and combines results

// What is Map-Reduce?
// It is a programming model for processing large datasets in parallel
// 1. Map: Transform each element independently (parallelizable)
// 2. Reduce: Combine all results into a single value (aggregation)

    // Input:  [1, 2, 3, 4, 5, 6, 7, 8]
    //          ↓
    // Map:    [2, 4, 6, 8, 10, 12, 14, 16]  (each element * 2, done in parallel)
    //          ↓
    // Reduce: 72  (sum all elements)

// Why map-reduce matters
// - Embarassingly parallel: Map operations don't depend on each other
// - Scalable: Split work across any number of workers
// - Foundation of data processing: Hadoop, Spark, Rayon all use this pattern
// - After work stealing, this is the next step toward understanding Rayon

// The Problem is to build a parallel map-reduce system where:
// 1. Split input data into chunks (one per worker)
// 2. Each worker independently maps its chunk
// 3. Workers send mapped results through channels
// 4. Main thread reduces (combines) all results

// Example: Given a vector of numbers, compute the sum of squares in parallel

    // Input: [1, 2, 3, 4, 5, 6, 7, 8]

    // Split into 4 chunks:
    // - Worker 0: [1, 2]
    // - Worker 1: [3, 4]  
    // - Worker 2: [5, 6]
    // - Worker 3: [7, 8]

    // Map (square each):
    // - Worker 0: [1, 4]    → local sum = 5
    // - Worker 1: [9, 16]   → local sum = 25
    // - Worker 2: [25, 36]  → local sum = 61
    // - Worker 3: [49, 64]  → local sum = 113

    // Reduce (sum the sums):
    // 5 + 25 + 61 + 113 = 204

// What we will learn:
// 1. Chunking data: Split a Vec into equal parts using .chunks()
// 2. Parallel mapping: Each worker processes its chunk independently
// 3. Channel-based reduce: Collect partial results via mpsc
// 4. Generic functions: Write parallel_map_reduce<T, R, M, F> that works with any types
// 5. Closures as parameters: Pass mapping and reducing functions

use std::thread::{self, JoinHandle};
use std::sync::mpsc;

// This is a function with generic type parameters
// T: The type of elements in your input data
// R: The type that mapping produces (and final result)
// M: The type of mapping function
// F: The type of reduce function
fn parallel_map_reduce<T, R, M, F>(
    data: Vec<T>, // Input data - vec of T elements
    num_workers: usize, // How many threads to spawn
    map_fn: M, // Function to apply to each element
    reduce_fn: F, // Function to combine results
) -> R // Returns a single R value
where // The trait bounds
    // Send = type can safely be transferred between threads
    // 'static = doesn't borrow anything with a limited lifetime
    T: Send + 'static + Clone,
    R: Send + 'static,
    // Fn(&T) -> R = a callable that takes a reference to T and returns R
    // Sync = multiple threads can call this function simultaneously
    // Clone = we can make copies of the function
    M: Fn(&T) -> R + Send + Sync + 'static + Clone,
    // Fn(R, R) -> R = callable that takes two R values and returns one R value
    // No Send/Sync/Clone needed since reduce happens on the main thread only
    F: Fn(R, R) -> R + Send + Copy + 'static,

    // So, our function:
    // 1. The input data (T) can be a vector of any type that implements Send + 'static + Clone
    // 2. The output (R) can be any type that implements Send + 'static
    // 3. The map function (M) must take &T and return R plus implememnt Send + Sync + 'static + Clone
    // 4. The reduce function (F) must takes two Rs and return one R plus implement Send + Copy + 'static

{   
    // We want the chunk size to be the length of the data / number of workers
    let chunk_size = data.len().div_ceil(num_workers);
    
    // We will be using .chunks() to split the data
    // .chunks(n) splits the data into fixed-size groups
    // .chunks() borrows data, so you can't move data into threads
    // But each thread needs to own its chunk of data
    let chunks: Vec<Vec<T>> = data.chunks(chunk_size)
        .map(|slice| slice.to_vec())
        .collect();
    // We are using .to_vec() since &[T] (result of .chunks()) is a borrowed slice (doesn't own the data)
    // .to_vec() creates a new Vec<T> by cloning all elements
    // Now we have owned data that can be moved into threads
    // The result of this is a Vector of Vec<T>, like [[1, 2], [3, 4]]

    // Creating a sender and a receiver
    let (tx, rx) = mpsc::channel();

    // Creating a vector to store handles
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    // We are looping over the chunks to spawn chunks number of worker threads
    // Rather than iterating over a range and creating that number of threads
    for chunk in chunks { // This consumes chunks, moves each Vec<T>, so no cloning needed
        // We could clone the chunks Vector, but it is unnecessary and we would be paying a clone cost
        
        // Clone map_fn for this thread (each thread needs its own copy)
        // This is why we need the clone Trait
        let map_fn_clone = map_fn.clone();

        // Clone tx for this thread (each thread needs its own copy)
        let tx_clone = tx.clone();

        // Here, we are spawning a thread and assigning it to the handle variable
        // A handle lets us interact with a spawned thread
        let handle = thread::spawn(move || {

            // MAP PHASE

            // Variable to store the mapped results
            let mut partial_results = Vec::new();

            // The for loop expression determines what type you get
                // for element in chunk { ... }    // element: T (owns each element)
                // for element in &chunk { ... }   // element: &T (borrows each element)
                // for element in &mut chunk { ... } // element: &mut T (mutable borrow)
            // We choose based on what the FUNCTION needs
            for element in &chunk { // Borrows elements from chunk
                let result = map_fn_clone(element); // map_fn takes &T
                partial_results.push(result);
            }

            // LOCAL REDUCE PHASE
            // Combine all results in THIS chunk into one value

            // We will use .reduce() since it takes a closure with 2 parameters (both the same type) and returns 1 value
            //  Signature: Fn(T, T) -> T
            let partial_result = partial_results.into_iter().reduce(|acc, r| reduce_fn(acc, r)).unwrap();
            // .reduce() is an iterator method that combines all elements into a single value
            // .reduce(|accumulator, element| /* combine them */)
            // It:
            // 1. Takes the first 2 elements
            // 2. Combines them using your closure
            // 3. Takes that result and combines it with the next element
            // 4. Repeats until all elements are processed
            // We need .unwrap() since .reduce() returns Option<R>
            // .unwrap() extracts the value, panics if empty (assumes you always have chunk data)
            // Alternative to .reduce() is .fold(), but .fold() lets you provide an initial value
            // .reduce() uses the first element as the accumulator, not a custom one you provide

            // Send partial result back to the main thread
            tx_clone.send(partial_result).unwrap();

            // As a note, we do not need to return partial_result
            // In this case, it is redundant because we are sending it through .send() and the main thread is receiving it through .recv()
            // We would return it if we didn't use a channel
            // Then we can return it by using a for loop and storing the result of handle.join().unwrap() in a variable
                // partial_result

        });
        
        handles.push(handle); // Store the handle
    }
    
    // Drop the original tx so rx knows when all workers are done
    drop(tx);
    
    // FINAL REDUCE PHASE

    // Collect all partial results from workers and combine them
    let mut final_results = Vec::new();

    // Pushing all of the partial results into the same Vector
    // We are getting all of the partial results from the thread through .send()
    // We are using a for loop instead of indefinite loop because the match and break if Err is happening implicitly
    // because rx implements Iterator - it automatically breaks when all senders are dropped
    for partial_result in rx {
        final_results.push(partial_result);
    }
    
    // Reduce all partial results into final answer
    final_results.into_iter().reduce(|acc, r| reduce_fn(acc, r)).unwrap()
}

// So, for our function, the steps are:
// 1. Determine the chunk size using the length of the data and number of workers
// 2. Create a Vector of Vec<T> -> the caveat is that we need to use .to_vec() on each chunk so we own the data
// 3. Create a handles vector and spawn (tx, rx)
// 4. Iterate over the chunks vector with a for loop, consuming the chunk
// 5. Cloning our mapping function and transmitter to move into the spawned threads
// 6. Spawning a thread:
    // a. The thread loops over each element in the chunk in a for loop and applies the mapping function
    // b. After each element of the chunk is mapped and stored in an intermediate vector, we combine the results locally (in the thread) using .reduce()
    // c. We then have the thread send the partial result through the channel
// 7. Push the handle to the handles vector
// 8. We then drop the original transmitter so that the receiver (main thread) knows to stop receiving
// 9. Start the final reduction phase, which entails collecting all partial results from the channel into an intermediate vector
// 10. The final result is then acquired through doing one last reduction on the intermediate vector combining all partial results

fn main() {
    // Test 1: Sum of squares
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    
    let result = parallel_map_reduce(
        data,
        4,  // 4 workers
        |x| x * x,  // Map: square each number
        |a, b| a + b,  // Reduce: sum them
    );
    
    println!("Sum of squares: {}", result);
    // Expected: 1 + 4 + 9 + 16 + 25 + 36 + 49 + 64 = 204
    
    
    // Test 2: Product
    let data2 = vec![1, 2, 3, 4, 5];
    
    let product = parallel_map_reduce(
        data2,
        2,
        |x| *x,  // Map: identity (just return the value)
        |a, b| a * b,  // Reduce: multiply
    );
    
    println!("Product: {}", product);
    // Expected: 1 * 2 * 3 * 4 * 5 = 120
    
    
    // Test 3: String concatenation
    let words = vec!["Hello", "parallel", "map", "reduce"];
    
    let sentence = parallel_map_reduce(
        words.iter().map(|s| s.to_string()).collect(),
        2,
        |s| s.to_uppercase(),  // Map: uppercase each word
        |a, b| format!("{} {}", a, b),  // Reduce: concatenate with space
    );
    
    println!("Sentence: {}", sentence);
    // Expected: "HELLO PARALLEL MAP REDUCE"
    
    
    // Test 4: Large dataset (see the parallelism!)
    let big_data: Vec<i32> = (1..=1000).collect();
    
    let sum = parallel_map_reduce(
        big_data,
        8,
        |x| x * 2,  // Map: double each
        |a, b| a + b,  // Reduce: sum
    );
    
    println!("Sum of doubled 1-1000: {}", sum);
    // Expected: 2 * (1+2+...+1000) = 2 * 500500 = 1001000
}

// The ultimate goal of map-reduce is that each thread does it's own mapping and initial reduction
// Then all of the results are combined together in the final stage