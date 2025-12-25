#![allow(dead_code)]

use std::fmt;

struct DataPoint {
    id: u32,
    value: f64,
    category: String,
    timestamp: u64,
}

struct DataStats {
    count: usize,
    sum: f64,
    average: f64,
    min: f64,
    max: f64,
}

#[derive(Debug)]
enum ProcessingError {
    EmptyDataset,
    InvalidValue(f64),
    OutOfRange { value: f64, max: f64 },
}

struct Dataset {
    name: String,
    points: Vec<DataPoint>,
}

impl fmt::Display for DataPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DataPoint #{}: {} (category: {}) @ {}", self.id, self.value, self.category, self.timestamp)
    }
}

// Associated function
// It is a call on the type, not on an instance of the type
impl Default for Dataset {
    fn default() -> Self {
        Self {
            name: "Untitled".to_string(),
            points: Vec::new(),
        }
    }
}

impl Dataset {
    fn new(name: String) -> Self {
        Self {
            name,
            points: Vec::new(),
        }
    }

    fn add_point(&mut self, point: DataPoint) {
        self.points.push(point)
    }

    fn len(&self) -> usize {
        self.points.len()
    }

    fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    fn calculate_stats(&self) -> Result<DataStats, ProcessingError> {
        if self.is_empty() {
            return Err(ProcessingError::EmptyDataset)
        }

        // First we are extracting all values into a vector
        // This is because it is more efficient - we need to iterate multiple times
        // Working with Vec<f64> is simpler than repeatedly extracting from Vec<DataPoint>
        // Better to extract once than call .iter().map() 3 times
        let values: Vec<f64> = self.points.iter().map(|point| point.value).collect();

        let sum: f64 = values.iter().sum();
        let count = values.len();

        // .fold() is an iterator method that combines all elements into a single value by repeatedly applying a function
        // f64::INFINITY is the initial value - starting point
        // For each element in the iterator, we call the closure with the current accumulator and next element
        // The result becomes the new accumulator
        // Start with this value, then repeatedly combine it with each element to produce a final result
        // It's like a general version of .sum() or .collect() where YOU define how to combine things

        // It starts at the initial value (a) and compares it to the first element (b) -> the minimum becomes the new a
        // It then compares the second element (b) to the previous value (a)  -> the minimum becomes the new a
        // This continues until all elements are processed and returns the final (a), which is the overall minimum
        // Within the closure ||, on the first iteration the first element (accumulator) is the initial value and the second element (element) is the first element in the collection
        // On subsequent iterations, accumulator is the result from previous iterations and element is the next element in the collection
        // The accumulator "accumulates" or "carries forward"
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        // For max, we start with negative infinity (smallest possible)
        // .fold() takes many elements, repeatedly applies a function to combine them, and returns ONE single value
        // It is a reduction operation - you're reducing or folding a collection down into a single result
        // "Take this collection and fold/collapse it down into a single thing by repeatedly applying this combining function"
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Ok(DataStats { 
            count, 
            sum, 
            average: sum / count as f64, 
            min, 
            max})
    }

    // This is returning an iterator without collecting into a Vec
    // Instead of returning Vec<&DataPoint>, we are returning impl Iterator<>
    fn filter_by_category<'a>(&'a self, category: &'a str) -> impl Iterator<Item = &'a DataPoint> + 'a {
        self.points.iter().filter(move |point| point.category == category) // No collect
        // We return an iterator when we want lazy evaluation - doesn't do work until consumed
        // more efficient - no allocation
        // composable - caller can chain more operations

