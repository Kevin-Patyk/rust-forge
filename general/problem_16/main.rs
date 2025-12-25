trait Plugin {
    // &str is a string slice - a reference to a string
    // For methods that read or return data without modifying, &str is preferred because
    // it's more flexible (works with string literals, owned Strings, etc.)
    // It's efficient (no copying)
    // It signals: "I'm not taking ownership"
    fn name(&self) -> &str;
    // &str can reference string literals (&'static str), an owned String, a slice of a String
    // &'static str is a specific type of &str that lives for the entire program
    // Regular &str can have any lifetime
    // String can be coerced to &str automatically
    // &str is a universal string reference that works with multiple sources
    fn execute(&mut self, input: &str) -> Result<String, PluginError>;
    // Every string literal is a &'static str (let s = "hello";)
    // They are hardcoded into the binary at compile time, so they exist for the entire program's lifetime
    // All &'static str are &str
    // Not all &str are &'static str (they could have shorter lifetimes)
}
// A string literal is written directly into the code (type is &'static str)
    // let s: &str = "hello";  // this is a string literal

// A string slice is a reference to some UTF-8 text
// can refer to: part of a String, part of a &str, a string literal
// type is &str but not static
    // let s = String::from("hello");
    // let slice: &str = &s[0..2];  // string slice

// A string literal (&'static str) is a kind of string slice (&str) but not all string slices (&str) are string literals (&'static str)

// The Debug trait allows us to print with the Debug format: {:?}
// If we wanted to implement the Display trait, we would have to do it ourselves manually
// We generally use Debug for development and debugging 
// We use Display for user-facing messages since it is more polish and readable
#[derive(Debug)]
#[allow(dead_code)]
enum PluginError {
    ExecutionFailed(String), // This will include an error message in this enum variant
    InvalidInput,
}

// These structs are empty (no fields) because they don't need to store any state
// They just need to implement the plugin trait
struct UpperCasePlugin; // This is a concrete type, not a trait object - it has a known size and type
struct ReversePlugin;
// Box<dyn Plugin> is a trait object
// They say: I don't care what concrete type it is, just that it implements Plugin

impl Plugin for UpperCasePlugin {
    fn name(&self) -> &str {
        "Uppercase"
    }

    // We are using &mut self for the type signature since the trait is generic and designed for any plugin
    // Even though our current ones don't, some plugins might need to track state and modify internal data
    // &mut self allows flexibility for all possible plugin implementations
    // When designing a trait, use &mut self for any implementation that might need mutability
    fn execute(&mut self, input: &str) -> Result<String, PluginError> {
        if input.is_empty() {
            return Err(PluginError::ExecutionFailed(
                "Something went wrong: details here".to_string()
            ));
        }
        Ok(input.to_uppercase())
    }
}

impl Plugin for ReversePlugin {
    fn name(&self) -> &str {
        "Reverse"
    }
    
    fn execute(&mut self, input: &str) -> Result<String, PluginError> {
        if input.is_empty() {
            return Err(PluginError::ExecutionFailed(
                "Something went wrong: details here".to_string()
            ));
        } 
        Ok(input.chars().rev().collect())
    }
}

struct PluginManager {
    // Box<dyn Plugin> means:
    // dyn Plugin - "any type that implements the Plugin trait" (dynamic dispatch)
    // Box<...> - heap allocated pointer to that trait objects
    // All together, this means: "A vector of pointers to any types that implement plugin"
    plugins: Vec<Box<dyn Plugin>>,
    // As a note, we need Box since Rust doesn't know the size of dyn Plugin at compile time (could be any type)
    // Box allocates memory on the heap and stores a pointer, which has a known fixed size
    // The correct method gets called at runtime based on what the type actually is
    execution_history: Vec<String>,
}

impl PluginManager {
    fn new() -> Self {
        Self {
            // The type is defined in the struct, so when you create an instance, like below,
            // Rust infers the correct types for the fields
            plugins: Vec::new(),
            execution_history: Vec::new(),
        }
    }

    fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin)
    }

    fn execute_plugin(&mut self, plugin_name: &str, input: &str) -> Result<String, PluginError> {        
        // We are using &mut self.plugins because the function signature requires &mut self
        // Also, execute() requires &mut self
        // The mutability of your loop variable must match what the methods you're calling require
        for plugin in &mut self.plugins {
            if plugin.name() == plugin_name {
                // This uses the ? operator to unwrap the Result
                // If it's Ok(value), you get the String value
                // If it's Err(e), the function returns that error immediately
                // This will return the error provided by execute()
                let result = plugin.execute(input)?;
                // The ? operator does:
                // calls plugin.execute(input) which returns Result<String, PluginError>
                // If it's an Ok(value), it extracts the String and assigns it to result
                // If it's an Err(e), the ? immedatiately returns the error from execute_plugin()
                // The error "bubbles up" with all its data preserved - the ? operator propagates it exactly as it
                self.execution_history.push(format!("{}: {}", plugin_name, result));
                // We then wrap it in Ok() before returning
                return Ok(result)
                // You can with or without the return here
                // return is explicit - exit the function now
                // Without return, it's an implicit return - the last expression in the function is returned
                // It is best to use an explicit return when you need to exit early, like a loop
                // Use implicit at the end of the function
            }
        }
        Err(PluginError::InvalidInput)
    }

    fn list_plugins(&self) -> Vec<&str> {
        // Here, we are iterating through the plugins, calling name() on each, and collecting the results into a Vec
        // So, we got through each Plugin struct present in the vector, call the name() method, and collect it into a Vec
        let plugins: Vec<&str> = self.plugins.iter().map(|plugin| plugin.name()).collect();
        plugins
    }

    fn _get_history(&self) -> &Vec<String> {
        &self.execution_history
    }
}

fn main() {
    let mut manager = PluginManager::new();

    // We are using Box here since we need to wrap the concrete type in a Box so it becomes a trait object
    // "I have this specific plugin type, but I'm wrapping it in a pointer so it can go in a vector of any plugin type"
    // Also, register_plugin() requires a Box<dyn Plugin>, so we do not need a type annotation
    manager.register_plugin(Box::new(UpperCasePlugin));
    // We need Box or a pointer for trait objects because Rust doesn't know the size at compile time
    // It could be any type
    // We always need to use some kind of pointer with dyn, but Box is the most common choice for owned trait objects
    manager.register_plugin(Box::new(ReversePlugin));
    // When we do Box::new(UpperCasePlugin), we are converting the concrete type into a trait object wrapped in a pointer
    // UpperCasePlugin = concrete type that implements Plugin
    // Box<dyn Plugin> = trait object (pointer to any type that implements plugin)
    // Once the concrete type is wrapped in Box<dyn Plugin>, the concrete type still exists but it can only
    // be accessed through trait methods - you lose access to type-specific methods
    // from specific type - any type that implements this trait

    // before wrapping let plugin = UpperCasePlugin;
    // You can call any method specific to UpperCasePlugin

    // After wrapping it let plugin: Box<dyn Plugin> = Box::new(UpperCasePlugin);
    // You can ONLY call methods from the Plugin trait
    // We cannot call any UpperCasePlugin-specific methods 

    println!("Available plugins:");
    // Since list_plugins(), returns a Vec<&str>, we are iterating over it to print the names of the plugins
    for name in manager.list_plugins() {
        println!("{}", name);
    }

    match manager.execute_plugin("Uppercase", "hello") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("{:?}", e),
    }

    match manager.execute_plugin("Uppercase", "") {
        Ok(result) => println!("Result: {}", result),
        // Since this will fail due to input being an empty string ("")
        // The error from execute() will be propagated up
        Err(e) => println!("{:?}", e),
    }

    match manager.execute_plugin("Reverse", "hello") {
        Ok(result) => println!("{}", result),
        // As a note, {:?} is called the Debug format
        Err(e) => println!("{:?}", e),
    }

    match manager.execute_plugin("Reverse", "") {
        Ok(result) => println!("{}", result),
        // Since this will fail due to input being an empty string ("")
        // The error from execute() will be propagated up
        Err(e) => println!("{:?}", e),
    }

    match manager.execute_plugin("NonExistent", "hello") {
        Ok(result) => println!("{}", result),
        // In this case, the error will come from execute_plugin() and not execute()
        // This is because the for loop will not match any existing plugins names (it will not find a match)
        // So Err(PluginError::InvalidInput) will come up
        Err(e) => println!("{:?}", e),
    }

    // When we have several structs with different fields (different sizes) that all implement the same trait
    // We need box<dyn Trait> since Rust will not know exactly which concrete type we are referring to
    // and Rust needs to have a known size at compile time
    // Box<dyn Trait>:
    // Box is heap allocation, pointer with known size
    // dyn - dynamic dispatch (runtime polymorphism) -> we will find out the type during runtime - This is a trait object - we will determine the concrete type dynamically at runtime
    // Trait - the trait being implemented
    // We aren't able to put a Trait in a vector, like Vec<Trait> since it is a trait, not a concrete type - Rust doesn't know the size at compile time
    // With Vec<Box<dyn Trait>>, Box has a known size (pointer size)

    // Box<dyn Trait> means: "A heap-allocated pointer to some type that implements Trait, we'll figure out which type at runtime"
    // "Put a Box (heap pointer) around an unknown-sized type that implements the Trait"
}