#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;

enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    List(Vec<ConfigValue>) // Recursive variant
}

enum ConfigError {
    ParseError(String),
    MissingKey(String),
    TypeError { expected: String, found: String },
    ValidationError(String),
}

// This is a type alias - a shorthand name for a longer type
// The first part is the new short name
// The second part is the actual type it represents
// You can use it like: fn get_string(&self, key: &str) -> ConfigResult<String> { }
type ConfigResult<T> = Result<T, ConfigError>;
// <T> is still generic - it is the success type 
// ConfigResult<String>  = Result<String, ConfigError>
// Using type aliases is less repetitive, easier to change, more readable, and a common convention in Rust libraries

struct Config {
    data: HashMap<String, ConfigValue>,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::ParseError(string) => write!(f, "Parsing Error Encountered: {}", string),
            ConfigError::MissingKey(string) => write!(f, "Missing Required Key: {}", string),
            // Just as with the enum variant, you do not need parentheses around the curly braces
            ConfigError::TypeError {expected, found} => write!(f, "Type Error Encountered. Expected: {}, Found: {}", expected, found),
            ConfigError::ValidationError(string) => write!(f, "Validation Error Encountered: {}", string),
        }
    }
}

// This converts parsing errors into ConfigError
// This allows using ? with parse operations
impl From<std::num::ParseIntError> for ConfigError {
    fn from(parse_error: std::num::ParseIntError) -> Self {
        ConfigError::ParseError(parse_error.to_string())
    }

    // You can now use ? with From:
    // s.parse::<i64>()? -> returns Result<i64, ParseIntError>
    // But your function return requires ConfigResult<i64> = Result<i64, ConfigError>
    // ? operator automatically sees the error types don't match
    // Automatically calls the .into() on the error
    // .into() uses our From implementation
    // ParseIntError gets converted to ConfigError
}

impl Config {
    fn new() -> Self {
        Self {
            data: HashMap::new()
        }
    }

    fn set(&mut self, key: String, value: ConfigValue) {
        // The .insert() method inserts or updates the key-value pair
        self.data.insert(key, value);
        // We are using the semi-colon here since we are not returning anything
        // It is now a statement, which doesn't return anything in Rust
    }

    fn get(&self, key: &str) -> ConfigResult<&ConfigValue> {
        // The .get() method gets a value by a key
        // self.data.get(key) returns an Option<&ConfigValue> 
        // .ok_or_else() is a method for converts an Option into a Result
        // If Some(value) -> Ok(Value)
        // If None -> Err(whatever the closure returns)
        // The _else suffix means "use a closure" (lazy evaluation)
        self.data.get(key).ok_or_else(|| ConfigError::MissingKey(key.to_string()))
    }

    // Either returns an Ok(String) or an Err(ConfigError)
    fn get_string(&self, key: &str) -> ConfigResult<String> {

        // Get the value -> this returns ConfigResult<&ConfigValue>
        // If this fails to find the key, it propagates the MissingKey error
        let value = self.get(key)?;

        // If value matches any other variant rather than String, it will throw an error
        // Both the error from value (get()) (MissingKeyError) and TypeError are ConfigErrors
        // They both match the ConfigResult<T> type
        // We would not be able to use a different error outside of ConfigError here since the type would not match the return annotation
        match value {
            ConfigValue::String(string) => Ok(string.clone()),
            ConfigValue::Integer(_) => Err(ConfigError::TypeError {
                expected: "String".to_string(),
                found: "Integer".to_string(),
            }),
            ConfigValue::Float(_) => Err(ConfigError::TypeError {
                expected: "String".to_string(),
                found: "Float".to_string(),
            }),
            ConfigValue::Boolean(_) => Err(ConfigError::TypeError {
                expected: "String".to_string(),
                found: "Boolean".to_string(),
            }),
            ConfigValue::List(_) => Err(ConfigError::TypeError {
                expected: "String".to_string(),
                found: "List".to_string(),
            })
        }
    }

    fn get_int(&self, key: &str) -> ConfigResult<i64> {
        let value = self.get(key)?;
        match value {
            // Here _, the wildcard pattern, means: I don't care about this value, just ignore it
            // It's a wildcard pattern that matches but doesn't bind the value to a variable
            // match but ignore the string - you don't care about the value
            // "Something goes here, but I'm not going to use it"
            ConfigValue::String(_) => Err(ConfigError::TypeError {
                expected: "Integer".to_string(),
                found: "String".to_string(),
            }),
            ConfigValue::Integer(integer) => Ok(*integer),
            ConfigValue::Float(_) => Err(ConfigError::TypeError {
                expected: "Integer".to_string(),
                found: "Float".to_string(),
            }),
            ConfigValue::Boolean(_) => Err(ConfigError::TypeError {
                expected: "Integer".to_string(),
                found: "Boolean".to_string(),
            }),
            ConfigValue::List(_) => Err(ConfigError::TypeError {
                expected: "Integer".to_string(),
                found: "List".to_string(),
            })
        }
    }

