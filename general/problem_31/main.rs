// Rust stack versus heap

// Stack allocation -----

// Stack memory: Fast, fixed size, automatically managed
// Data stored directly on the stack, cleaned up when out of scope

fn stack_examples() {

    // Primitives live on the stack
    let x: i32 = 42;           // 4 bytes on stack
    let y: f64 = 3.14;         // 8 bytes on stack
    let flag: bool = true;     // 1 byte on stack
    let ch: char = 'A';        // 4 bytes on stack

    // Fixed-size arrays live on the stack
    let arr: [i32; 5] = [1, 2, 3, 4, 5];  // 5 * 4 = 20 bytes on stack

    // Tuples live on the stack 
    let tuple: (i32, f64, bool) = (42, 3.14, true);  // 4 + 8 + 1 = 16 bytes (with padding)

    // Structs with only primitive types live entirely on the stack
    struct Point {
        x: i32,
        y: i32,
    }
    let point = Point { x: 10, y: 20 };  // 8 bytes on stack

    // Stack values are COPIED when assigned
    let a = 5;
    let b = a;  // COPIES the value 5
    println!("  a = {}, b = {}", a, b);  // Both valid! a wasn't moved

} // All stack variables are cleaned up here instantly

// Heap allocation -----

// Box<T> is for explicit heap allocation
// Stack holds: pointer (8 bytes on 64 bit)
// Heap holds: the actual data
// The heap is for data with dynamic or unknown size at compile time
// Allocation is slower and requires explicit management (Rust automates this through ownership)

fn heap_examples() {
    
    // Box: explicit heap allocation for single values
    // Stack: pointer
    // Heap: actual data
    let boxed_int = Box::new(42);           // Stack: 8 bytes | Heap: 4 bytes
    let boxed_float = Box::new(3.14);       // Stack: 8 bytes | Heap: 8 bytes
    let boxed_bool = Box::new(true);        // Stack: 8 bytes | Heap: 1 byte
    let boxed_char = Box::new('A');         // Stack: 8 bytes | Heap: 4 bytes

    // String: growable text
    // Stack: pointer + length + capacity
    // Heap: actual character data
    let s1 = String::from("hello");         // Stack: 24 bytes | Heap: 5 bytes
    let s2 = String::from("world!");        // Stack: 24 bytes | Heap: 6 bytes

    // Vec: Growable array
    // Stack: pointer + length + capacity
    // Heap: actual elements
    let vec_ints = vec![1, 2, 3, 4, 5];     // Stack: 24 bytes | Heap: 20 bytes (5 * 4)
    let vec_floats = vec![1.1, 2.2, 3.3];   // Stack: 24 bytes | Heap: 24 bytes (3 * 8)

    // Struct with heap-allocated fields
    struct Person {
        name: String,      // 24 bytes on stack, data on heap
        age: i32,          // 4 bytes on stack
    }
    let person = Person {
        name: String::from("Alice"),  // Stack: 24 bytes | Heap: 5 bytes
        age: 30,                       // Stack: 4 bytes
    }

    // Heap values are MOVED when assigned (not copied)
    let heap_str1 = String::from("move me");
    let heap_str2 = heap_str1;  // MOVES ownership, heap_str1 is now invalid
    // println!("{}", heap_str1);  // ERROR! heap_str1 was moved
    println!("{}", heap_str2);     // Only heap_str2 is valid

    // To copy heap data, use clone() for explicit deep copy
    let original = String::from("clone me");
    let copy = original.clone();  // Allocates NEW heap memory and copies data
    println!("{} and {}", original, copy);  // Both valid!

} // All heap allocations are freed here via Drop trait
// Deallocation happens automatically when stack pointers go out of scope

// References -----

// References are always pointers on the stack (8 bytes on 64-bit systems)
// References point to data that lives elsewhere (stack or heap)
// References don't own the data, so they don't trigger Drop when they go out of scope
// Borrowing rules prevent data races (either many readers OR one writer, never both)
// For heap data, it's two-level indirection: reference -> stack metadata -> heap data
// References let you access data without taking ownership, so the original owner stays valid

fn reference_examples () {

    // References to stack data ----- 

    let x = 42;        // Stack: 4 bytes (the value)
    let ref_x = &x;    // Stack: 8 bytes (pointer to x on stack)

    // Memory layout:
    // Stack: [x: 42] [ref_x: pointer to x]
    //         ↑_______________/

    // References to heap data -----

    let s = String::from("hello");  // Stack: 24 bytes | Heap: 5 bytes
    let ref_s = &s;                 // Stack: 8 bytes (pointer to s's stack location)

    // Memory layout:
    // Stack: [s: ptr|len|cap] [ref_s: pointer to s]
    //          ↓               ↑
    //          |_______________/
    //          ↓
    // Heap:  ['h']['e']['l']['l']['o']

    // ref_s points to s on the stack, which points to heap data
    // It's a pointer to a pointer
}

fn main() {
}
