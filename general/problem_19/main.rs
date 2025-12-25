use std::fmt;

#[derive(Clone)]
enum Temperature {
    Celsius(f64),
    Fahrenheit(f64),
    Kelvin(f64),
}

// Here, we are implementing the Display trait for temperature
// The display trait allows you to define how your custom type can be formatted
// Display works with the {} placeholder in things like println!() or format!()
// Without the Display trait, you would get a compiler error
impl fmt::Display for Temperature {
    // This is the required signature for the formatter
    // &self is an immutable borrow
    // f is a mutable borrow to the fmt::Formatter - it is a blank canvas and each write!() will append something
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // write!() takes the formatter f, then a format string, and then a value to interpolate
            // write!() does not add a new line only writeln!() does
            Temperature::Celsius(value) => write!(f, "{}°C", value),
            // This entire match statement is an expression and we want to return it, so that is why we do not need a semicolon
            // There is an implicit return
            Temperature::Fahrenheit(value) => write!(f, "{}°F", value),
            Temperature::Kelvin(value) => write!(f, "{}K", value,)
        }
    }
}

fn f_to_c(f: f64) -> f64 {
    (f - 32.0) * 5.0/9.0
}

fn k_to_c(k: f64) -> f64 {
    k - 273.15
}

// This struct will be a wrapper for our temperature conversions to Celsius
struct CelsiusTemp(Temperature); // This is called a tuple struct
// This is because the fields have no names just types
// We access field positions using .0, .1
// We use parentheses instead of {}
// Besides named/regular structs with fields, and tuple structs, there are also unit structs with no fields at all
// struct Marker;

// Here, we are converting the From trait to convert between Temperature and CelsiumTemp
// This will allow us to use .into() and ::from() to convert between them
// To convert using .into(), we would do Temperature into CelsiusTemp with a type annotation = let converted: CelsiusTemp = Temperature::Kelvin(f64).into()
// This is saying: "Convert Temperature INTO CelsiusTemp"
// To convert using ::from, we would do: CelsiusTemp::from(Temperature::Kelvin(f64))
// This is saying: "Convert FROM Temperature to CelsiusTemp"
impl From<Temperature> for CelsiusTemp {
    // Here, we are using Self as the return annotation since we are working on the type itself
    // We are essentially instantiating a new instance of the struct CelsiusTemp through the conversion
    fn from(temperature: Temperature) -> Self {
        match temperature {
            Temperature::Celsius(value) => CelsiusTemp(Temperature::Celsius(value)),
            Temperature::Fahrenheit(value) => CelsiusTemp(Temperature::Celsius(f_to_c(value))),
            Temperature::Kelvin(value) => CelsiusTemp(Temperature::Celsius(k_to_c(value))),
        }
    }
}

// In this TryFrom block, we are converting f64 into a Temperature::Kelvin
// But only if the value is valid (>=0) since Kelvin can't be negative
// If invalid, return an error
// This will be from f64 into Kelvin

// The TryFrom trait signature is:
// pub trait TryFrom<T>: Sized {
//     type Error;  // You must define what error type to use
//     fn try_from(value: T) -> Result<Self, Self::Error>;
// }
impl TryFrom<f64> for Temperature {
    // First, we have to define the error type
    // We will be using a String for simplicity
    type Error = String;

    // Now, we will implement the converstion function
    // We are returning Self since we are working on the type itself -> Temperature
    // It will either create an instance of the Temperature type or provide an error (that we defined before)
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        // First, we check if the value is valid for Kelvin (>=0)
        if value < 0.0 {
            // We do not need return since if/else is an expression in Rust - it returns a value
            // The last expression in each branch is automatically returned
            // No semicolons needed - the values flow naturally
            // We would use return with an early return (exiting before the end of the function)
            Err(format!("Kelvin cannot be negative. Got: {}", value))
        } else {
            Ok(Temperature::Kelvin(value)) // Here, we are creating a new instance of temperature (Self) from f64
        }
    }
}

