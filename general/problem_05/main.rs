#[derive(Debug)]
enum Operations {
    Add,
    Subtract,
    Multiply,
    Divide,
}

// This automatically implements the Debug trait for your enum
// This lets you print it using {:?} in print statements
// Without it, you wouldn't be able to print your enum with println!()
#[derive(Debug)]
enum CalculatorError {
    DivisionByZero
}
// It is generally good practice to put errors in an enum
// It makes error handling explicit and type-safe
// The caller knows exactly what kinds of errors are possible
// This is more idiomatic Rust than just returning a generic error message

fn calculate(a: f64, b: f64, operation: &Operations) -> Result<f64, CalculatorError> {
    match operation {
        // Ok() is the other variant of the Result enum that represents a successful operation
        // When you return Ok(value), you're saying: "the operation succeeded and here's the result"
        // The calling code can then use match or .unwrap() to handle the case
        // However, .unwrap() is not safe because if you call it on an Err() variant it will panic and crash
        // .unwrap() works for the success case but it's generally not recommended because it doesn't handle errors gracefully
        // match is more idiomatic
        Operations::Add => Ok(a + b),
        Operations::Subtract => Ok(a - b),
        Operations::Multiply => Ok(a * b),
        // This match arm first checks if b is equal to 0 using an if statement inside the match arm
        // If b is 0, it returns an Err() variant because you can't divide by 0
        // In Rust, Err() is a variant of the Result enum that represents a failure or error case
        // When you return Err(something) you're saying: This operation failed and here's the error information"
        Operations::Divide => if b == 0.0 {
            Err(CalculatorError::DivisionByZero)
        } else {
            Ok(a / b)
        }
    }
}

fn main() {
    
    let operations_vec: Vec<(f64, f64, Operations)> = vec![
        (10.0, 2.0, Operations::Add), // 12
        (10.0, 2.0, Operations::Subtract), // 8
        (10.0, 2.0, Operations::Multiply), // 20
        (10.0, 0.0, Operations::Divide), // Error
    ];

    // When we write (a, b, operation) in operations_vec, we are destructuring
    // Rust is breaking apart each tuple in the vector and extracting the individuals values into a, b, and operation
    // Instead of getting a single tuple, you get 3 separate variables to work with
    for (a, b, operation) in operations_vec {
        match calculate(a, b, &operation) {
            Ok(value) => println!("{} {} {:?} {}", a, b, operation, value),
            Err(error) => println!("{} {} {:?} {:?}", a, b, operation, error)
        }
    }
}
