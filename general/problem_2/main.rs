// We are using an enum because we need a Vec that holds both numbers and text
// Vectors require all elements to be of the same type, so you wouldnt be able to hold both numbers and strings
// An enum lets you create a custom type that can represent multiple possibilities
enum FizzBuzzValue {
    Number(u32),
    Text(String),
}

fn main() {
    let n = 21;

    // Making an empty vector to handle to values of the enum that we made
    let mut results: Vec<FizzBuzzValue> = Vec::new();

    // .push() adds an element to the end of a Vector
    // Each .push() appends the value to the end of the Vector
    // The vector needs to be mutable in order to use .push() because you're modifying it
    for num in 1..=n {
        if num % 15 == 0 {
            // We are converting to string here since we need to convert &str string literals
            // into owned String values, as is expected by the enum
            results.push(FizzBuzzValue::Text("FizzBuzz".to_string()));
        } else if num % 5 == 0 {
            results.push(FizzBuzzValue::Text("Buzz".to_string()));
        } else if num % 3 == 0 {
            results.push(FizzBuzzValue::Text("Fizz".to_string()));
        } else {
            results.push(FizzBuzzValue::Number(num));
        }
    }

    for value in results {
        match value {
            // On the left hand side of the arm, that is the pattern to match
            // It says: If the value is the Number variant of FizzBuzzValue, then n captures the number inside that variable (destructuring)
            // => means "then do this"
            // Which in this case is printing the value inside the enum variant
            FizzBuzzValue::Number(n) => println!("{}", n),
            FizzBuzzValue::Text(s) => println!("{}", s),

            // If the value is a number, extract the number inside of it and print it
            // If the value is text, extract the string inside of it and print it
            // The match statement checks which variant the value is and runs the corresponding arm
        }

        // match is used when you need to handle different cases or variants of a value, such as with
        // enums, option types, result types (error handling), and pattern matching
        // use match when you have multiple distinct cases (especially enums)
        // use if/else for simple boolean conditions
        // match is more powerful and idiomatic in Rust, especially for enums
    }
}
