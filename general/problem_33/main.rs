#![allow(dead_code)]

// General Iterator Notes -----

// What is an iterator?

// An iterator is an object that lets you traverse through a collection of items, one at a time
// Think of it like a bookmark that moves through the book or a cursor that points to elements in a list

/*
In Rust, an iterator is anything that implements the iterator trait, which looks like:

trait Iterator {
    type Item; // The type of thing we are iterating over
    // This represents the type of values that the iterator produces when you call .next() on it

    fn next(&mut self) -> Option<Self::Item>; // How to get the next item
}
*/

// The entire concept boils down to: "Give me the next item or tell me you're done"

// A simple mental model:
// Vec: [1, 2, 3, 4, 5]
// Iterator: "I am currently pointing at 1"
// Call .next() -> Returns Some(1), moves pointer forward: "Now I'm pointing at 2"
// Call .next() -> Returns Some(2), moves pointer forward: "Now I'm pointing at 3"
// Keeps going until we are at the end
// Call .next() -> Returns None (no more items)

// Why Option?
// The next() method returns Option because:
// - Some(value) means: "Here's the next item"
// - None means: "I'm done, no more items"
// This lets you know when to stop iterating

// Three ways to iterate in Rust:
// 1. .into_iter() - consumes the collection, gives you owned values
// 2. .iter() - borrows the collection, gives you references
// 3. .iter_mut() - mutably borrows, gives you mutable references

// Iterators are lazy 
// This is crucial to understand - iterators do nothing until you consume them 

// Iterator Adapters vs Consumers

// Adapters transform an iterator into another iterator (lazy)
// .map(|x| x * 2) - transform each element
// .filter(|x| x > 5) - keep only some elements
// .take(3) - take first 3 elements
// .skip(2) - skip first 2 elements

// Consumers actually execute the iterator and produce a result
// .collect() - gather into a collection
// .count() - count the items
// .sum() - add them all up
// .for_each(|x| println!("{}", x)) - do something with each
// for loop is also a consumer

// Why iterators?
// 1. Composability: Chain operations together cleanly
// 2. Efficiency: No intermediate collections (lazy evaluation)
// 3. Expressiveness: Clear what you are trying to do
// 4. Safety: No off-by-one errors, no manual index management

// Problem 1: Basic Custom Iterator -----

// Let's start with the simplest possible iterator - one that counts up

// A counter that goes from start to end
// This struct holds the iterators state 
// We always need some type to implement Iterator on
// The iterator needs memory since it has to remember:
// - What number did I just return?
// - What's my ending point?
// The memory has to live somewhere -> struct fields
// Even built in iterators have structs with state
struct Counter {
    current: i32, // Where am I now?
    end: i32, // When should I stop?
}

// Every iterators follows this pattern:
// 1. Define a struct to hold state 
// 2. Provide a way to create it (how to initialize)
// 3. Implement Iterator on that struct

impl Counter {
    fn new(start: i32, end: i32) -> Self {
        Self {
            current: start,
            end,
        }
    }
}

impl Iterator for Counter {
    type Item = i32; // Defines what the iterator yields

    // next() takes &mut self since it needs a self to mutate 
    // This self is the instance of the struct
    fn next(&mut self) -> Option<Self::Item> { // How to get the next item
        // next() method defines the logic for producing each item in the sequence 
        // What should I return right now and how do I prepare for the next call?
        // 2 jobs: Figure out what to return and update internal state so the next call returns the right thing
        if self.current < self.end {
            // We save the value before incrementing, otherwise we skip returning the first value in the sequence
            let value = self.current;
            self.current += 1;
            Some(value)
        } else {
            None
        }
    }
    // The general pattern is:
    // 1. Save what you want to return
    // 2. Update the state for next call
    // 3. Return the saved value
}

// Problem 2: Another Basic Custom Iterator -----

// 1. Create a struct the store the state
struct StepBy {
    current: i32,
    end: i32,
    step: i32,
}

// 2. Provide a way to initialize 
impl StepBy {
    fn new(start: i32, end: i32, step: i32) -> Self {
        Self {
            current: start,
            end,
            step,
        }
    }
}

