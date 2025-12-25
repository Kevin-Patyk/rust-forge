#![allow(dead_code, unused_imports)]

use std::cmp::Ordering;
use std::borrow::Borrow;
use std::collections::HashMap;

// #[derive(PartialEq, Eq)]
// When we do the above, called an attribute, Rust automatically generates implementations of the PartialEq and Eq traits
// for our type
// derive = "Automatically generate the trait implementation based on the fields of the struct"
enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

struct Task {
    id: u32,
    name: String,
    priority: Priority,
    deadline: u32,
}

// Newtype struct 
// This is wrapping an existing struct in a struct
// This has no overhead because they are optimized away by the compiler
// Good for API clarity, going around Rust's orphan rule, making types that meet certain criteria, and distinguishing between types with same underlying values
struct TaskRef(String);

// PartialEq allows comparing for equality with == and !=
// Eq is a marker trait (it has no methods) -> indicates that equality is reflexive for all values
// Types that implement Eq must already implement PartialEq
impl PartialEq for Priority {
    // &self is Priority
    // &Self is another Priority to compare
    // They both have to be of type Priority
    fn eq(&self, other: &Self) -> bool {
        // Two priorities are equal if they are the same variant
        match (self, other) {
            // Here, we are matching on a tuple of two references
            // Both must be the same for it to be true
            // We use a tuple to make matching both values clean and simple
            // Tuple matching lets you pattern match on multiple values at once
            (Priority::Low, Priority::Low) => true,
            (Priority::Medium, Priority::Medium) => true,
            (Priority::High, Priority::High) => true,
            (Priority::Critical, Priority::Critical) => true,
            // Any other combination is false (different variants)
            _ => false,
        }
    }
}

// Implement Eq (marker trait - no methods required)
// A marker trait is a trait that has no methods or associated items - it purely exists as 
// a compile-time label to indicate a property about a type
impl Eq for Priority {}

// PartialOrd allows comparing values with <, >, <=, >=
// Ord - total ordering (every comparison has a defined result)
// To implement Ord, you must also implement PartialOrd, Eq, PartialEq

impl PartialOrd for Priority {
    // &self is Priority
    // &Self is another Priority to compare
    // We return an Option because comparison might not be defined
    // Some types can't always be compared and will return None, such as 1.0 and f64::NAN
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other)) // Delegate to Ord::cmp
        // .cmp() needs Ord for this to be used
    }
}

impl Ord for Priority {
    // &self is Priority
    // &Self is another Priority to compare
    // The Ordering enum is defined as:
    // pub enum Ordering {
    //      Less, Equal, Greater
    // }

    // So, this method returns Ordering (Less, Equal, Greater)
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            // Same variants are equal
            (Priority::Low, Priority::Low) => Ordering::Equal,
            (Priority::Medium, Priority::Medium) => Ordering::Equal,
            (Priority::High, Priority::High) => Ordering::Equal,
            (Priority::Critical, Priority::Critical) => Ordering::Equal,
            
            // Low is less than everything else
            (Priority::Low, _) => Ordering::Less,
            (_, Priority::Low) => Ordering::Greater,
            
            // Medium comparisons
            (Priority::Medium, Priority::High) => Ordering::Less,
            (Priority::Medium, Priority::Critical) => Ordering::Less,
            (Priority::High, Priority::Medium) => Ordering::Greater,
            (Priority::Critical, Priority::Medium) => Ordering::Greater,
            
            // High comparisons
            (Priority::High, Priority::Critical) => Ordering::Less,
            (Priority::Critical, Priority::High) => Ordering::Greater,
        }
    }
}

// For simple enums, it is just best to derive them all:
// #[derive(PartialEq, Eq, PartialOrd, Ord)]

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

// Trait marker
impl Eq for Task {}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other)) // Delegate to Ord::cmp
    }
}

// This will be used by .sort()
impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare priority first (reversed for descending)
        match other.priority.cmp(&self.priority) {
            Ordering::Equal => {
                // If priorities are equal, compare deadlines (ascending)
                self.deadline.cmp(&other.deadline)
            }
            other_ordering => other_ordering,
        }
    }
}

// Borrow<T> is a trait that allows borrowing owned data in a borrowed form
// It is similar to AsRef, but with stricter requirements - it must preserve equality and ordering
// Borrow is meant for borrowing collections (like HashMaps) and must preserve Eq and Ord behavior
// AsRef is general reference conversions and just needs to return a reference to the TargetType
impl Borrow<str> for TaskRef {
    // <str> is the Target Type
    // TaskRef is the source type
    // More strict than AsRef and enables ergonomic collection APIs
    // Powers flexible HashMap/HashSet lookups
    fn borrow(&self) -> &str {
        &self.0 // self.0 is a String
        // &self.0 is a &String, which derefs to &str
    }
}

// HashMaps use hashing and equality to find keys 
// Using Borrow<str> guarantees:
// hash(string) == hash(string.borrow())
// string == string.borrow()
// string.cmp(x) == string.borrow().cmp(x)

struct TaskQueue {
    tasks: Vec<Task>,
}

impl TaskQueue {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
        }
    }

    fn add(&mut self, task: Task) {
        self.tasks.push(task);
        // Here, .sort() works by using our Ord implementation
        // Rust internally calls .cmp() on your Task type to compare elements
        self.tasks.sort();
    }

    fn next(&mut self) -> Option<Task> {
        // .pop() needs &mut self since we are removing something from the vector 
        // It modifies the vector (removes the element)
        // It modifies by removing the last element, decreasing the length, changing the internal state
        if self.tasks.is_empty() {
            None
        } else {
            // Since tasks are sorted (highest priority first)
            // pop removes and returns the last element (lowest priority)
            // so we need to remove the first element instead
            // .pop() removes the last item in a vector - it is O(1)
            // .remove() is O(n) - has to shift all elements since it removes the first one
            self.tasks.pop() // We do not need to wrap this in Some() since .pop() already returns Option<Task>
        }
    }

    fn peek(&self) -> Option<&Task> {
        // .last() returns Some(&Task) if the vec is not empty - reference to the last element
        // None if the vec is empty
        self.tasks.last()
    }

    // This is a generic function with a trait bound
    // It takes Q, which is something that implement Borrow<str>>, meaning it can be converted to a reference that preserves equality and ordering
    fn find_by_name<Q: Borrow<str>>(&self, name: Q) -> Option<&Task> {
        // Generic type Q that can be borrowed as &str
        
        // Convert Q to &str using Borrow trait
        let name_str: &str = name.borrow();

        // Iterate over references to tasks
        // Find the first where the name matches
        self.tasks.iter().find(|task| task.name == name_str)
    }
    }


fn main() {
    // Why you Borrow instead of just &str?

    // Without the trait bound, you can do:
        // queue.find_by_name("task");  // ✅ Works

        // let s = String::from("task");
        // queue.find_by_name(s);       // ❌ Error: expected &str, found String
        // queue.find_by_name(&s);      // ✅ Have to borrow manually
    
    // With the trait bound, you can accept many types:
        // queue.find_by_name("task");              // ✅ &str
        // queue.find_by_name(String::from("task")); // ✅ String
        // queue.find_by_name(&s);                  // ✅ &String
        // queue.find_by_name(task_ref);            // ✅ TaskRef

    // .find_by_name() will now accept &str, String, &String, TaskRef, etc.
    // Standard Rust pattern for flexible look ups
}
