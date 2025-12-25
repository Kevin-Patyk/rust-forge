#![allow(dead_code)]

use std::fmt;

struct Student {
    name: String,
    id: u32,
    scores: Vec<f64>,
}

enum GradeLevel {
    Freshman,
    Sophmore,
    Junior,
    Senior,
}

struct Course {
    name: String,
    students: Vec<Student>,
    passing_grade: f64,
}

impl fmt::Display for Student {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Vec<T> does not implement Display in Rust
        // It does implement Debug though
        // If you want Display-style output, you must format it yourself
        write!(f, "Student: {} (ID: {}) - Scores: {:?}", self.name, self.id, self.scores)
    }
}

// Default is a constructor, like fn new()
// Default is better with simple initialization logic and a sensible default/empty state
impl Default for Course {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            students: Vec::new(),
            passing_grade: 60.0,
        }
    }
}

impl Student {
    fn new(name: String, id: u32) -> Self {
        Self {
            name, 
            id,
            scores: Vec::new(),
        }
    }

    fn add_score(&mut self, score: f64) {
        self.scores.push(score)
    }

    fn get_average(&self) -> Option<f64> {
        if self.scores.is_empty() {
            return None
        } else {
            // We do not want to use .into_iter() here since it will take ownership and consume the collection
            // We need to provide a type hint since .sum() is a generic method that can produce different types
            // Without a hint, Rust doesn't know if you want f64, f32, i32, etc.
            let total: f64 = self.scores.iter().sum();
            Some(total / self.scores.len() as f64)
        }
    }

    // This is a method that takes a closure/function as a parameter
    // Generic type parameter <F> - for function/closure types - like <T> but for functions
    // curve_fn is a parameter that is some callable thing (function or closure)
    // "Give me any function or closure that takes a score (f64) and returns a modified score (f64) and I will apply it to all the student's scores"
    // This is a higher order function - a function that takes another function as input
    fn apply_curve<F> (&mut self, curve_fn: F) // F is just a convention - you can name it anything
    where
        // Fn: trait for callable things (functions/closures)
        // (f64) - takes one parameter of type f64
        // -> returns f64
        F: Fn(f64) -> f64 // F must be a function/closure that takes f64 and returns f64
        // Fn is Rust's way of saying: "I don't care it it's a function or a closure - just something I can call"
    {   
        // Here, we are calling the function on each score
        self.scores = self.scores.iter().map(|score| curve_fn(*score)).collect();
    }
    // Can use it like: student.apply_curve(function);
    // student.apply_curve(|score| score + 5.0)
    // The above is closure syntax - anonymous function
    // |score| is the input - like function parameters 
    // after |score| is the body (what it does) - the expression to evaluate
    // The vertical bars || are how you define closure parameters - think of them like parentheses in a function signature
    // And what comes after the vertical bars || is like the function body
}

impl Course {
    fn new(name: String, passing_grade: f64) -> Self {
        Self {
            name,
            students: Vec::new(), // This is a vector of Student structs
            passing_grade,
        }
    }

    fn add_student(&mut self, student: Student) {
        self.students.push(student)
    }

    fn get_passing_students(&self) -> Vec<&Student> {
        self.students.iter().filter(|student| {
            // .filter() needs to return a boolean (true/false)
            match student.get_average() {
                Some(grade) => grade >= self.passing_grade,
                None => false,
            }
        })
        .collect()
    }

    fn get_top_performers(&self, min_average: f64) -> Vec<String> {
        // .filter() keeps students with average >= min_average
        // .map() transforms from &Student to String (their names)
        self.students.iter().filter(|student| {
            match student.get_average() {
                Some(grade) => grade >= min_average,
                None => false,
            }
        }).map(|student| student.name.clone())
        .collect()
        // .iter() returns an Iterator<Item = &Student>
        // .filter() returns an Iterator<Item = &Student> but fewer items - still an iterator
        // .map() returns Iterator<Item = String> and transforms it
        // .collect() makes it a Vector
        // .filter() followed by .map() is processing the same structure but through lazy iterators that only execute when collected
    }

    // <F> is a generic type parameter (like <T> but for functions)
    // curve_fn: F is a parameter that is some callable thing (function or closure)
    fn apply_curve_to_all<F>(&mut self, curve_fn: F) 
    where
        // This is a constraint - F must be something you can call with an f64 that returns an f64
        // Fn - trait for callable things (functions/closures)
        // (f64) - takes one parameter of type f64
        // -> f64 returns f64
        F: Fn(f64) -> f64 
    {
        for student in &mut self.students {
            // We are referencing &curve_fn since, in the first iteration, it will be moved into apply_curve()
            // Then, in the second iteration, we can't use curve_fn again since it's been moved
            // The & lets you use the function/closure multiple times without moving it
            student.apply_curve(&curve_fn);
            // We are using the .apply_curve() method we made for the Student struct cause of DRY
            // Don't Repeat Yourself
        }
    }
}

