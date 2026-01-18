// In this problem, we are going to implement a thread pool builder
// A thread pool builder is a helper struct that makes it easier and more flexible to create a thread pool with custom options
// Rayon's thread pool builder let's you configure:
    // number of threads
    // stack size per thread
    // thread name prefix
    // panic handler
    // start/exit hooks for threads
// We are building a configuration object that collects our preferences, then constructs the thread pool with all those settings


struct ThreadPoolBuilder {
    // We are using Option for the builder fields to disinguish between "not set" and "set to something specific"
    // Which allows us to provide sensible defaults if something is not
    // With Option, each field can be in three conceptual states:
        // 1. None - User didn't configure this, use default
        // 2. Some(value) - User explicitly set this value
        // 3. Use .unwrap_or(default) to handle both cases

    // The number of threads to spawn
    num_threads: Option<usize>,
    // The amount of memory allocated for each thread's call stack 
    // When a thread runs code, it needs memory to store:
        // Local variables
        // Function parameters
        // Return addresses
        // Each nested function call
    // This memory is organized as a stack (LIFO - last in, first out)
    stack_size: Option<usize>,

    // Refresher notes -----
    
    // Stack:
        // Fixed size per thread
        // Fast allocation (just move a pointer)
        // Automatic cleanup 
        // Limited size
    
    // Heap (Box, Vec, etc):
        // Unlimited size (until you run out of RAM)
        // Slower allocation
        // Manual management (via ownership)
        // Use when data too big for stack
}

// Below, we are implementing the builder pattern
// The builder pattern is a way to construct complex objects step-by-step, where you can:
    // Set only the fields you care about
    // Get sensible defaults for everything else
    // Chain methods together in a fluent, readable way

// There are 2 types of builders in Rust: immutable and mutable

// Immutable builder (creating new instances):
    // Takes ownership using self 
    // Each method returns a brand new instance using Self {}
    // Old fields are copied to the brand new instance

// Mutable builder (modifying same instance - most common)
    // Modify and return the same instance
    // Take mut self and return self

// Key rules for builder methods:
    // Take self - ownership is key
    // Mark parameters as mut self if modifying
    // Return Self for chainable methods
    // Return the final product type for .build()

impl ThreadPoolBuilder {
    // new() is an associated function
    // It does not take the self parameter
    // It does not need an instance of the struct to work
    // Rather, it creates instances of the struct
    // It's job is to create the first instance
    fn new() -> Self {
        Self {
            num_threads: None,
            stack_size: None,
        }
    }

    // This is a builder method
    // We are taking ownership of self and allowing mutation
    // Self = refers to the TYPE itself (like ThreadPoolBuilder)
    //   - Used in type annotations (return types, field types)
    //   - Capitalized because it's a type name
    // self = refers to an INSTANCE/VALUE of the type  
    //   - Used as a parameter and can be returned as a value
    //   - Lowercase because it's a variable/value
    // We take an instance of the struct, take ownership, modify it, and then return it
    // We are not creating new instances each time - it is the same exact instance being modified and passed along
    fn num_threads(mut self, n: usize) -> Self { // Self = ThreadPoolBuilder
        // We are using Some(n) since the field is an option
        self.num_threads = Some(n);
        self
    }

    // This is our second build method which takes an instance of the struct, takes ownership, modifies it, and then returns it
    fn stack_size(mut self, size: usize) -> Self { // Self = ThreadPoolBuilder
        self.stack_size = Some(size);
        self
    }

    // The final .build() method will return an instance of ThreadPool with the values provided
    // In this example, we do not actually have a ThreadPool struct since this is just a demonstration
    // We consume self here so that the builder is destroyed and all of its data is moved into ThreadPool
    // Thus, it can no longer be used anymore
    fn build(self) -> ThreadPool {
        // build() consumes self (takes ownership without returning it)
        // This is by design:
            //   1. Prevents calling .build() twice
            //   2. Makes it clear the builder is done - transition to product
            //   3. Builder is useless after building anyway
            //   4. Moves all configuration data into the final ThreadPool

        // Use .unwrap_or() to provide defaults if options are None
        // .unwrap_or() is a method on Option<T> that extracts the value if it exists or pvodies a default if it doesn't
            // If Some(value) -> return value
            // If None -> return the default you provided
        // We are doing this because both num_threads and stack_size are Options
        // If the user uses the methods to set them, it will use the values provided by the user
        // If the user does not use the methods to set them, it will provide sensible defaults
        let num_threads = self.num_threads.unwrap_or(4);
        let stack_size = self.stack_size.unwrap_or(2 * 1024 * 1024); // 2MB default

        // As a note:
        // .unwrap_or(value) - default is evaluated immediately (use for cheap defaults like numbers)
        // .unwrap_or_else(|| closure) - default computed lazily (use for expensive defaults)

        // Now you can call ThreadPool::new() or build it directly
        // You could either:
            // 1. Have ThreadPool::new() accept these parameters
            // 2. Or build the ThreadPool directly with these values
        
        ThreadPool::new(num_threads, stack_size)
    }
}

// How method chaining works:
// ThreadPoolBuilder::new()    // Creates instance A
//     .num_threads(4)          // Takes ownership of A, modifies, returns A
//     .stack_size(1024)        // Takes ownership of A, modifies, returns A  
//     .build();                // Takes ownership of A, consumes it, returns ThreadPool

fn main() {
    // Example 1: Set both options
    let pool = ThreadPoolBuilder::new()
        .num_threads(8)
        .stack_size(4 * 1024 * 1024)
        .build();
    
    // Example 2: Only set num_threads (stack_size gets default)
    let pool = ThreadPoolBuilder::new()
        .num_threads(8)
        .build();
    
    // Example 3: Use all defaults
    let pool = ThreadPoolBuilder::new().build();
}