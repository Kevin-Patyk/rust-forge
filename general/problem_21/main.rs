#![allow(dead_code)]
// We are bringing the standard library into scope
// This will make it so that we can implement the Display trait
use std::fmt;
use std::fmt::Display;
struct Book {
    title: String,
    author: String,
    isbn: String,
    quantity: u32,
}

struct Electronic {
    name: String,
    brand: String,
    model: String,
    quantity: u32,
}

// Here we are creating a trait
// A trait is like a contract - any type that wants to implement this trait needs to have these signatures
// The implementations can be different but the signature must be the same
// They are used to define shared behavior
// Traits are good for polymorphism - static dispatch - you can write functions that accept any type implementing a trait
// Dynamic dispatch - trait objects (boxed interfaces)
trait Storable {
    // We are returning a string slice
    // It is a reference & to a sequence of UTF-8 characters stored somewhere else
    // Usually in a String or the program's binary
    // Without owning the data
    fn get_name(&self) -> &str;
    fn get_quantity(&self) -> u32;
    fn set_quantity(&mut self, qty: u32);
}

impl Storable for Book {
    fn get_name(&self) -> &str {
        &self.title
    }

    fn get_quantity(&self) -> u32 {
        self.quantity
    }

    fn set_quantity(&mut self, qty: u32) {
        // Since we are not returning anything (no return annotation)
        // We are using a semicolon to make it a statement since it will return nothing
        // Without the semicolon, it would be an expression and return a u32
        self.quantity = qty;
    }
}


impl Storable for Electronic {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_quantity(&self) -> u32 {
        self.quantity
    }

    fn set_quantity(&mut self, qty: u32) {
        self.quantity = qty;
    }
}

// Here, we are implementing the Display trait
// This will allow us to set up formatting for our custom type
// This is to make it displayable when using println!({}) or format!({}) for string
impl fmt::Display for Book {
    // f is a mutable reference to the formatter
    // blank canvas we are appending onto using write!() macros
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Here we are not using ? or a semicolon since write!() returns fmt::Result
        // We would use multiple write!() lines with ? and semicolons if there were multiple write!() statements
        // Then we would need Ok(()) at the end
        write!(f, "Book: '{}' by {} (ISBN: {}) - Qty: {}", self.title, self.author, self.isbn, self.quantity)
    }
}

impl fmt::Display for Electronic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The formatter f is the first argument
        // Followed by a format string
        // Then values to interpolate
        write!(f, "Electronic: {} by {} (Model: {}) - Qty: {}", self.name, self.brand, self.model, self.quantity)
    }
}

// This is a generic struct 
// Inventory is generic over type T
// T is a placeholder for any type
// When you create an instance, you specify what T is
// "Inventory can hold any type of items in a vector, as long as you tell me what type when you create it"
// This allows Inventory to work with Books and Electronics
struct Inventory<T> {
    items: Vec<T>,
    // usize is an unsigned integer type whose sized depends on your computer's architecture
    // It can only be positive (0 and up)
    // It is used for sizes and indices - anything related to memory/collections
    capacity: usize,
}

// We have a generic struct Inventory<T> - it can hold anything
// However, some of our methods depend on trait bounds 
// So, despite that Inventory is generic, some of the methods will only work on types that implement Storable
// This provides flexible generics with compile-time safety

// Here we are implementing methods for generic Inventory<T>
impl<T> Inventory<T> {
    // We do not need where because we are not calling any Storable methods
    // We are not accessing trait specific functionality
    // We are just creating an empty vector and storing a capacity
    // Vec::new() works for any type T and doesn't need Storable
    fn new(capacity: usize) -> Self {
        Self {
            items: Vec::new(),
            capacity,
        }
    }

    // where is the keyword that starts the trait bounds section
    // constraint: "T must implement the Storable trait"
    //An alterative syntax without where: impl<T: Storable> Inventory<T> 
    // Where is preferred with multiple constraints or for readability
    fn add_item(&mut self, item: T) -> Result<(), String> 
    // As a note, we don't actually need a trait bound here since we are
    // only using generic functionality and not calling any Storable methods
    where 
        T: Storable,
    {
        if self.items.len() >= self.capacity {
            Err("Inventory is at full capacity".to_string())
        } else {
            self.items.push(item);
            Ok(())
        }
    }

    fn remove_item(&mut self, name: &str) -> Result<T, String> 
    where
        T: Storable
    {
        // Find the index of the item with matching name 
        // if let Some(index) pattern matches the option 
        // If found, extract the index and remove
        // If not found, return error
        // We could also use a match statement 
        // .position() returns an Option<usize>, this is why we need to use if let or match
        // "If this is Some(index), extract the index and run this block"
        // "If Some(index) is returned from self.items.iter()..., run this block."
        // We use if let when we only care about ONE variant (usually 'Some')
        // You want simpler snytax than full match
        // You have an else case to handle the other variant
        if let Some(index) = self.items.iter().position(|item| item.get_name() == name) {
            // Remove and return the item at that index
            // .remove() removes the item at that index and returns it
            // This is perfect because we return Result<T, String>
            Ok(self.items.remove(index))
        } else {
            Err(format!("Item {} not found.", name))
        }
    }

    // We need T: Storable since we are using Storable trait metohds, like .get_quantity() and .get_name()
    // If you call methods from a trait -> need trait bounds
    // If you only use generic functionality -> no trait bound needed
    fn get_total_quantity(&self) -> u32 
    where
        T: Storable
    {
        self.items.iter().map(|item| item.get_quantity()).sum()
    }

    fn find_item(&self, name: &str) -> Option<&T> 
    where
        T: Storable
    {   
        // We are using .find() here since we need to return the Item, not the index (which .position() returns)
        self.items.iter().find(|item| item.get_name() == name)
    }

    fn is_full(&self) -> bool {
        self.items.len() >= self.capacity // Don't need an if-else since this already returns a boolean
    }
}

// This is a generic function with multiple trait bounds
// "This function works with any Inventory<T> as long as the type T implements both the Storable and Display traits"
// The + means AND for trait bounds
fn print_inventory<T>(inventory: &Inventory<T>) 
where
    T: Storable + Display // T must implement both Storable and Display
{   
    // Without the borrow, we would move items out of the inventory and we do not want to
    // What we are doing in the loop should match what we have in the function signature
    for item in &inventory.items {
        println!("{}", item)
    }
}

// This returns a Vector of references
// We would need to iterate over it to then get the names of the low stock items
    // for item in low_stock {
    //     println!("Low stock: {}", item.get_name());
    // }
fn get_low_stock<T>(inventory: &Inventory<T>, threshold: u32) -> Vec<&T> 
where
    T: Storable
{   
    // We are not using .filter().map() here since we want to SELECT items not TRANSFORM them
    // We would use .filter().map(), for example, to select low stock items and transform them to names
    inventory.items.iter().filter(|item| item.get_quantity() <= threshold).collect()
}

fn main() {
    // We are making new Inventories with Book and Eletronic types
    // We need a type hint since Inventory is generic
    // Rust needs to know what type of Inventory we are creating
    let mut _book_inv: Inventory<Book> = Inventory::new(10);
    let mut _ele_inv: Inventory<Electronic> = Inventory::new(5);


}
