// Problem 33: Trait-Based Iterator Chain

// Goal: Build a custom iterator that supports method chaining like Rust's standard iterators
// This teches you the foundation of how Rayon's parallel iterators work

// What we are building:
// data.my_iter()
//     .my_map(|x| x * 2)
//     .my_filter(|x| x > 10)
//     .collect::<Vec<_>>()

// Key concepts:
// 1. Each method returns a NEW iterator type that wraps the previous one
// 2. Iterators are lazy - nothing happens until you call .collect() or iterate
// 3. Generic types and trait bounds everywhere
// 4. This is exactly how Rayon's .par_iter().map().filter() works

// The pattern (Iterator Adapter Chain)

// Original data: [1, 2, 3, 4, 5]
//        ↓
// MyIter([1, 2, 3, 4, 5])              ← Base iterator
//        ↓ .my_map(|x| x * 2)
// MapIter(MyIter, |x| x * 2)           ← Wraps base + function
//        ↓ .my_filter(|x| x > 5)
// FilterIter(MapIter(...), |x| x > 5)  ← Wraps map + function
//        ↓ .collect()
// Execute the whole chain: [6, 8, 10]

// Why this matters for Rayon:
// - Rayon does the same thing with ParIter, ParMap, ParFilter
// - Each adapter wraps the previous one
// - All parallel operations compose through this pattern

use std::marker::PhantomData;

// Part 1 - Base Iterator - MyIter -----

// This is our base iterator - it just holds a Vec and yields items one by one
// Think of this as the equivalent of [1,2,3].iter()
// The base iterator is the starting point of the chain - it's the one that actually has or references the data
// Every chain has exactly one base
// It is the foundation - where the data actually lives - all the adapters just transform the flow of data from the base
// We implement the Iterator trait since we need to be able to move through the data one item at a time
// Also, implemeting Iterator allows us to use it in for loops, call .collect(), chain, and use any standard iterator methods

// 1. Create a struct to store the state
// Our struct is generic - it can work with any data type
// The data type will be determined by the data type of the vector
// We do not need a lifetime annotation since we will be taking ownership, like .into_iter()
struct MyIter<T> {
    data: Vec<T>,
    index: usize, // Track current position
}

// 2. Provide a way to initialize
// When you write impl for a generic struct, you need to declare all the generic parameters that the struct uses
impl<T> MyIter<T> {
    fn new(data: Vec<T>) -> Self {
        Self {
            data,
            index: 0,
        }
    }
}

// 3. Implement the Iterator trait
// We are implementing the standard Iterator trait so we can use it in for loops
// We have a trait bound Clone
// We need the Clone trait bound because we need to get T out of Vec<T> without moving/destroying the original Vec 
// Cloning is the simplest way to do this while keeping the API clean (returning T instead of &T)
// This a hybrid iterator since it consumes the vec (like .into_iter()) but clones them instead of moving them out and yields owned values (like .into_iter())
impl<T: Clone> Iterator for MyIter<T> {

    // Type alias
    // Associated type
    // Define what the iterator yields
    type Item = T;

    // 1. Save what you want to return
    // 2. Update the state for the next call
    // 3. Return the saved value
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let item = self.data[self.index].clone();
            self.index +=1;
            Some(item)
        } else {
            None
        }
    }
}

// Part 2 - Map Adapter - MapIter -----

// MapIter wraps another iterator and applies a function to each element
// This is like .map() in standard Rust

// Each iterator (base or adapter) follows the struct (for state) -> initialize -> iterator trait sequence

// Generic parameters:
// I: The iterator we are wrapping (could be MyIter, another MapIter, etc.)
// F: The function we are applying
// T: Input type (what I yields)
// U: Output type (what F produces)

// 1. Create a struct to store the state
struct MapIter<I, F, T, U>
where
    // I is an Iterator that yields T
    I: Iterator<Item = T>,
    // F is a callable that takes T and returns U
    F: Fn(T) -> U,
{
    iter: I, // The iterator we are wrapping
    func: F, // The transformation function
    _phanton: PhantomData<(T, U)>, // Tell the compiler about T and U
}

// PhantomData explanation:
// We need to tell the compiler about T and U even though we don't store them directly - it is only for compile-time reasoning
// PhantomData is a zero-sized type that says "this struct logically contains or depends on these types, even though they don't appear in any fields"
// If a generic type parameter is not used in any field, the compiler assumes the struct has no relationship to that type