// From: can't fail, returns Self, no error type needed, use .into(), use Type::from()
// TryFrom: can fail, returns Result<Self, Error>, must define type Error, use .try_into(), use Type::try_from()
// As a note, you can implement both From and TryFrom on the same type - this is a common pattern when you have some conversions that always succeed and some that might fail

struct TemperatureConverter {
    readings: Vec<Temperature>,
}

impl TemperatureConverter {
    fn new() -> Self {
        Self {
            readings: Vec::new()
        }
    }

    fn add_reading(&mut self, temp: Temperature) {
        self.readings.push(temp)
    }

    fn convert_all_to_celsius(&self) -> Vec<Temperature> {
        self.readings.iter().map(|temperature| {
            // Here, we need to first do .into() with a type hint 
            // We are converting Temperature INTO CelsiusTemp
            let celsius: CelsiusTemp = (*temperature).clone().into();
            // celsius.0 because Temperature is stored at position 0 in the struct -> struct CelsiusTemp(Temperature)
            // .into() will give us the CelsiusTemp wrapper, not Temperature
            // We need to access the first and only field of the tuple struct
            celsius.0
        }).collect()
    }

    fn get_average_celsius(&self) -> Option<f64> {
        if self.readings.is_empty() {
            return None
        }

        // Here, we are calling the convert_all_to_celsius() method that we defined previously instead of rewriting all of the logic
        let all_celsius: Vec<Temperature> = self.convert_all_to_celsius();

        // We cannot do math directly on enums
        // We need to pattern match to extract the numeric value instead and then do math on those numbers
        // This is why we have a match statement inside of .map()
        let total: f64 = all_celsius.iter().map(|temperature| {
            match temperature {
                Temperature::Celsius(value) => *value,
                _ => 0.0,
            }
        }).sum();

        // all_celsius.len() returns usize (unsigned integer type for sizes/lengths)
        // total is a f64 (floating point)
        // You can't divide different numeric types in Rust
        // as f64 converts usize -> f64 (such as 5 -> 5.0)
        // Rust is strict about types - there are no automatic conversions 
        // We must explicitly cast with as
        Some(total / all_celsius.len() as f64)
    }
}

fn main() {
    
    let c: Temperature = Temperature::Celsius(16.0); // Instantiating several Temperature structs
    let f: Temperature = Temperature::Fahrenheit(32.0);
    let k: Temperature = Temperature::Kelvin(64.0);

    println!("{}", c); // testing the formatting from our Display trait implementation
    println!("{}", f);
    println!("{}", k);

    let f_to_c: CelsiusTemp = f.clone().into();
    println!("{}", f_to_c.0);

    let k_to_c: CelsiusTemp = k.clone().into();
    println!("{}", k_to_c.0);

    let valid_temp: f64 = 10.0;
    let invalid_temp: f64 = -10.0;

    // Since .try_into() returns a Result, we need to handle the Result
    // using either match or .unwrap()
    // Remember, .unwrap() is only good for when are 100% sure it will succeed, otherwise the program will panic
    let k_valid_temp: Temperature = valid_temp.try_into().unwrap();
    println!("{}", k_valid_temp);

    match Temperature::try_from(invalid_temp) {
        Ok(temp) => println!("Valid: {}", temp),
        Err(e) => println!("Error: {}", e),
    };

    let mut temp_conv: TemperatureConverter = TemperatureConverter::new();

    temp_conv.add_reading(c);
    temp_conv.add_reading(f);
    temp_conv.add_reading(k);

    let all_celsius: Vec<Temperature> = temp_conv.convert_all_to_celsius();

    // This will print only the value itself, not the enum
    // Which does not use the display trait
    for temp in &all_celsius {
        match temp {
            Temperature::Celsius(value) => println!("{}", value),
            _ => {},
        }
    }

    // This will print using the Display trait since it is the actual enum variant
    // Not just the value inside
    for temp in &all_celsius {
        println!("{}", temp);
    }

    match temp_conv.get_average_celsius() {
        Some(value) => println!("Average celsius: {}", value),
        None => println!("Nothing")
    };
    
 }
