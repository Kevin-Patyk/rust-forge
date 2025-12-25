#![allow(dead_code, unused_variables)]

use std::ops::Deref;

struct Item {
    name: String,
    quantity: u32,
    price: f64,
}

enum Category {
    Electronics,
    Clothing,
    Food,
    Other,
}

struct CategorizedItem {
    item: Item,
    category: Category,
}

// This is a newtype or tuple struct
// It is a struct with one unnamed field
// The field is Box<Item> (heap-allocated Item)
// We can access the inner value with .0: item_ref.0
struct ItemRef(Box<Item>);
// This creates a distinct type instead of using Box<Item> directly

trait Valuable {
    // Associated constant - each implementor provides their own value
    // Each type that implements valuable must provide this value
    // Accessed with Self::MIN_VALUE inside trait method
    // Accessed with TYPE::MIN_VALUE outside trait
    const MIN_VALUE: f64;
    // A constant in Rust is an immutable value that's known at compile time
    // Immutable, no memory address, type must be annotated, screaming snake case
    // constants are perfect for configuration that never changes
    // constants with references have a 'static lifetime but the constant itself is inlined rather than stored in a single location like a true static

    // Required method - must be implemented by each type
    // Each implementor MUST provide implementation
    fn total_value(&self) -> f64;

    // Default implementation - can be used as is or overriden
    // Implementors can use this default or override it
    fn is_expensive(&self) -> bool {
        self.total_value() > Self::MIN_VALUE
    }
}

impl Valuable for Item {
    const MIN_VALUE: f64 = 100.0;

    fn total_value(&self) -> f64 {
        self.price * self.quantity as f64
    }

    // .is_expensive() uses the default implementation automatically
}

impl Valuable for CategorizedItem {
    const MIN_VALUE: f64 = 150.0;

    fn total_value(&self) -> f64 {
        self.item.price * self.item.quantity as f64
    }

    // Here, we are overriding the .is_expensive() method to make our own custom implementation
    fn is_expensive(&self) -> bool {
        match self.category {
            Category::Electronics => self.total_value() > 200.0,
            Category::Clothing => self.total_value() > 100.0,
            Category::Food => self.total_value() > 50.0,
            Category::Other => self.total_value() > Self::MIN_VALUE,
        }
    }
}

// The Deref trait allows a type to behave like a reference to another type
// How a type behaves when it is deferenced using the * operator 
// It is commonly used by smart pointers 
// Dereferencing means accessing a value that a pointer points to
// "The Deref trait lets a wrapper type automatically act like the type it wraps, so you can call methods on the inner type without manually unwrapping."
// "When a type implements Deref, Rust can automatically treats a value of that type as if it were the value it contains."
// Deref makes the wrapper type behave like the value inside it, letting you use methods of the inner type without manual unwrapping or * everywhere
// Deref makes wrapper types transparent - use them like the thing inside
impl Deref for ItemRef {
    // This is the associated type - what we deref to
    // "When you deref ItemRef, you get an Item."
    type Target = Item; // <- What you get when you dereference 

    // Returns a reference to item
    fn deref(&self) -> &Item { // <- How to dereference
        // Access the inner Box<Item> and return a reference
        // Access the first (and only) field of the tuple struct
        &self.0
    }
    // You can now use it like: 
    // item_ref.name instead of item_ref.0.name
    // Rust automatically calls Deref for you
    // Deref helps with omitting wrapped syntax, smart pointers feel natural, method calls working deref coercion
}

impl Default for Item {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            quantity: 0,
            price: 0.0,
        }
    }
}

impl Default for Category {
    fn default() -> Self {
        Category::Other
    }
}

fn main() {
    // The newtype pattern (wrapping an existing type in a tuple struct) is useful in several scenarios:
    // 1. Type safety and semantic meaning - use newtypes when you want to distinguish between values that have the same underlying type but different meanings
    struct UserId(u64);
    struct OrderId(u64);
    // This prevents accidentally mixing them up
    fn get_user(id: UserId) { }
    fn get_order(id: OrderId) { }
    // Without newtypes, both would be just u64 and you could accidentally pass an order ID where a user ID was expected

    // 2. Enforcing invariants - when you need to maintain certain constraints or validation (guaranteeing values meeting certain constraints)
    // such as validated email addresses, non-empty strings, positive numbers
    struct EmailAddress(String);

    impl EmailAddress {
        fn new(email: String) -> Result<Self, &'static str> {
            if email.contains('@') {
                Ok(EmailAddress(email))
            } else {
                Err("Invalid email")
            }
        }
    }
    // This ensures EmailAddress is always valid because you control it's construction

    // 3. Implementing traits on external types - Rust's orphan rule prevents you from implementing external traits on external types, but you can wrap
    // a type to work around this
    struct MyVec(Vec<i32>);

    impl std::fmt::Display for MyVec {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "MyVec: {:?}", self.0)
        }
    }
    // The orphan rule is a Rust restriction that says: You can only implement a trait for a type if either the trait or the type is defined in your crate
    // So, for example, you can't implement Display on Vec since neither Display nor Vec are in our crate
    // The orphan rule exists to prevent coherence problems

    // 4. API clarity and intent - Even without strict validation, newtypes make function signatures more self-documenting
    // This is clearer
        // fn calculate_distance(meters: Meters) -> Kilometers { }
    // Than
        // fn _calculate_distance(meters: f64) -> f64 { }

    // Newtypes have no runtime overhead since they are optimized away by the compiler, making them a powerful performance design tool without the performance costs

    // However, for a newtype, we have to access the inner value with .0, which can feel cumbersome
    // We can make this more ergonomic by:
    // Implementing the Deref trait to automatically access the inner type 
    // Providing accessor methods - create methods on the newtype that expose the functionality you need
    // Implement relevant trait likes Display, From, or AsRef to make working with them more natural

    // deref is implemented for String
    // String -> str
    // Owned string -> string view
    let s = String::from("hello");
    // The deref trait for String returns a reference to the string slice inside
    // You can then call methods like s.len(), s.chars(), s.to_uppercase()
    // All str methods work on String
    // We do not have to explicitly deference something (*s).len()

    // deref is implemented for Vector
    // Owned array -> array view
    let v = vec![1, 2, 3, 4, 5];
    // It returns a slice viewing the Vec's contents
    // This allows slice methods to work on Vec
    // Vec<T> -> <T>

    // It also works for Box<T> -> T
    // Heap pointer -> heap value

    // -----
    // Box<T> is a smart pointer that allocates data on the heap instead of on the stack
    // Box moves data to the heap and gives you a pointer to it
    let x = 5; // i32 on the stack
    let boxed = Box:: new(5); // i32 on the heap, Box points to it
    // Use box for large data that would overflow the stack, recursive types, and trait objects
    
    // Your data is too big for your house (stack)
    // Put it in a storage unit (heap)
    // Keep the key/address (pointer) in your pocket
    // When you throw away the key, the storage unit is automatically emptied 

    // Stack:
    // Fast, automatic memory
    // Fixed size, known at compile time
    // LIFO - last in, first out
    // limited size
    // Automatically cleaned up
    
    // Heap:
    // Slower, manual memory
    // Dynamic size, determined at runtime
    // Can grow/shrink as needed
    // Much larger
    // You control allocation/deallocation

    // Stack holds values and pointers, Heap holds dynamic/large data
    // Box is on the stack and the data it points to is on the heap

}
