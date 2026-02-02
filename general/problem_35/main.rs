// Recursion is when a function calls itself to solve a problem by breaking it down into smaller, similar subproblems.
// Each recursive call works on a simpler version of the original problem until it reaches a case simple enough to solve directly.

// Every recursive function needs two essential parts:

// 1. Base Case(s):
// - The stopping condition that prevents infinite recursion
// - Move towards the base with each call
// - Combines the result of the recursive call to build the final answer

// 2. Recursive Case(s):
// - Calls the function again with a simpler/smaller input
// - Moves towards the base case with each call
// - Combines the result of the recursive call to build the final answer

// How recrusion works:
// - Each function call gets its own space on the call stack
// - The call stack grows as recursive calls are made
// When a base case is reached, calls start returning and the stack unwinds
// Results build up as the stack unwinds (from the inside out)

// Key tips:
// - Always identify the base case first - ask yourself "when should this stop?"
// - Make sure you're moving toward the base case - each recursive call should work with smaller/simpler input
// - Use `match` to destructure enums and handle different cases - pattern matching is your friend in Rust
// - Remember that recursive calls build results from the inside out - the deepest call completes first
// - Trust the recursion - assume the recursive call works correctly for smaller inputs
// - Use `Box` for recursive types (like trees) to avoid infinite size at compile time

// Problem 1: Factorial Calculate -----

// Calculate the factorial of a number `n`
// For example, factorial(5) should return 120

// Base case is when value is 0 or 1

fn factorial_calculate(value: u128) -> u128 {
    match value {
        // When the function hits this case, it just returns 1 since 1! and 0! are both == 1
        // This is the simplest problem we can solve without breaking it down further
        0 | 1 => {
            1
        }
        // For any other case, we need to recurse further
        _ => {
            value * factorial_calculate(value - 1) 
        }
    }
}

// Problem 2: Fibonnaci Sequence

// The sequence starts: 0, 1, 1, 2, 3, 5, 8

// This has 2 base cases: 0 and 1

// The input value is the position (or index) in the Fibonacci sequence
// The return value is the actual Fibonnaci number at that position
fn fibonacci_calculate(value: u128) -> u128 {
    match value {
        0 => {
            0
        }
        1 => {
            1
        }
        // For any other case, recurse further
        _ => {
            fibonacci_calculate(value-1) + fibonacci_calculate(value-2)
        }
    }
}

// Problem 3: Vector Sum -----

// Get the sum of all numbers in a vector recursively (no using `.iter().sum()`)

// The base case is 0

fn vector_sum(vec: Vec<u128>) -> u128 {
    match vec.len() {
        0 => {
            0
        }
        // Recursive case
        n => {
            // 0..n-1 is not inclusive, so if n is currently 5, it will take 0..4, which is 0,1,2,3
            // Alternative syntax is ..n-1, which is exclusive of the end, so it goes from 0 up to (but not including) n-1
            // The last index of a vector is always length - 1 due to zero indexing
            // This means that, if you do vec[length], you will always get an index out of bounds panic
            vec[n-1] + vector_sum(vec[0..n-1].to_vec())
        }
    }
}

// For each call of `vector_sum()`, get the length of the vector
// If the vector length is 0, we return 0
// If the vector length is greater than 0, we bind the length to `n`
// We then take the value at index `n-1` (the last element) and add it to the next value that `vector_sum()` produces, but for the reduced range
// meaning not including the current last element (everything from 0 to n-2)

// Problem 4: Count down -----

// Print numbers down to 1, then print "Done!"
// Base case is 0

fn count_down(value: i32) -> () {
    match value {
        // Base case 
        0 => {
            println!("Done!");
        }
        // Recursive case
        n => {
            println!("{}", n);
            count_down(n - 1);
        }   
    }
}

fn main() {
    let test1 = factorial_calculate(5);
    println!("{}", test1);

    let test2 = fibonacci_calculate(10);
    println!("{}", test2);

    let test3 = vector_sum(vec![1,2,3,4,5]);
    println!("{}", test3);

    count_down(3);
}