        // The iterator can only live as long as the Dataset (self) it points to - because it yields &DataPoint references from self
        // The category can only live as long as the category parameter - because the closure inside captures category
        // They must all live for at least a and none can outlive a
        // The iterator is constrained by BOTH: the dataset is borrows from and the category it captures
        // They all live and die together
    }

    // impl - returns something that implements
    // Iterator - the Iterator trait
    // Item = f64 - and yields f64 values
    // + '_ - and it tied to the lifetime of self
    fn values_above(&self, threshold: f64) -> impl Iterator<Item = f64> + '_ {
        // impl Iterator - I'm returning something callable/iterable - filter, map, chain, or any iterator type
        // <Item = f64> - This is an associated type from the Iterator trait - when you iterate, each item with be an f64
        // + '_ - The iterator borrows from self, infer lifetime from context, the iterator can't outlive the borrow of self
        // if self (DataSet instance) goes out of scope, then the iterator goes out of scope - they live and die together
        // Think of it like a leash - The iterator (dog) can't wander away from the dataset (owner) - when the owner leaves, so does the dog

        // We are first using .filter() to get the DataPoints above the threshold
        // We are then using .map() to transform the remaining DataPoints into their actual values
        self.points.iter().filter(move |point| point.value >= threshold).map(|point| point.value)
    }

    // F here is a generic - usually used for callables like functions and closures
    // F - has to be something that implements the Fn trait (is a callable), takes an f64 and returns an f64
    fn transform_values<F>(&self, f: F) -> Vec<f64> 
    where
        F: Fn(f64) -> f64
    {   
        // We are iterating over &DataPoint
        // Applying function f to each value (extract and transform it)
        // Then gathing into a Vec<f64>
         self.points.iter().map(|point| f(point.value)).collect()

        // Usage example: let doubled = dataset.transform_values(|x| x * 2.0);
        // let shifted = dataset.transform_values(|x| x + 10.0);
    }

    // usize is an unsigned integer for indices and lengths
    fn top_n_by_value(&self, n: usize) -> Vec<&DataPoint> {
        let mut points: Vec<&DataPoint> = self.points.iter().collect();

        // .sort_by() takes a closure that compares 2 items
        // then the comparison logic
        // .partial_cmp compares f64 values and returns Option<Ordering>
        // b first - descending order
        points.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());

        // .take(n) takes the first n items from an iterator
        // "Give me the first N items, then stop"
        // Very useful for limiting results
        points.into_iter().take(n).collect()
    }
}

// &[f64] is a reference to a continguous sequence of f64 values
// It is a slice of f64 values
// A slice is a view into a sequence of elements
// slices can come from:
// Borrowing the entire vec as a slice: let slice: &[f64] = &vec;
// Borrowing part of a vec: let slice: &[f64] = &vec[0..2];
// We use &[f64] because it accepts ANY borrowed sequence of f64 - it is the most flexible paramter type for "give me a sequence of characters to read" 
// Works with Vec, arrays, and other slices
// It is like &str but for numeric values 
fn moving_average(values: &[f64], window_size: usize) -> Vec<f64> {
    // Handle edge cases
    if values.is_empty() || window_size == 0 || window_size > values.len() {
        return Vec::new();
    }

    let mut result = Vec::new();

    // Slide the window across the data
    for i in 0..=(values.len() - window_size) {
        // Get the window slice
        let window = &values[i..i + window_size];
        
        // Calculate average of this window
        let sum: f64 = window.iter().sum();
        let avg = sum / window_size as f64;
        
        result.push(avg);
    }

    result
}

// This takes Option<T> since fields start as None
// Each builder method sets a field to Some(value)
// build() checks if all the required fields are set
struct DataPointBuilder {
    id: Option<u32>,
    value: Option<f64>,
    category: Option<String>,
    timestamp: Option<u64>,
}
impl DataPointBuilder {
    fn new() -> Self {
        Self {
            id: None,
            value: None,
            category: None,
            timestamp: None,
        }
    }