// This is a closure factory - a function that creates customized closures
// This is a function that returns a closure
// Takes points as an input
// Returns "Something callable (Fn) that takes f64 and returns f64"
fn create_add_curve(points: f64) -> impl Fn(f64) -> f64 { // Returns something that implements the Fn trait & takes f64 and returns f64
    // The move keyword captures points by value (takes ownership)
    // This is the closure that is being returned
    move |score| score + points // move transfers ownership of points into the closure, so it lives as long as the closure
    // score is the score to curve
    // score + points is what the closure does (adds the points)

    // You can then use this function like: let add_five = create_add_curve(5.0); -> This creates a function that will always add 5 to the input
    // Now add_five() is a closure that adds 5 to any score: let curved = add_five(80.0);
}

// The Fn trait is one of three traits for callable things (functions and closures) in Rust
// Fn borrows immutably
// FnMut borrows captured variables mutably
// FnOnce takes ownership of captured variables

// Writing F: Fn(...) is saying: F must be a callable that can be called multiple times, doesn't mutate captures variables, is the most restrictive/safe callable
// Some callables implement Fn, others FnMut, and others FnOnce

// Generally, if something implements Fn, FnMut, or FnOnce it's callable 
// If something is callable -> It implements at least FnOnce
// These Fn traits are how Rust represents callable things

// This return annotation is: "Returns a callable function/closure (implements the Fn trait) that takes f64 and returns f64"
// We can then make a function for this that will satisfy the trait bounds for .add_curve()
// let add_curve = create_add_curve(5.0); This is a callable that implements Fn(f64) -> f64
// student.apply_curve(add_curve); Satisfies the trait bound F: Fn(f64) -> f64
fn create_multiply_curve(factor: f64) -> impl Fn(f64) -> f64 {
    move |score| score * factor
}

// This takes a generic F
// Where F must be a callable (Fn) thats takes an f64 and returns an f64
fn process_scores<F>(scores: &[f64], processor: F) -> Vec<f64>
// &[f64] is a reference to a contiguous sequence of f64 values
// It is a slice of f64 values
// A slice is a view into a sequence of elements - doesnt own the data, borrows - has unknown sizer at compile time - always behind reference
// slices can come from:
// Borrowing the entire vec as a slice: let slice: &[f64] = &vec;
// Borrowing part of a vec: let slice: &[f64] = &vec[0..2];
// We use &[f64] because it accepts ANY borrowed sequence of f64 - it is the most flexible parameter type for "give me a sequence of characters to read" 
// Works with Vec, arrays, and other slices
// It is like &str but for numeric values 
where
    F: Fn(f64) -> f64
{
    scores.iter().map(|score| processor(*score)).collect()
    // We iterate over the slice
    // Transform each score
    // Call the processor on each score
    // Collect into a Vec<f64>
}

fn main() {

    // Creating a course
    let mut course = Course::new("Intro to Rust".to_string(), 70.0);

    // Create students and add scores
    let mut alice = Student::new("Alice".to_string(), 101);
    alice.add_score(85.0);
    alice.add_score(90.0);
    alice.add_score(78.0);
    
    let mut bob = Student::new("Bob".to_string(), 102);
    bob.add_score(65.0);
    bob.add_score(70.0);
    bob.add_score(68.0);
    
    let mut charlie = Student::new("Charlie".to_string(), 103);
    charlie.add_score(92.0);
    charlie.add_score(88.0);
    charlie.add_score(95.0);
    
    // Add students to course
    course.add_student(alice);
    course.add_student(bob);
    course.add_student(charlie);

    // Print passing students
    println!("Passing students:");
    for student in course.get_passing_students() {
        println!("  - {}", student.name);
    }
    
    // Get top performers
    println!("\nTop performers (>= 85 average):");
    for name in course.get_top_performers(85.0) {
        println!("  - {}", name);
    }
    
    // Apply curve to entire course

    // Here, we are creating a function from our closure factory
    // This function returns impl Fn(f64) -> f64, so it will satisfy the constraint required by .apply_curve_to_all()
    // It is going to create a function called add_curve which will always add 5 points to our score
    // We then use the add_curve function to add 5 points to every score
    let add_curve = create_add_curve(5.0);
    course.apply_curve_to_all(&add_curve);
    println!("\nAfter adding 5 point curve:");
    
    // Print passing students again
    println!("Passing students:");
    for student in course.get_passing_students() {
        println!("  - {}", student.name);
    }
    
    // Use process_scores
    let scores = vec![75.0, 80.0, 85.0];
    // Here, instead of making a function from our closure factory, we are putting in a custom closure
    // The closure takes the score (x) and multiplies it by a factor of 1.1
    // Think of the || as parentheses for a function
    // The input parameter (scores) goes inside of the bars ||
    // When comes after the bars is like the function body, which will take the score and multiply it by 1.1
    let boosted = process_scores(&scores, |x| x * 1.1);
    // Here, we are doing &scores so that it satisfies the type annotation
    // It is a slice of f64 values
    // We achieve the by taking the whole vector and making a reference to it
    // Can also be achieved by taking a slice of the vector
    println!("\nBoosted scores: {:?}", boosted);

}

