struct LogEntry {
    timestamp: String,
    level: LogLevel,
    message: String,
}

// This is an attribute that automatically implements the PartialEq trait
// Attributes are pieces of metadata that tells the Rust compiler how to treat your code
// This is telling the compiler: "Automatically generate code that implements the PartialEq trait for this enum"
// #[derive(...)] attributes automatically implement traits for you
// Instead of manually writing all the code to implement these traits, the derive attribute does it for you automatically
#[derive(PartialEq)]
// This automatically implements the Debug trait for your enum or struct
// The debug trait lets you print values using {:?} in println! macros
// Without it, you can't print enums or structs directly - youd get an error
// With this, Rust generates the code to print your type
#[derive(Debug)]
enum LogLevel {
    Info,
    Warning,
    Error,
}

// When we define this struct, Rust already knows that a Logger has a field called
// entries that is a Vec of LegEntry structs
// So when we call Logger::new(), Rust always knows that entries should be Vec<LogEntry> based on this struct definition
// The empty vector is created with the correct type automatically
struct Logger {
    entries: Vec<LogEntry>,
}

impl Logger {
    // new() is a constructor method it - it creates and returns a new instance of the struct
    // new() is the convention in Rust for constructors
    // -> Logger returns a logger struct
    // Logger {} creates and returns a new logger instance with an empty vector for the entries field
    // This can then be used with let logger = Logger::new();
    // new() is technically an associated function because it doesn't take self - it just creates a new instance
    fn new() -> Logger {
        Logger { entries: Vec::new() }
    }

    // We are using &mut self because log() needs to modify the Logger struct by adding a new entry
    // to its entries vector
    // &mut self gives you a mutable reference to the struct, so you can call methods on it that change the state
    // if we used &self instead, we would only be able to read from the Logger not modify it
    // Rust wouldn't let us .push() to the vector because that changes the struct
    fn log(&mut self, level: LogLevel, message: String) {
        // When we call this method, it will create a LogEntry struct and push it to the vector
        let timestamp = "2025-11-08 10:30:45".to_string();
        self.entries.push(LogEntry {
            // In Rust, when the field name matches the variable name, you can use shorthand
            // meaning not timestamp: timestamp
            timestamp,
            level,
            message,
        });
    }

    fn get_entries_by_level(&self, level:LogLevel) -> Vec<&LogEntry> {
        // A closure is a small anonymous function
        // |entry| - this is the parameter which is each item from the iterator
        // entry.level == level is the logic (returns true or false)
        // .filter() takes a closure and runs it on each item
        // If the closure returns true, the item is kept
        // If it returns false, the item is filtered out 
        self.entries.iter().filter(|entry| entry.level == level).collect()
        // .iter() creates an iterator that goes through each LogEntry struct in self.entries one by one
        // .filter() checks each struct's level field and keeps only ones that much
        // .collect() gathers all the matching structs (as references) into a Vec

        // .find() returns Option<T> - it stops at the first match and returns just that one item
        // .filter() returns an iterator of all items that match that condition

        // Thus, get_entries_by_level() returns a Vec of references LogEntry structs where the LogLevel matches the level we input
        // So if there are 5 structs in entries with the same log level, this will return a Vec<&LogEntry> of length 5 
    }

    fn clear(&mut self) {
        // .clear() removes all items from the vector without deallocating memory
        // The vector still exists, but it's just emptry
        self.entries.clear()

        // You can also do:
        // self.entries = Vec::new();
        // This creates a brand new empty vector and replaces the old one
        // .clear() is slightly more efficient because it reuses the existing memory
        // .clear() is also more idiomatic
    }
}

fn main() {
    // Here we are using the constructor method to create an empty Logger instance
    // This empty logger instance with instantiate the entries field with an empty vector
    let mut logger = Logger::new();

    // Each call to log() creates a new LogEntry and adds it to the Logger's entries vector
    // Which, in this case, was just created above
    // We are not manually creating LogEntry structs - the log() method is doing that for us
    logger.log(LogLevel::Info, "Application Started".to_string());
    logger.log(LogLevel::Warning, "Low Memory Detected".to_string());
    logger.log(LogLevel::Error, "Database Connection Failed.".to_string());
    logger.log(LogLevel::Error, "Database Connection Failed.".to_string());

    let levels: Vec<LogLevel> = vec![LogLevel::Info, LogLevel::Warning, LogLevel::Error];

    for level in levels {
        let entries = logger.get_entries_by_level(level);
        for entry in entries {
            println!("{}: {:?} - {}", entry.timestamp, entry.level, entry.message);
        }
    }

    logger.clear();
}
