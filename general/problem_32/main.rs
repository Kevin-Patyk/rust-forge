#![allow(dead_code)]
#![allow(unused_variables)]

use std::fs;
use std::io;
use std::fmt;
use serde_json::Value;

#[derive(Debug)]
enum ProcessError {
    IoError(io::Error),
    JsonError(String),
    ValidationError(String),
    ProcessingError{ field: String, reason: String}, // struct-like variant 
}

// fmt::Formatter is a buffer/writer that Rust uses to build formatted strings
// When you implement Display or Debug, Rust gives you this formatter to write to
// Think of it like a blank canvas - its where you write your formatted output, Rust manages it internally, and you write it piece by piece using write! or writeln!
// The &mut is import because we are mutating the formatter - each write!() adds more text to the internal buffer
// You append to the formatter and in the end you get a complete string

// Implementing the Display trait for our error enum
// Implementing the display trait allows you to define how your custom type is formatted
// Display works with the {} placeholder in format strings like println!() and format!()
// Without Display, you'd get a compiler error because it doesn't implement the Display trait
impl fmt::Display for ProcessError {
    // This is the required signature
    // We are borrowing the value we are formatting (&self)
    // f is the formatter you write to
    // fmt::Result returns either Ok or an error
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // self is the ProcessError enum - it could be any of the variants
            ProcessError::IoError(error) => write!(f, "IO Processor Error Encountered: {}.", error),
            // The write!() macro writes formatted text to the formatter
            // The first argument is always f - the formatter
            // Then a format string like println!
            // Then any values to interpolate
            ProcessError::JsonError(message) => write!(f, "Json Processor Error Encountered: {}.", message),
            ProcessError::ValidationError(message) => write!(f, "Validation Processor Error Encountered: {}.", message),
            ProcessError::ProcessingError{ field, reason } => write!(f, "General Processing Error Encountered: Field {}, Reason {}.", field, reason),
        }
    }
}

// Implement From traits for automatic conversion between errors
// This is how to convert from an io::Error to a ProcessError
impl From<io::Error> for ProcessError {
    // We are returning Self, which refers to the type itself
    // The type itself in this case is a ProcessError
    // We use Self since we will be returning an instance of ProcessError
    // We use Self in impl blocks because it makes refactoring easier if you rename the type - you don't have to change it everywhere
    fn from(error: io::Error) -> Self {
        // Now, if we use the ? operator on something that returns Result<T, io::Error>, Rust will automatically convert it for us
        ProcessError::IoError(error)
    }
}

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    email: String,
}

// This will return either the User or a ProcessError
fn validate_user(name: &str, age: i32, email: &str) -> Result<User, ProcessError> {
    // Validate name is not empty
    if name.trim().is_empty() {
        return Err(ProcessError::ValidationError("Name cannot be empty".to_string()));
    }
    
    // Validate age is positive
    if age <= 0 {
        return Err(ProcessError::ProcessingError {
            field: "age".to_string(),
            reason: format!("Age must be positive, got {}", age),
        });
    }
    
    // Validate email contains @
    if !email.contains('@') {
        return Err(ProcessError::ProcessingError {
            field: "email".to_string(),
            reason: "Email must contain @".to_string(),
        });
    }
    
    Ok(User {
        name: name.to_string(),
        age: age as u32,
        email: email.to_string(),
    })
}

// Main processing pipeline
// &str is a string slice - a reference to a string
// We use &str since it is the most flexible - it can be a string literal, owned String, slice of a String
// It is efficient - no copying - it signals "I am not taking ownership"
// &'static str is a type of &str that lives for the entire program
// &str is a universal string reference that works with multiple sources
fn process_user_file(path: &str) -> Result<Vec<User>, ProcessError> {
    // When encountering any issues in this function, all need to be converted to ProcessError
    // This allows the ? operator to work seamlessly throughout the function
    // This is because our function signature expects this
    // If something returns a different error type, this would not work

    // Reading the file
    // The ? operator will automatically convert an io::Error to ProcessorError
    let contents = fs::read_to_string(path)?;

    // Parsing the JSON - we need to handle this error
    // We need to convert the serde_json::Error into ProcessError::JsonError
    let json: Value = match serde_json::from_str(&contents) {
        // If it is the Ok() variant, we assign val to val and unwrap it, returning the value
        Ok(val) => val,
        // If it is the error variant, we convert it to a ProcessError
        Err(e) => return Err(ProcessError::JsonError(e.to_string())),

    };

    // Now, extract the array of users
    // As before, we need to convert the error type
    let users_array = match json.as_array() {
        Some(arr) => arr,
        None => return Err(ProcessError::JsonError("Expected an array".to_string())),
    };
    
    let mut users = Vec::new();

    for user_value in users_array {
        let name = user_value["name"].as_str()
            .ok_or_else(|| ProcessError::JsonError("Missing name field".to_string()))?;
        // .ok_or_else() is a method on Option<T> that converts an option into a Result, lazily creating the error only if needed
        // So, this is saying, if there is no name field, create and return an error
        
        let age = user_value["age"].as_i64()
            .ok_or_else(|| ProcessError::JsonError("Missing age field".to_string()))?;
        
        let email = user_value["email"].as_str()
            .ok_or_else(|| ProcessError::JsonError("Missing email field".to_string()))?;


        // Validate and add user - ? propagates any validation errors
        // Remember that the validation error function also returns a Result with a ProcessError, so these can be safely propagated
        // and they will work with our function
        let user = validate_user(name, age as i32, email)?;
        users.push(user);
    };

    Ok(users)
}

