struct FileParser<T> {
    file_path: String,
    content: String,
    data: Vec<T>,
}

struct Person {
    name: String,
    age: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum ParseError {
    FileNotFound,
    InvalidFormat,
    EmptyFile,
}

// If we do not add Sized here, the compiler will give us an error
// The error will say that Self needs to have a known size at compile time
// Result needs to know how much memory to allocate Self
// By adding Sized, you're telling Rust: "Only types with a known size at compile time can implement this trait."
// This works for the Person struct because Person has a known size at compile time - String and u32 
// Rust can calculate the total size of a Person at compile time
// Trait objects (dyn Trait) and Slices (dynamic length) and str (dynamic length) don't have a known size
trait Parseable: Sized {
    // When we return Result<Self, ...> we are saying:
    // This method will either successfully create and return an instance of the type implementing this trait or return an error
    // Self refers to whatever type is implementing this trait
    // "Parse this line and give me back and instance of whatever type you are."
    fn parse(line: &str) -> Result<Self, ParseError>; // Either create an instance or error
    // Using Self (capital S) refers to the type implementing the trait
    // Self is a placeholder for "the type implementing this trait"
    // Using self (lowercase s) refers to an instance of the type

    // Person::parse(...) creates a Person instance
    // Student::parse(...) creates a Student instance

    // &str is called a string slice
    // It is a reference to an unsized string slice type representing UTF-8 text
    // String is owned, stored on the heap, you can change it, has to free when goes out of scope
    // &str is a borrowed reference, points into a String or string literal, read only, does not own data
}

impl Parseable for Person {
    // This argument is &str since we want the function to accept any string reference, not just static ones
    // &str accepts a reference to a String or a string literal
    // String literals like "hello" are already &str (technically &'static str)
    // String is owned data, so you need to borrow it with & to get &str
    // &str accepts string slices, which you get from: string literals, borrowing a string, or slicing either of those
    // For input parameters, we usually use &str it is the most flexible
    // The rust pattern is input parameters &str and return values or owned data (String)
    // String owns data on the heap, &str borrows data from anywhere, and &'static str borrows data that lives forever (program's binary)
    fn parse(line: &str) -> Result<Self, ParseError> {
        // .collect() is part of Rust's iterator system
        // It turns an iterator into a collection
        // It consumes an iterator and builds a data structure from it, like vector, string, hashmap, etc.
        // Because collect has different types, Rust often needs a type annotation
        // .collect() works on collection types that implement the FromIterator trait
        let parts: Vec<&str> = line.split(',').collect();
        let name = parts[0].to_string();

        // If this fails, meaning it encounters the error variant,
        // The return keyword immediately exits the function and returns the error variant
        // If it succeeds, it will assign the number extracted to the age variable
        let age = match parts[1].parse::<u32>() {
            Ok(num) => num,
            // In an error arm, return short circuits the function and returns the error immediately
            Err(_) => return Err(ParseError::InvalidFormat),
        };

        // If this method succeeds, it will return an instance of the Person struct
        // with fields name and age
        Ok(Self {
            name,
            age,
        })

        // The question mark is shorthand for error handling
        // If parsing succeeds, give me the u32 value. If it fails, immediately return the error from this function
        // This is used instead of matching
            // let age = parts[1].parse::<u32>()?;
        // The ? operator works in functions that return Result or Option
        // This is a clean way of propagating errors up the call stack without writing verbose match statements

        // For the ? operator to work in this method, it would need to return ParseError when it errors, which it does not

            // let age = match parts[1].parse::<u32>() {
            //     Ok(num) => num,
            //     Err(_) => return Err(ParseError::InvalidFormat),
            // };


    }
}

// We implemented Parseable for Person but since FileParser<T> is generic,
// When we create FileParser<Person>, the T becomes Person
impl<T: Parseable> FileParser<T> {
    // The trait bound here says that FileParser can only be generic over types T that implement Parseable
    // So, we are guaranteed that T::parse() exists and can be called
    // If someone tries FileParser<String>, it would fail at compile time because String doesn't implement Parseable

