#![allow(dead_code)]
use std::ops::{Deref, DerefMut};

struct Config {
    app_name: String,
    version: String,
    debug_mode: bool,
    max_connections: u32,
}

// A trait is a way to define shared behavior that multiple types can implement
// It lets you specify methods (functions) that a type must provide
// A trait is a collection of methods and associated items that define behavior a type can implement, enabling both static and dynamic polymorphism
// Polymorphism - many forms - the ability to write code that works with different data types through a shared interface or behavior
trait ConfigSource {
    fn load(&self) -> Result<Config, String>;
    fn name(&self) -> &str;
    // Default implementation
    fn is_ready(&self) -> bool {
        true
    }
}

// Regular named field structs
struct FileConfig { path: String}
struct EnvConfig { path: String }
// Unit struct
// Even with no data, you can implement methods on unit structs
struct DefaultConfig;

impl ConfigSource for FileConfig {
    fn load(&self) -> Result<Config, String> {
        if self.path.is_empty() {
            Err("File config path is empty. Cannot produce a configuration with an empty path.".to_string())
        } else {
            Ok(Config {
                app_name: "FileApp".to_string(),
                version: self.path.clone(), // clone the path so it does not transfer ownership/move it
                debug_mode: true,
                max_connections: 0,
            })
        }
    }

    fn name(&self) -> &str {
        "FileApp"
    }

    // Even if we override a default method, the signature still must match exactly
    fn is_ready(&self) -> bool {
        !self.path.is_empty() // not empty = not ready
    }
}

impl ConfigSource for EnvConfig {
    fn load(&self) -> Result<Config, String> {
        if self.path.is_empty() {
            Err("File config path is empty. Cannot produce a configuration with an empty path.".to_string())
        } else {
            Ok(Config {
                app_name: "EnvApp".to_string(),
                version: self.path.clone(), // clone the path so it does not transfer ownership/move it
                debug_mode: true,
                max_connections: 0,
            })
        }
    }

    fn name(&self) -> &str {
        "EnvApp"
    }

    // Default implementation for is_ready()
}

impl ConfigSource for DefaultConfig {
    fn load(&self) -> Result<Config, String> {
        // No validation needed - just return sensible defaults
        Ok(Config {
            app_name: "DefaultApp".to_string(),
            version: "1.0.0".to_string(),
            debug_mode: false,
            max_connections: 100,
        })

        // If the function's return type is Result<T, E>, you can just return Ok(value) when everything succeeds
        // There is no requirements to ever return Err, as long as your function returns Result
        // Returning an Ok value is valid when success is guaranteed or errors arent possible
    }

    fn name(&self) -> &str {
        "Default"
    }

    // Default implementation for is_ready()
}

struct ConfigManager {
    // dyn Trait is a trait object - a way to use dynamic dispatch with traits
    // dyn Trait means that the vector accepts any type that implements Trait, but the concrete type is only known at runtime
    // This allows for heterogenous collections, runtime polymorphism, and store different types behind a single interface
    // We nee Box<> around dyn Trait because trait objects have an unknown size and Rust requires all values to have a known size at compile time
    sources: Vec<Box<dyn ConfigSource>>,
    active_config: Option<Config>,
}

// Now, we are creating a wrapper for Config
// This is the newtype pattern
// The newtype pattern is good for semantic meaning, enforcing invariants/criteria, API clarity, and going around Rust's orphan rule
struct ConfigWrapper(Config);

// Deref makes the wrapper type behave like the value inside it, letting you use methods of the inner type without manual unwrapping or * everywhere
// Rust will automatically call the Deref trait for you so you do not need to do it yourself
// Dereferencing means accessing a value the pointer points to
impl Deref for ConfigWrapper {

    // This is the associated type - what we deref to
    // "When you Deref ConfigWrapper you get Config"
    type Target = Config; // <- What you get when you deref

    fn deref(&self) -> &Config {
        &self.0 // Access the first and only inner item (Config)
    }
}

// For DerefMut to work, Deref must first be implemented
// &mut self takes a mutable reference to the wrapper
// Returns a mutable reference to the inner wrapper
// &mut self.0 - mutable reference to the Config inside
impl DerefMut for ConfigWrapper {
    fn deref_mut(&mut self) -> &mut Config {
        &mut self.0
    }

