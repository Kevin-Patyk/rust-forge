#![allow(dead_code)]
// The use statement brings the fmt module into scope
// std = standard library (comes with Rust)
// fmt = formatting module
// We need it for implementing the Display trait
use std::fmt;

struct Item {
    name: String, // Owned, growable text
    // All &'static str are &str but not all &str are &'static str
    // &str can point to string literals, Strings, or slices of &str and Strings
    // Anything in double quotation marks, like "hello", lives in the programs binary and is &'static str
    price: f64,
    quantity: u32,
}

#[derive(Debug)]
enum CartError {
    EmptyCart,
    InvalidDiscount(String), // Enum variants containing information
    ItemNotFound(String),
}

// The Default trait provides a way to create "default" or "zero" value for a type
// It's like a constructor that creates a sensible starting state
// Think of it as: "What should this type look like when it's empty or initialized with no data"
impl Default for Item {
    // This is the required signature for the Default trait
    // It is useful for creating consistent inititial state across code
    // Common pattern for "empty" structs
    // Works with generic code that needs to create default values
    // Alternative to always writing constructors
    // You can use it several ways with ::default();
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            price: 0.0,
            quantity: 0,
        }
    }
    // Use Default when there's an obvious zero or empty state
    // Use new() when you need parameters or complex initialization
}

struct ShoppingCart {
    items: Vec<Item>, // Vector of Item structs
    discount_percent: f64,
}

impl Default for ShoppingCart {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            discount_percent: 0.0,
        }
    }
}

// Here, we are implementing the Display trait for Item
// This is so it can be displayed in println!() or made into a String with format!()
// This is how our custom type should be formatted
impl fmt::Display for Item {
    // This is the required signature
    // An immutable borrow to self
    // A mutable borrow to f, the formatter -> blank canvas that we are appending to using write!()
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
        // The first argument is the formatter
        // The second argument is a format string
        // The last arguments are values to interpolate
        write!(f, "Item: {} (${} x {})", self.name, self.price, self.quantity)
        // We are not using a semicolon here due to the implicit return of fmt::Result
        
        // We would want to use to ? operator if we had multiple write!() statements
        // Then we would also have Ok(()) at the end
        // We are not using ? and Ok(()) here since write!() already returns fmt::Result
        // When you add ?, it unwraps the Result and gives you ()
        // But our function needs to return fmt::Result not ()
        // With a single write!() dont use ?, just return it directly
     }
}

fn apply_discount(amount: f64, discount: f64) -> Result<f64, CartError> {
    if discount < 0.0 || discount > 100.0 {
        return Err(CartError::InvalidDiscount(format!("Discount percent must be between 0 and 100. Got {}.", discount)))
    }

    let discount_multiplier = 1.0 - (discount / 100.0);

    Ok(amount * discount_multiplier)
}

impl ShoppingCart {
    fn new() -> Self {
        ShoppingCart::default()
    }

    fn add_item(&mut self, item: Item) {
        self.items.push(item)
    }

    fn set_discount(&mut self, percent: f64) -> Result<(), CartError> {
        if percent < 0.0 || percent > 100.0 {
            return Err(CartError::InvalidDiscount(format!("Discount percent must be between 0 and 100. Got {}.", percent)))
        }

        self.discount_percent = percent;
        Ok(())
    }

    fn calculate_subtotal(&self) -> Result<f64, CartError> {
        if self.items.is_empty() {
            return Err(CartError::EmptyCart)
        }

        let total_price: f64 = self.items.iter().map(|item| item.price * item.quantity as f64).sum();

        Ok(total_price)
    }

    // When a function returns Result<T, SomeError>, any statement using ? must return a Result
    // with the same error type (SomeError)
    // Every method/function you call with ? must return Result<T, CartError> or whatever error type your function returns
    // The Ok type (T) can be different but the Err type must match
    fn calculate_total(&self) -> Result<f64, CartError> {
        // calculate_subtotal() returns Result<f64, CartError>
        // The ? operator here is used instead of a match statement
        // If it is the Ok() variant, it unwraps it and assigns it to the subtotal variable
        // If it is the error variant, it will propagate the error as defined in calculate_subtotal() which is EmptyCart
        // This is error propagation - both functions return the same error type
        let subtotal = self.calculate_subtotal()?;

        let total = apply_discount(subtotal, self.discount_percent)?;

        Ok(total)
    }

    fn get_item_count(&self) -> usize {
        self.items.len()
    }

    fn get_average_item_price(&self) -> Result<f64, CartError> {
        // calculate_subtotal() returns Result<f64, CartError>
        // The ? operator here is used instead of a match statement
        // If it is the Ok() variant, it unwraps it and assigns it to the subtotal variable
        // If it is the error variant, it will propagate the error as defined in calculate_subtotal()
        // This is error propagation - both functions return the same error type
        let subtotal = self.calculate_subtotal()?;

        Ok(subtotal / self.items.len() as f64)
    }
}

fn process_cart(cart: &ShoppingCart) -> Result<(), CartError> {
    // Here we are using ? for error propagation
    // Easier to use than a match statement
    // It will unwrap the Ok() variant and assign it to the variable
    // or it will propagate the error, which will all be on type CartError
    // The semicolon will make it a statement
    // It will not return anything and keep going
    let subtotal = cart.calculate_subtotal()?;
    let total = cart.calculate_total()?;
    let aip = cart.get_average_item_price()?;

    // The {:.2} is format precision for floating point numbers
    // {} is the normal placeholder
    // :.2 format with 2 decimal places
    // {:.2} displays a floating point number with exactly 2 decimal places
    println!("Subtotal: ${:.2}", subtotal);
    println!("Total: ${:.2}", total);
    println!("AIP: ${:.2}", aip);

    Ok(())
}

fn main() {
    let mut shopping_cart = ShoppingCart::default();

    let item1 = Item { 
        name: "one".to_string(),
        price: 10.0,
        quantity: 1,
     };

     let item2 = Item {
        name: "two".to_string(),
        price: 20.0,
        quantity: 1,
     };

     let item3 = Item {
        name: "three".to_string(),
        price: 30.0,
        quantity: 1,
     };

     let item4 = Item::default();

     shopping_cart.add_item(item1);
     shopping_cart.add_item(item2);
     shopping_cart.add_item(item3);
     shopping_cart.add_item(item4);

     println!("Item count: {}", shopping_cart.get_item_count());

     match process_cart(&shopping_cart) {
        Ok(()) => println!("Cart processed."),
        Err(e) => println!("{:?}", e),
     }

     match shopping_cart.set_discount(10.0) {
        Ok(()) => println!("Discount set."),
        Err(e) => println!("{:?}", e),
     }

     match shopping_cart.set_discount(-10.0) {
        Ok(()) => println!("Discount set."),
        Err(e) => println!("{:?}", e),
     }

     match shopping_cart.calculate_subtotal() {
        Ok(subtotal) => println!("Subtotal: {}", subtotal),
        Err(e) => println!("{:?}", e),
     }

     match shopping_cart.calculate_total() {
        Ok(total) => println!("Total: {}", total),
        Err(e) => println!("{:?}", e),
     }

     match shopping_cart.get_average_item_price() {
        Ok(price) => println!("AIP: {}", price),
        Err(e) => println!("{:?}", e),
     }

     let empty_cart = ShoppingCart {
        items: Vec::new(),
        discount_percent: 0.0,
     };

     match empty_cart.calculate_subtotal() {
        Ok(subtotal) => println!("Subtotal: {}", subtotal),
        Err(e) => println!("{:?}", e),
     }
}