    // Self here is an alias for FileParser<T>
    // We could also write here -> FileParser<T> but it is more verbose
    // And if we change the name of the struct, we would then have to update this
    fn new(file_path: String) -> Self {
        Self {
            file_path,
            content: "".to_string(),
            data: Vec::new(),
        }
    }

    fn read_file(&mut self) -> Result<(), ParseError> {
        // We can also use 2 if statements here rather than if-else with early returns
        // We do not need return here since the if-else block is an expression and the last expression in each branch is automatically returned
        if self.file_path.is_empty() {
            Err(ParseError::FileNotFound)
        } else if self.content.is_empty() {
            Err(ParseError::EmptyFile)
        } else {
            Ok(())
        }
    }

    // This is more idiomatic because early returns reduce nesting and make the code easier to read
    // You check conditions and bail immediately if they fail
    // It's called the "guard clause" pattern and is very common in Rust
    fn _read_file_two(&mut self) -> Result<(), ParseError> {
        // Here we need return because we are exiting the function early, not relying on the if-else structure to return
        // return keyword = explicitly exit the function early
        // No return = rely on implicit return of the last expression
        if self.file_path.is_empty() {
            return Err(ParseError::FileNotFound);
        }
        if self.content.is_empty() {
            return Err(ParseError::EmptyFile);
        }
        Ok(())
    }

    // If we successfully create a Person struct from the contents, push it to the data vector of type Vec<Person>
    // Otherwise, give an error
    fn parse_lines(&mut self) -> Result<(), ParseError> {
        let lines: Vec<&str> = self.content.split('\n').collect();
        for line in lines {
            if line.is_empty() {
                // continue skips the rest of the current loop iteration and moves to the next one
                // If a line is empty, continue jumps to the next iteration
                // It never reaches the match statement for the empty line
                // It is useful for skipping over data you don't want to process
                continue;
            }
            // Since T::parse() returns Self{name, age}
            // We are saying that, on success, push the Person{name, age} struct to the data vector
            // The data vector is Vec<T> so when we create a FileParser<Person>, it will be Vec<Person>
            // so the vector will already match the expected data type and we can push Person structs to it
            match T::parse(line) {
                // We are using :: notation since it is used for associated functions (functions that don't take self)
                // . is used for methods (functions that take self, &self, &mut self)
                // Since ::parse() returns an instance of Person and doesn't need an existing instance to work
                // It is an associated function and we use :: rather than .
                // Associated functions create or work on the type itself
                // Methods work on instances of the type
                Ok(person) => self.data.push(person),
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

fn main() {
    // Any string literal in double quotes is always a &'static str because:
    // The actual text is embedded in your compiled binary
    // It exists for the entire program lifetime
    // It's immutable and can't be modified
    // The reason we see &str instead of &'static str in type annotations is:
    // &'static str is a subtype of &str (with any lifetime)
    // Writing &str is more flexible - accepts Strings and borrowed strings
    // Rust's type inference handles the details
    let sample_data = "Alice,30\nBob,25\nCharlie,35\n\nDiana,28";

    // Here, since Person implements the Parseable trait, our parse_line() method will work without any issues
    let mut parser: FileParser<Person> = FileParser::new("people.txt".to_string());

    parser.content = sample_data.to_string();

    match parser.read_file() {
        Ok(()) => println!("File read successfully."),
        // e is just a generic variable name
        // it can be more descriptive
        // we use _ when we want to ignore the error
        // e is acceptable in short, simpler code, but more descriptive names are better for readability
        Err(e) => println!("Error reading file: {:?}", e),
    }

    match parser.parse_lines() {
        Ok(()) => println!("Lines parsed successfully."),
        Err(e) => println!("Error parsing lines: {:?}", e),
    }

    println!("\nParsed People:");
    for person in &parser.data {
        println!("Name: {}, Age: {}", person.name, person.age);
    }
}