    // This is using a different builder pattern than we have seen before
    // In the previous builder pattern we have seen before, we were creating NEW instances: -> Self and returning Self {}
    // In this new builder pattern, we are mutating and returning the SAME instance
    // mut allows the modifying self
    fn id(mut self, id: u32) -> Self { // Self is DataPointBuilder (Type)
        // We do not return self since self is a VALUE not a TYPE
        // Return type annotations need types, not values
        // You can never use self (lowercase) as a return TYPE
        // but you can return self as a VALUE
        // After -> in signature must be a TYPE
        // In a function body (return value) can be a VALUE
        // You can return value self but never use it as a TYPE
        // self refers to an instance of a type not the type itself

        self.id = Some(id); // modify the existing instance
        self // Return the SAME (modified) instance
        // lowercase self - return the instance we just modified
        // We are returning self because we want to return the same instance we just modified
        // If we return Self {} - this creates a brand new instance from scratch
        // self is the actual builder instance and the type is Self (DataPointBuilder)
        // You could write Self {...} with copying all other fields, but it is more verbose
        // We are passing the same builder instance, modfying it at each step, then returning it
    }

    fn value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }

    fn category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    fn timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    fn build(self) -> DataPoint {
        DataPoint {
            //.unwrap_or(default_value)
            // If Some(value), returns the value
            // If None, returns the default value you provide 
            // Use when default is cheap and a constant
            // This is evaluated immediately - 0 is created if its needed or not
            id: self.id.unwrap_or(0), // 
            value: self.value.unwrap_or(0.0),
            // .unwrap_or_else(|| closure)
            // If Some(value), returns the value
            // If None, calls the closure to compute the default
            // Use when default is expensive to compute and you want lazy evaluation
            // The closure here is only called if its needed - only creates a String if None
            category: self.category.unwrap_or_else(|| "Unknown".to_string()),
            timestamp: self.timestamp.unwrap_or(0),
        }
    }
}