    fn get_float(&self, key: &str) -> ConfigResult<f64> {
        let value = self.get(key)?;
        match value {
            ConfigValue::String(_) => Err(ConfigError::TypeError {
                expected: "Float".into(),
                found: "String".into(),
            }),
            ConfigValue::Integer(_) => Err(ConfigError::TypeError {
                expected: "Float".into(),
                found: "Integer".into(),
            }),
            ConfigValue::Float(f) => Ok(*f),
            ConfigValue::Boolean(_) => Err(ConfigError::TypeError {
                expected: "Float".into(),
                found: "Boolean".into(),
            }),
            ConfigValue::List(_) => Err(ConfigError::TypeError {
                expected: "Float".into(),
                found: "List".into(),
            }),
        }  
    }

    fn get_bool(&self, key: &str) -> ConfigResult<bool> {
        let value = self.get(key)?;
        match value {
            // Here _, the wildcard pattern, means: I don't care about this value, just ignore it
            // It's a wildcard pattern that matches but doesn't bind the value to a variable
            // match but ignore the string - you don't care about the value
            // "Something goes here, but I'm not going to use it"
            ConfigValue::String(_) => Err(ConfigError::TypeError {
                expected: "Boolean".to_string(),
                found: "String".to_string(),
            }),
            ConfigValue::Integer(_) => Err(ConfigError::TypeError {
                expected: "Boolean".to_string(),
                found: "Integer".to_string(),
            }),
            ConfigValue::Float(_) => Err(ConfigError::TypeError {
                expected: "Boolean".to_string(),
                found: "Float".to_string(),
            }),
            ConfigValue::Boolean(boolean) => Ok(*boolean),
            ConfigValue::List(_) => Err(ConfigError::TypeError {
                expected: "Boolean".to_string(),
                found: "List".to_string(),
            })
        }
    }
    }


fn validate_positive(value: i64) -> ConfigResult<i64> {
    if value <= 0 {
        Err(ConfigError::ValidationError(format!("Value cannot be <= 0. Got {}.", value)))
    } else {
        Ok(value)
    }
}

fn validate_in_range(value: i64, min: i64, max: i64) -> ConfigResult<i64> {
    if value < min || value > max {
        Err(ConfigError::ValidationError(format!("Value cannot exceed the provided range: min {}, max {}. Got {}", min, max, value)))
    } else {
        Ok(value)
    }
}

fn parse_and_validate_port(s: &str) -> ConfigResult<i64> {
    // This makes use of the ? operator for automatic From conversion, that we defined before
    // If a std::num::ParseIntError is encountered 
    // it is automatically converted into ConfigError::ParseError(parse_error.to_string())
    // Rust will automatically call .into() which uses our From impl
    // enables seamless propagation across different error types
    let port = s.parse::<i64>()?;

    // Since validate_in_range() also produces a ConfigResult<i64>, it will propagate the error
    // if something goes wrong
    validate_in_range(port, 1, 65535)?;

    Ok(port)
}

struct ConfigBuilder {
    data: HashMap<String, ConfigValue>
}

impl ConfigBuilder {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    // We are using mut self because we want to modify the instance and return a modified version
    // self is returning the instance we modified
    // self takes ownership - enables chaining by consuming and returning
    // mut allows for modification
    // Together they consume, modify, return
    // Builder methods always consume self 
    // Builder is no longer useful after building - youve extracted the final product
    // prevents from calling .build() twice
    // moves ownership -> transfers the data to the final struct
    // The entire builder pattern is built on consuming self
    fn set_string(mut self, key: String, value: String) -> Self {
        self.data.insert(key, ConfigValue::String(value));
        self
    }

    fn set_int(mut self, key: String, value: i64) -> Self {
        self.data.insert(key, ConfigValue::Integer(value));
        self
    }

    fn set_float(mut self, key: String, value: f64) -> Self {
        self.data.insert(key, ConfigValue::Float(value));
        self
    }

    fn set_bool(mut self, key: String, value: bool) -> Self {
        self.data.insert(key, ConfigValue::Boolean(value));
        self
    }

    fn build(self) -> Config {
        Config {
            data: self.data
        }
    }
}

struct ServerConfig {
    host: String,
    port: i64,
    debug: bool,
}

fn load_server_config(config: &Config) -> ConfigResult<ServerConfig> {
    
    // If "host" does not contain a String value (ConfigValue::String()), we will return a ConfigError
    // .get_string() returns ConfigResult<T> which returns ConfigError if error encountered
    let host = config.get_string("host")?;
    // If "port" does not contain an int value (ConfigValue::Integer()), we will return a ConfigError (ConfigResult)
    let port = config.get_int("port")?;
    // If the port is not in range, return a ConfigError (ConfigResult)
    validate_in_range(port, 1, 65535)?;
    // If "debug" does not contain a boolean (ConfigValue::Boolean()), we will return a ConfigError (ConfigResult)
    let debug = config.get_bool("debug")?;

    // All of our methods and functions return the same type of error ConfigResult<T> = Result<T, ConfigError>
    // The ? operator is short-hand for match - it will either return Ok() and unwrap the value and assign it
    // Or it will return an error -> ConfigError
    // Since all of these return the same error variant we defined, the return annotation is satisfied
    // This is a good demonstration of error propagation in Rust

    Ok(ServerConfig { host, port, debug })
}

fn main() {
    println!("Hello, world!");
}