// 2. Provide a way to initialize
// This will create a new instance of MapIter with the iterator we are wrapping and the function we are applying
impl<I, F, T, U> MapIter<I, F, T, U>
where
    // I is an Iterator that yields T
    I: Iterator<Item = T>,
    // F is a callable that takes T and returns U
    F: Fn(T) -> U,
{
    fn new(iter: I, func: F) -> Self {
        Self {
            iter, // Store the iterator we are wrapping
            func, // Store the function we will apply
            _phanton: PhantomData, // Zero-sized type marker
        }
    }
}

// 3. Implement the Iterator trait
impl<I, F, T, U> Iterator for MapIter<I, F, T, U>
where
    // I is an Iterator that yields T
    I: Iterator<Item = T>,
    // F is a callable that takes T and returns U
    F: Fn(T) -> U,
{
    type Item = U; // We output U (the result of the function)
    // This is because we are mapping (transforming)

    // The purpose of this .next() call is to call .next() on the wrapper iterator and transform it using the stored function
    fn next(&mut self) -> Option<Self::Item> {
        // Get the next item from the wrapper iterator and apply a function to it

        // 1. We take the wrapped iterator and call .next() on it -> if it is MyIter, it calls MyIter::next()
        // 2. This returns Option<T>
        // 3. We then apply a function using Option::map(), if it is Some(item) it returns Some(result), if None then None
        // 4. We then apply the closure/function that was stored in the MapIter struct
        self.iter.next().map(|item| (self.func)(item)) // Transformation function is being applied to every item (no skipping)

        // The parentheses around self.func make it clear we are treating it as a callable - distinguishing it from a method call (self.func(item))
        // Get the field func first then call it with item
        // "I want to call the function stored in the func field, not look for a method named func."
        // This is standard Rust for calling closures stored in struct fields
    }
}

// Part 3 - Filter Adapter - FilterIter -----

// FilterIter wraps another iterator and only yields items that pass a predicate
// This is like .filter() in standard Rust

// Each iterator (base or adapter) follows the struct (for state) -> initialize -> iterator trait sequence

// Generic parameters:
// I: The iterator we are wrapping (could be MyIter, another MapIter, etc.)
// F: The function we are applying
// T: Input type (what I yields)
// We do not need U since we are not transforming and returning a different type - we are returning the same type

// 1. Create a struct to store the state
struct FilterIter<I, F, T> 
where
    // I is an iterator the yields T
    I: Iterator<Item = T>,
    // F is a callable that takes a reference to T and returns a bool
    F: Fn(&T) -> bool,
{
    iter: I,
    predicate: F,
    _phantom: PhantomData<T>,
}

// PhantomData explanation:
// We need to tell the compiler about T and U even though we don't store them directly - it is only for compile-time reasoning
// PhantomData is a zero-sized type that says "this struct logically contains or depends on these types, even though they don't appear in any fields"
// If a generic type parameter is not used in any field, the compiler assumes the struct has no relationship to that type

// 2. Provide a way to initialize
impl<I, F, T> FilterIter<I, F, T>
where
    // I is an iterator that yields T
    I: Iterator<Item = T>,
    // F is a callable that takes a reference to T and returns a bool
    F: Fn(&T) -> bool,
{
    fn new(iter: I, predicate: F) -> Self {
        Self {
            iter, // Store the iterator we are wrapping
            predicate, // Store the filter condition
            _phantom: PhantomData, // Zero-sized type marker
        }
    }
}

// 3. Implement the Iterator trait
impl<I, F, T> Iterator for FilterIter<I, F, T>
where
    // I is an iterator that yields T
    I: Iterator<Item = T>,
    // F is a callable that takes a reference to T and returns a bool
    F: Fn(&T) -> bool,
{
    type Item = T; // Filter doesn't change the type, just which items pass through

    fn next(&mut self) -> Option<Self::Item> {
        // Keep calling next() until we find an item that passes the predicate
        // This is the key difference from map - we might skip items
        loop {
            match self.iter.next() {
                Some(item) => {
                    // With (self.predicate), it is about calling a function stored in a field
                    // Call the function stored in self.predicate, passing &item as its argument
                    // This is standard Rust for calling closures stored in struct fields
                    if (self.predicate)(&item) {
                        return Some(item); // Only return an item if the condition is true
                    }
                    // Item didn't pass (condition is false), continue to next loop iteration
                }
                None => return None, // Wrapped iterator is exhausted
            }
        }
    }
}

// Note Block -----

// The universal adapter pattern:
// 1. Struct with trait bounds
// 2. Provide a way to initialize
// 3. Implement Iterator to move through the wrapped iterator

// MapIter Pattern:
// 1. Struct with wrapped iter and transformation function
// 2. Initialize
// 3. Iterator implementation that yields transformed type
    // Get item from wrapped, apply function

// FilterIter Pattern:
// 1. Struct with wrapped iter and filter condition
// 2. Initialize
// 3. Iterator implementation that yields the same type
    // Get item from wrapped, apply predicate, skip if false