    // This is just like Deref, but for mutable access to the inner type
    // We can now do something like:
    // let config = Config { ... }
    // let wrapper = ConfigWrapper(config)
    // println!("App: {}", wrapper.app_name) <- This is just reading (uses Deref)
    // wrapper.app_name = "NewApp".to_string() <- This is writing (uses DerefMut)

}

impl ConfigManager {
    fn new() -> Self {
        Self {
            sources: Vec::new(),
            active_config: None,
        }
    }

    // The source here a trait object - any type that implements the ConfigSource trait
    // The concrete type is only known at runtime
    // We need Box<> since trait objects don't have a known size compile time and Rust needs to know the size
    fn add_source(&mut self, source: Box<dyn ConfigSource>) {
        self.sources.push(source);
    }

    fn load_from(&mut self, index: usize) -> Result<(), String> {
        // You cannot call .is_empty() on a trait object, so we have to use this
        if index >= self.sources.len() {
            return Err(format!("Index {} is out of bounds", index));
        }

        // Since .load() returns a Result<Config, String>, the ? operator will either
        // store the result in the config variable if it is Ok(Config)
        // Or it will throw an error
        // The error will propagate since they are both of the Result type (default Rust error)
        let config = self.sources[index].load()?;

        self.active_config = Some(config);

        Ok(())
    }

    // Alternative with .get()
    fn load_from_two(&mut self, index: usize) -> Result<(), String> {
        // .get() returns an Option<&T> and is a safe way to access elements by index in a vector
        // It will either return Some(&elements) or None if out of bounds
        // With direct indexing, if index >= length, the program panics and crashes
        // Using .get() allows you to handle errors gracefully if they occur - no panic
        match self.sources.get(index) {
            Some(source) => {
                let config = source.load()?;
                self.active_config = Some(config);
                Ok(())
            }
            None => Err(format!("No source at index {}.", index))
        }
    }

    fn get_config(&self) -> Option<&Config> {
        // Using this is simpler than matching or using if let 
        // self.active_config is already an Option<Config>
        // .as_ref() converts Option<Config> into Option<&Config>
        // .as_ref() is a method that converts an owned value into a reference
        // It is commonly used on Option<T> to make it Option<&T>
        // It is also used on Result<T, E> to make it Result<&T, E> without taking ownership
        self.active_config.as_ref()
    }

    fn list_sources(&self) -> Vec<&str> {
        self.sources.iter().map(|source| source.name()).collect()
    }
}

struct ConfigBuilder {
    app_name: Option<String>,
    version: Option<String>,
    debug_mode: Option<bool>,
    max_connections: Option<u32>,
}

impl ConfigBuilder {
    // We are using mut self instead of &mut self
    // mut self is correct for builder methods
    // Regular methods use &self or &mut self
    // Builder patterns need ownership transfer for clean chaining
    fn app_name(mut self, app_name: String) -> Self {
        self.app_name = Some(app_name);
        self
    }

    // Each method:
    // 1. Takes ownership (mut self)
    // 2. Modifies the builder
    // 3. Returns ownership back (Self)
    fn version(mut self, version: String) -> Self {
        self.version = Some(version);
        self // Enables chaining
    }

    fn debug_mode(mut self, debug_mode: bool) -> Self {
        self.debug_mode = Some(debug_mode);
        self
    }

    fn max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = Some(max_connections);
        self
    }

    fn build(self) -> Config {
        Config {
            // .unwrap_or() is a method on Option<T> that returns the value inside of Some or a default it it's None
            // It is good for providing fallback values 
            // Not good for expensive default computation since it is eager
            // If it is an expensive default computation, use .unwrap_or_else() instead 
            app_name: self.app_name.unwrap_or("DefaultApp".to_string()),
            version: self.version.unwrap_or("0.0.1".to_string()),
            debug_mode: self.debug_mode.unwrap_or(false),
            max_connections: self.max_connections.unwrap_or(10),
        }
    }
}

fn main() {
    println!("Hello, world!");
}