// 3. Implement iterator on the struct
impl Iterator for StepBy {
    type Item = i32; // Defines what the iterator yields

    // 1. Save what you want to return
    // 2. Update the state for the next call
    // 3. Return the saved value
    fn next(&mut self) -> Option<Self::Item> { // How to get to the next item
        // For simple progression if/else is cleaner
        // For conditional skipping while or loop is necessary
        if self.current < self.end {
            // We first save the value 
            let value = self.current;
            // Then we increment
            self.current += self.step;
            // Finally, we return the value
            Some(value)
        } else {
            None
        }
    }
}

// Problem 3: Basic Iterator with Skipping -----

// 1. Create a struct to store the state
struct OddNumbers {
    current: i32,
    end: i32,
}

// 2. Provide a way to initialize
impl OddNumbers {
    fn new(start: i32, end: i32) -> Self {
        Self {
            current: start,
            end,
        }
    }
}

// 3. Implement iterator on the struct
impl Iterator for OddNumbers {
    type Item = i32; // Defines what the iterator yields

    // 1. Save what you want to return
    // 2. Update the state for the next call
    // 3. Return the saved value
    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.end {
            if self.current % 2 != 0 {
                let value = self.current;
                self.current += 1;
                return Some(value);
            } else {
                // When we hit an even number, we still need to increment, but just not return anything
                // The key: Always increment current whether you return the value or not, otherwise you get stuck checking the same even number forever
                self.current += 1;
            }

            // You can also shorten this code to only use an if statement to only return if odd
            // If even, the loop will do nothing and continue to check the next number
        }
        // We just need None here since the while loop will exit once self.current >= self.end
        // In other words, when the condition becomes false, then the loop exits 
        None
    }
}

// Problem 4: Iterator that Yields References -----

// Now let's make an iterator over a Vec that yields references (like .iter() does)

// 1. Create a struct to store the state
// In this case, our struct is generic T, meaning it can work with any data type
// The data type will be determined based on the data type of the vector we reference
// We are also using a lifetime parameter 'a 
// This lifetime parameter means that Vec<T> must live at least as long as the iterator itself
// In other words, MyVecIter cannot outlive the Vec<T> it references
// This is because we are borrowing a vector instead of owning it
struct MyVecIter<'a, T> {
    data: &'a Vec<T>,
    index: usize,
}

// 2. Provide a way to initialize
// When you write impl for a generic struct, you need to declare all the generic parameters that the struct uses
// In this case, we have 2 generic parameters: 'a and T
// The rule: When implementing something for a generic type, you must declare those generics in the impl block
impl<'a, T> MyVecIter<'a, T> {
    // In new(), we need 'a so Rust knows what the lifetime of the iterator is tied to 
    // In our case, the lifetime is tied to the Vec<T> we are referencing
    // Our new() function is also generic, so it knows what type of struct to create
    fn new(data: &'a Vec<T>) -> Self {
        Self {
            data,
            index: 0,
        }
    }
}

// 3. Implement Iterator on the struct
impl<'a, T> Iterator for MyVecIter<'a, T> {

    // As a note, type Item is an Associated Type
    // It is a type alias
    // Self::Item refers back to the type Item you defined

    // Since we have a reference with lifetime 'a and a generic, that is what we need to return
    type Item = &'a T; // Defines what the iterator yields

    // 1. Save what you want to return
    // 2. Update the state for the next call
    // 3. Return the saved value
    fn next(&mut self) -> Option<Self::Item> {
        // .get() is perfect for next() since it already returns Option<&T>
        // .get() already handles bounds checking - it returns None if index is out of bounds, so you don't need to check twice
        let value = self.data.get(self.index);
        if value.is_some() {
            self.index += 1; // Found an item, move forward
        }
        // If value is None, don't increment (we are already past the end)

        value // Return whatever we got (Some(&T) or None)
    }
}

// Problem 5: Function that Returns an Iterator -----

// Now, let's create a function that returns an iterator using impl Iterator
// This is a common pattern when you want to hide the implementation details and just say:
// "I want to return something that's iterable"