// What changes betwen adapters:
// 1. The callable's signature (transform vs predicate)
// 2. The logic in next() (transform vs filter)
// 3. The output type (U vs T)

// Every adapter follows:
// 1. Struct holding
    // - Wrapped iterator (I)
    // - Modification logic (F or state)
    // - PhantomData for type parameters
// 2. Constructor (new)
    // - Takes wrapped iterator
    // - Takes modification logic/params
    // - Returns Self
// 3. Iterator implementation
    // - Define what type we yield
    // - Call wrapped.next()
    // - Apply modification logic
    // - Return result

// Part 4 - Extension Methods - The Magic of Chaining -----

// Now we add .my_map() and .my_filter() methods to ANY iterator, not just MyIter
// This is done through a trait with blanket implementation
// This is exactly how Rust's standard library does it

// We cannot add methods to Iterator directly with impl Iterator
// This is because we do not own the Iterator trait -> it is from Rust's standard library
// Thus, we have to create our own trait with methods that we want and implement the trait for all iterators

// 1. The trait definition
// The trait bounds are:
// - Iterator: This trait can only be implemented on types that already implement iterator
// - Sized: The type must have a known size at compile time
    // Most types are Sized, but trait objects (dyn Iterator) are not
// For a type to implement MyIteratorExt, it will have to have the required methods below
// These methods only work on types that are Iterator + Sized
// We have a default implementation for both methods
// For each method, we ask what does it take? + what does it produce?
trait MyIteratorExt: Iterator + Sized {
    // F - generic: the function type
    // U - generic: the output type
    // func - the transformation function (closure)
    // my_map:
        // Takes self (current iterator)
        // Takes func (transformation function)
        // Wraps them together in a new MapIter
        // Returns that MapIter
        // The MapIter has Self (iterator type - type calling this method), F (function type), Self::Item (input type - whatever the iterator yields), U (output type)
    fn my_map<F, U>(self, func: F) -> MapIter<Self, F, Self::Item, U> 
    where
        // Self since we are taking ownership
        F: Fn(Self::Item) -> U,
        {
            MapIter::new(self, func)
        }
    // F - generic: the function type
    // predicate - filter condition
    // my_filter:
        // Takes self (current iterator)
        // Takes predicate (filter condition)
        // Wraps them together in FilterIter
        // Returns FilterIter
        // The FilterIter has Self (iterator type - type calling this method), F (filter condition), Self::Item (item type)
    fn my_filter<F>(self, predicate: F) -> FilterIter<Self, F, Self::Item>
    where
        // &Self since we are not taking ownership
        F: Fn(&Self::Item) -> bool,
    {
        FilterIter::new(self, predicate)
    }
}

// The point of having this methods is to be able to cleanly chain methods
// Without this, we would have to manually call MapIter::new() and FilterIter::new(), nest everything ourselves, and keep track of intermediate variables
// With this extension method, you can chain naturally with dots, there are no intermediate variables, and it is clear + concise

// Blanket implementation: ANY type that implements Iterator gets these methods
// This one line gives every iterator ever access to our methods
// For any type I that implements the Iterator tait, implement the MyIteratorExt 
// We are implementing MyIteratorExt trait for type I (anything that implements the Iterator trait)
// It is empty since we already have default implementations for all its methods
impl<I: Iterator> MyIteratorExt for I {}

// To summarize this section, we are adding methods to every iterator through an extension
// This is called the extension trait pattern - it's how you add methods to types you don't own
// This will allow all types that implement the Iterator trait to call our methods
// It will also let use write .into_iter().my_map(), etc.

// Part 5 - Convenience Trait for Creating MyIter from Vec -----

// We are extending Rust's Vec to have a new method through a trait
// The method we are adding is .my_iter()
// Give Vec<T> a new method .my_iter() that converts it into a MyIter<T>

// Types that implement this trait have a .my_iter() method that produces a MyIter<T>
// T: generic - works for Vec<i32>, Vec<String>, etc.
trait IntoMyIter<T> {
    // Takes Vec and returns MyIter
    fn my_iter(self) -> MyIter<T>;
}

// For any Vec<T>, implement IntoMyIter<T>
impl<T> IntoMyIter<T> for Vec<T> {
    fn my_iter(self) -> MyIter<T> {
        // Pass the Vec (self) to MyIter::new()
        MyIter::new(self)
    }
}

// For type we don't own, like Vec, String, Option, etc. that come from Rust's standard library
// we have to use the extension trait pattern to add new methods
// We must:
    // 1. Create our own trait with the methods you want
    // 2. Implement the trait for that type
    // 3. Use it (bring it into scope/import it)