fn main() {
    // Example 1: Successful validation
    match validate_user("Alice", 30, "alice@example.com") {
        Ok(user) => println!("Valid user: {:?}", user),
        Err(e) => println!("Error: {}", e),
    }
    
    // Example 2: Empty name error
    match validate_user("", 25, "bob@example.com") {
        Ok(user) => println!("Valid user: {:?}", user),
        Err(e) => println!("Error: {}", e),
    }
    
    // Example 3: Negative age error
    match validate_user("Charlie", -5, "charlie@example.com") {
        Ok(user) => println!("Valid user: {:?}", user),
        Err(e) => println!("Error: {}", e),
    }
    
    // Example 4: Invalid email error
    match validate_user("Diana", 28, "invalid-email") {
        Ok(user) => println!("Valid user: {:?}", user),
        Err(e) => println!("Error: {}", e),
    }
    
    // Example 5: File processing (create a test file first)
    // Create users.json with: [{"name":"Alice","age":30,"email":"alice@example.com"}]
    match process_user_file("users.json") {
        Ok(users) => println!("Loaded {} users: {:?}", users.len(), users),
        Err(e) => println!("File processing error: {}", e),
    }
    
    // Example 6: Missing file error
    match process_user_file("nonexistent.json") {
        Ok(users) => println!("Loaded users: {:?}", users),
        Err(e) => println!("Error: {}", e),
    }
}

// The ? operator is both for error propagation and error conversion

// If there is an error, it immediately returns it from the current function
    // let contents = fs::read_to_string(path)?;
    // If this fails, the function stops here and returns the error
    // No need to write: match ... { Ok(v) => v, Err(e) => return Err(e) }

// If the error type doesn't match your function's return type, ? automatically converts it:
    // fn process_user_file(path: &str) -> Result<Vec<User>, ProcessError> {
    //     let contents = fs::read_to_string(path)?;
        //                                      ^
        // fs::read_to_string returns Result<String, io::Error>
        // But our function returns Result<Vec<User>, ProcessError>
        // 
        // The ? operator sees this mismatch and calls:
        // ProcessError::from(io_error)
        // 
        // This works because we implemented From<io::Error> for ProcessError!
    // }

// Error propagation means passing an error up the call stack to the caller, instead of handling it in the current function
// "I ecountered an error but instead of dealing with it myself, I'm going to return it to whoever called me and let them decide what to do"
// Example:
    // fn read_file(path: &str) -> Result<String, io::Error> {
    //     fs::read_to_string(path)? // Propagates error to caller
    // }

    // fn process_file(path: &str) -> Result<(), io::Error> {
    //     let contents = read_file(path)?; // Propagates error to caller
    //     println!("{}", contents);
    //     Ok(())
    // }

    // fn main() {
    //     match process_file("test.txt") {
    //         Ok(_) => println!("Success!"),
    //         Err(e) => println!("Error handled here: {}", e), // Finally handled!
    //     }
    // }
// 1. fs::read_to_string encounters an error
// 2. ? propagates it to read_file's caller, which is process_file
    // a. if main was calling read_file, then it would be the caller
// 3. read_file's ? propagates to process_file's caller
// 4. process_file's ? propagates it to main
// 5. main finally handles the error with match

// The key point: The ? operator keeps bubbling up until someone uses match, if let, .unwrap()
// or some other way to actually handle the Result instead of propagating it further
