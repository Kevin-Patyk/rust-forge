// In this example, we are implementing parallel join
// Parallel join takes 2 functions and runs them at the same time then gives you both results back:
    // let (left, right) = parallel_join(
    //     || expensive_computation_1(), // runs in parallel
    //     || expensive_computation_2(), // runs in parallel
    // )
    // Both are done, use both results

// The problem: sequential execution is slow
// Normally, code runs one thing at a time:
    // let left = expensive_computation_1();   // takes 2 seconds
    // let right = expensive_computation_2();  // takes 2 seconds
    // // Total time: 4 seconds

// With parallel join, both run simultaneously:
    // let (left, right) = parallel_join(
    //     || expensive_computation_1(),  // takes 2 seconds
    //     || expensive_computation_2()   // takes 2 seconds
    // );
    // // Total time: 2 seconds (they overlap!)

// The entire point of parallel join is to be able to run 2 functions at the same time instead
// of one after another

use std::thread;

// Below we are designing the parallel_join() function with trait bounds
// The trait bounds are:
    // A: Type of the first closure
    // B: Type of the second closure
    // RA: Return type of closure A
    // RB: Return type of closure B
// We need separate types for everything because the closures can be completely different
fn parallel_join<A, B, RA, RB>(a: A, b: B) -> (RA, RB)
where
    // A is a closure that can be called once, takes no arguments, and returns RA
    // Send: The type can be safely transferred between threads
    A: FnOnce() -> RA + Send + 'static,
    // B is a closure that can be called once, takes no arguments, and returns RB
    // Send: The type can be safely transferred between threads
    B: FnOnce() -> RB + Send + 'static,
    // We are using FnOnce since we only call it once anyway and because it accepts ALL closures
    // which allows for maximum flexibility
        // Closures that move ownership
        // Closures thats mutate
        // Closures that just borrow

    // Send: The type can be safely transferred between threads
    // Essentially, any type that implements Send
    RA: Send + 'static,
    // Send: The type can be safely transferred between threads
    // Essentially, any type that implements Send
    RB: Send + 'static,
    // We need Send since we are sending these things across thread boundaries
    // Most type are Send, essentially any type without raw pointers or thread-local stuff

    // The closure needs Send since the closure object (with all of its captured variables) moves to another thread
    // The return type needs Send since it will be coming back from another thread

    // We need 'static since thread::spawn requires that everything passed to it lives for 'static (the entire program lifetime)
    // This is because:
        // The spawned threads could live for the entire program
        // Rust needs to guarantee the closure won't reference data that gets dropped
    // 'static means that the closure cannot borrow any data with a shorter lifetime

    // For RA and RB, we need 'static so that RA doesn't contain any references to data that 
    // could be dropped while the thread is still running

    // All four 'static bounds are required because of how thread::spawn works internally
{

    // 1. Spawn a new thread to run function a
    let handle = thread::spawn(a);

    // 2. Run function b on the CURRENT thread (no spawning needed)
    let result_b = b();

    // 3. Wait for thread to finish and get result from a
    let result_a = handle.join().expect("Thread panicked");

    // 4. Return both results as a tuple
    (result_a, result_b)

}

// parallel_join() is the fundamental building block for:
    // Using multiple CPU cores effectively
    // Splitting work recursively (divide and conquer)
        // We say recursively since, in divide-and-conquer algorithms, you typically split the problem
        // and then call the same function on each half
    // Running independent tasks simultaneously
    // Reducing wall-clock time (even though total CPU work is the same)

// parallel_join() can also be used to run more than 2 functions simultaneously:
    // parallel_join(
    //     || parallel_join(|| task1(), || task2()),  // parallel_join_1
    //     || parallel_join(|| task3(), || task4())   // parallel_join_2
    // );

// Main parallel_join:
    // Spawns thread A -> runs parallel_join_1
    // Main thread -> runs parallel_join_2
// Inside parallel_join_1 (thread A)
    // Spawns thread C -> runs task1()
    // Thread A -> runs task2()
// Inside parallel_join_2 (main thread)
    // Spawns thread B -> runs task3()
    // Main thread -> runs task4()
// Total threads spawned: 3 (A, B, C)
// Total thread executing: 4 (Main + A + B + C)

// Each parallel_join splits work between a spawned thread and the current thread
// And nesting them creates the tree of parallel execution

fn main() {
}