// -----

// Short summary:

// 1. We create a base iterator and adapters
// 2. Extend all iterators using the extension trait pattern
// 3. Extend Vec to be able to call .my_iter(), also using the extension trait pattern

fn main() {
    println!("=== Problem A: Trait-Based Iterator Chain ===\n");

    // Test 1: Basic iteration
    println!("Test 1: Basic MyIter");
    let data = vec![1, 2, 3, 4, 5];
    let iter = MyIter::new(data);
    let result: Vec<i32> = iter.collect();
    println!("Input: [1, 2, 3, 4, 5]");
    println!("Output: {:?}", result);
    println!("Expected: [1, 2, 3, 4, 5]\n");

    // Test 2: Just map
    println!("Test 2: Map only");
    let data = vec![1, 2, 3, 4, 5];
    let result: Vec<i32> = data.my_iter()
        .my_map(|x| x * 2)
        .collect();
    println!("Input: [1, 2, 3, 4, 5]");
    println!("Transform: x * 2");
    println!("Output: {:?}", result);
    println!("Expected: [2, 4, 6, 8, 10]\n");

    // Test 3: Just filter
    println!("Test 3: Filter only");
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let result: Vec<i32> = data.my_iter()
        .my_filter(|x| x % 2 == 0)
        .collect();
    println!("Input: [1, 2, 3, 4, 5, 6, 7, 8]");
    println!("Filter: keep evens");
    println!("Output: {:?}", result);
    println!("Expected: [2, 4, 6, 8]\n");

    // Test 4: Map then filter (THE MAIN EVENT!)
    println!("Test 4: Map THEN Filter");
    let data = vec![1, 2, 3, 4, 5];
    let result: Vec<i32> = data.my_iter()
        .my_map(|x| x * 2)      // [2, 4, 6, 8, 10]
        .my_filter(|x| *x > 5)  // [6, 8, 10]
        .collect();
    println!("Input: [1, 2, 3, 4, 5]");
    println!("Step 1 (map): x * 2 → [2, 4, 6, 8, 10]");
    println!("Step 2 (filter): keep > 5 → [6, 8, 10]");
    println!("Output: {:?}", result);
    println!("Expected: [6, 8, 10]\n");

    // Test 5: Filter then map
    println!("Test 5: Filter THEN Map");
    let data = vec![1, 2, 3, 4, 5];
    let result: Vec<i32> = data.my_iter()
        .my_filter(|x| *x > 2)  // [3, 4, 5]
        .my_map(|x| x * 10)     // [30, 40, 50]
        .collect();
    println!("Input: [1, 2, 3, 4, 5]");
    println!("Step 1 (filter): keep > 2 → [3, 4, 5]");
    println!("Step 2 (map): x * 10 → [30, 40, 50]");
    println!("Output: {:?}", result);
    println!("Expected: [30, 40, 50]\n");

    // Test 6: Multiple maps
    println!("Test 6: Chain multiple maps");
    let data = vec![1, 2, 3];
    let result: Vec<i32> = data.my_iter()
        .my_map(|x| x + 1)      // [2, 3, 4]
        .my_map(|x| x * 2)      // [4, 6, 8]
        .my_map(|x| x - 1)      // [3, 5, 7]
        .collect();
    println!("Input: [1, 2, 3]");
    println!("Step 1: x + 1 → [2, 3, 4]");
    println!("Step 2: x * 2 → [4, 6, 8]");
    println!("Step 3: x - 1 → [3, 5, 7]");
    println!("Output: {:?}", result);
    println!("Expected: [3, 5, 7]\n");

    // Test 7: Type transformation (i32 -> String)
    println!("Test 7: Type transformation");
    let data = vec![1, 2, 3];
    let result: Vec<String> = data.my_iter()
        .my_map(|x| x * 2)
        .my_map(|x| format!("Number: {}", x))
        .collect();
    println!("Input: [1, 2, 3]");
    println!("Transform: i32 → String");
    println!("Output: {:?}", result);
    println!("Expected: [\"Number: 2\", \"Number: 4\", \"Number: 6\"]\n");

    // Test 8: Lazy evaluation demonstration
    println!("Test 8: Lazy evaluation");
    let data = vec![1, 2, 3, 4, 5];
    
    println!("Creating iterator chain (no execution yet)...");
    let iter = data.my_iter()
        .my_map(|x| {
            println!("  Mapping: {}", x);
            x * 2
        })
        .my_filter(|x| {
            println!("  Filtering: {}", x);
            *x > 5
        });
    
    println!("Now collecting (this triggers execution):");
    let result: Vec<i32> = iter.collect();
    println!("Final result: {:?}\n", result);
}