// This function takes a vector of i32 and returns something that implements iterator with Item = i32
// Instead of using our own struct, we are composing existing iterators
// Since data: Vec<i32> takes ownership, we will use .into_iter() not .iter()
fn double_iter(data: Vec<i32>) -> impl Iterator<Item = i32> {
    // .into_iter() consumes Vec, yields i32
    // .map() transforms each i32
    data.into_iter().map(|number| number * 2)

    // This function returns Map<IntoIter<i32>, [closure]>
    // But caller just sees: "something that implements Iterator<Item = i32>"

    // impl Iterator is useful
    // Without it, we would have to use a very verbose statement
}

// Problem 6: Chaining Iterator Adapters -----

// Now, let's practice using Rust's built-in iterator methods by chaining multiple adapters together

fn iter_adapt(data: Vec<i32>) -> Vec<i32> {
    // Since Vec<i32> takes ownership, we will use .into_iter() not .iter()

    // Each adapter in this chain returns a new iterator, so you can keep chaining
    // Each iterator adapter is just an iterator that wraps another iterator
    data.into_iter()
        .filter(|number| number % 2 == 0) // .filter() receives a closure that returns bool
        .map(|number| number * number) // .map() takes a closure that transforms the value
        .filter(|number| *number > 20) // We need to dereference here since it is now a reference - there is no .into_iter() before and .filter() always borrows items to check them
        .collect() // Since iterators are lazy, nothing will happen until we call .collect()

        // So, with each iterator adapter just wrapping another iterator, this becomes:
        /* 
        Filter {
            iter: Map {
                iter: Filter {
                    iter: IntoIter {
                        data: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                        index: 0
                    },
                    predicate: |number| number % 2 == 0
                },
                func: |number| number * number
            },
            predicate: |number| *number > 20
        }
        */
}

// Problem 7: Fibonacci Iterator -----

// 1. Create a struct to store the state
struct Fibonacci {
    curr: u64,
    next: u64,
}

// 2. Provide a way to initialize
impl Fibonacci {
    fn new() -> Self {
        Self {
            curr: 0,
            next: 1,
        }
    }
}

// 3. Implement the iterator trait
impl Iterator for Fibonacci {
    type Item = u64;

    // 1. Save what you want to return
    // 2. Update the state for the next call
    // 3. Return the saved value
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.curr; // Save the current value
        let new_next = self.curr + self.next; // Calculate the next next
        self.curr = self.next; // Update the current value to the old next
        self.next = new_next; // Update next to the new next
        Some(value)
    }
}

fn main() {
    // Problem 1: Basic Custom Iterator -----

    // Creating a new iterator
    let counter = Counter::new(1, 5);

    // Consuming that iterator with a for loop
    // The for loop is cyntactic sugar for calling .next() repeatedly until it returns None
    for num in counter {
        print!("{}, ", num);
    }
    println!();

    // Problem 2: Another Basic Custom Iterator -----

    // Creating a new struct
    let stepper = StepBy::new(0, 20, 3);

    // Consuming the iterator with a for loop
    for num in stepper {
        print!("{}, ", num);
    }
    println!();

    // Problem 3: Basic Iterator with Skipping -----

    let odds = OddNumbers::new(1, 10);

    for num in odds {
        print!("{}, ", num);
    }
    println!();

    // Problem 4: Iterator that Yields References -----

    let data = vec![10, 20, 30, 40];
    let iter = MyVecIter::new(&data);
    
    for num in iter {
        print!("{}, ", num);
    }
    println!();

    // Problem 5: Function that Returns an Iterator -----
    
    let numbers = vec![1, 2, 3, 4, 5];
    let doubled = double_iter(numbers);
    
    // Should print: 2, 4, 6, 8, 10,
    for num in doubled {
        print!("{}, ", num);
    }
    println!();

    // Problem 6: Chaining Iterator Adapters -----

    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let result: Vec<i32> = iter_adapt(numbers);

    println!("{:?}", result);

    // Problem 7: Fibonacci Iterator -----
    
    let fib = Fibonacci::new();
    
    for num in fib.take(10) {
        print!("{}, ", num);
    }
    println!();
}