fn main() {
    // Create a dataset
    let mut dataset = Dataset::new("Sales Data".to_string());
    
    // ========================================
    // BUILDER PATTERN EXAMPLES
    // ========================================
    
    // Example 1: Build with all fields
    let point1 = DataPointBuilder::new()
        .id(1)
        .value(85.5)
        .category("Electronics".to_string())
        .timestamp(1638360000)
        .build();
    
    // Example 2: Build with only some fields (others get defaults)
    let point2 = DataPointBuilder::new()
        .id(2)
        .value(120.0)
        .category("Electronics".to_string())
        .build();  // timestamp will be 0 (default)
    
    // Example 3: Chain in different orders
    let point3 = DataPointBuilder::new()
        .category("Furniture".to_string())
        .id(3)
        .timestamp(1638370000)
        .value(45.0)
        .build();
    
    // Add regular points (not using builder)
    dataset.add_point(DataPoint {
        id: 4,
        value: 95.0,
        category: "Electronics".to_string(),
        timestamp: 1638380000,
    });
    
    dataset.add_point(point1);
    dataset.add_point(point2);
    dataset.add_point(point3);
    
    println!("Dataset has {} points\n", dataset.len());
    
    // ========================================
    // ITERATOR EXAMPLES (NO COLLECTING)
    // ========================================
    
    // Example 1: filter_by_category - iterate directly
    println!("=== Electronics Category (using iterator directly) ===");
    for point in dataset.filter_by_category("Electronics") {
        println!("  ID: {}, Value: {}", point.id, point.value);
    }
    // dataset.filter_by_category("Electronics") returns an iterator
    // for loop automatically calls .into_iter() or whatever you give it
    // Each iteration calls .next() on the iterator
    // Stops when .next() returns None
    // for loops work directly with iterators
    
    // Example 2: filter_by_category - chain with map and collect
    println!("\n=== Electronics Values (chaining) ===");
    let electronics_values: Vec<f64> = dataset
        .filter_by_category("Electronics")
        .map(|point| point.value)
        .collect();
    println!("Values: {:?}", electronics_values);
    // dataset.filter_by_category("Electronics") returns an iterator
    // .map() transforms to Iterator<Item = f64>
    // .collect() consumes iterator, produces Vec<F64>
    
    // Example 3: filter_by_category - chain with sum
    let electronics_total: f64 = dataset
        .filter_by_category("Electronics")
        .map(|point| point.value)
        .sum();
    println!("Total electronics value: ${:.2}", electronics_total);
    // dataset.filter_by_category("Electronics") returns an iterator
    // .map() transforms to Iterator<Item = f64>
    // .sum() sums everything in the iterator

    // The benefits of returning an Iterator over a tangible result are:
    // 1 - Lazy evaluation - only does work when needed
    // 2 - Memory Efficiency - No intermediate allocations
    // 3 - Composability - Caller decies what to do, we can chain operations
    // 4 - Performance - Can short-circuit 

    // Returning an iterator is more flexible, efficient, and has lazy evaluation but it is more complex
    // Returning a Vec is simpler, but always allocates, does all the work upfront, and is less composable

    // Return iterators (most of the time) for filtering/mapping, when dont know what the caller will do with the resuts, performance, building library/reusable code
    // Return vec (sometimes) for needing to iterate multiple times, need random access, simple one-off functions, results are small

    // In modern Rust, returning iterators is considered idiomatic - it gives maximum flexibility to the caller
    
    // Example 4: filter_by_category - chain with count
    let electronics_count = dataset
        .filter_by_category("Electronics")
        .count();
    println!("Number of electronics: {}", electronics_count);
    
    // Example 5: values_above - sum high values
    println!("\n=== Values Above $80 ===");
    let high_value_total: f64 = dataset.values_above(80.0).sum();
    println!("Total of values above $80: ${:.2}", high_value_total);
    
    // Example 6: values_above - collect into Vec
    let high_values: Vec<f64> = dataset.values_above(80.0).collect();
    println!("High values: {:?}", high_values);
    
    // Example 7: values_above - iterate and print
    println!("\n=== All Values Above $50 ===");
    for value in dataset.values_above(50.0) {
        println!("  ${:.2}", value);
    }
    
    // Example 8: values_above - chain multiple operations
    let avg_high_values = {
        let values: Vec<f64> = dataset.values_above(80.0).collect();
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    };
    println!("\nAverage of values above $80: ${:.2}", avg_high_values);
    
    // Example 9: Combining both iterators
    println!("\n=== Electronics Above $90 ===");
    let result: Vec<f64> = dataset
        .filter_by_category("Electronics")
        .filter(|point| point.value > 90.0)
        .map(|point| point.value)
        .collect();
    println!("Values: {:?}", result);
    
    // Example 10: Using transform_values with closure
    println!("\n=== Transform Values ===");
    let doubled = dataset.transform_values(|x| x * 2.0);
    println!("Doubled values: {:?}", doubled);
    
    let with_tax = dataset.transform_values(|x| x * 1.08);
    println!("Values with 8% tax: {:?}", with_tax);
    
    // ========================================
    // STATS AND OTHER METHODS
    // ========================================
    
    match dataset.calculate_stats() {
        Ok(stats) => {
            println!("\n=== Dataset Statistics ===");
            println!("Count: {}", stats.count);
            println!("Sum: ${:.2}", stats.sum);
            println!("Average: ${:.2}", stats.average);
            println!("Min: ${:.2}", stats.min);
            println!("Max: ${:.2}", stats.max);
        }
        Err(e) => println!("Error: {:?}", e),
    }
    
    // Top performers
    println!("\n=== Top 3 by Value ===");
    for point in dataset.top_n_by_value(3) {
        println!("  ID {}: ${:.2}", point.id, point.value);
    }
    
    // Moving average
    let values: Vec<f64> = dataset.points.iter().map(|p| p.value).collect();
    let smoothed = moving_average(&values, 2);
    println!("\n=== Moving Average (window=2) ===");
    println!("Original: {:?}", values);
    println!("Smoothed: {:?}", smoothed);
}