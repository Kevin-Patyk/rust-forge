// Problem 45: Lock-Free Data Structures (Compare-and-Swap)

// What are lock-free data structures?
// So far we have used Mutex to protect shared data. But locks have overhead:
// - Threads block waiting for locks
// - Context switches when threads sleep
// - Risk of deadlock
// - Priority inversion issues

// Lock-free data structures use atomic operations instead of locks
// CAS is the foundation of lock-free programming

// Atomic operations = actions that are performed as a single, indivisible step (completely entirely or not at all) with no intermediate state visible to other processes or threads

// CAS in simple terms 
    // atomic_value.compare_exchange(
    //     expected_value,  // "I think the value is X"
    //     new_value       // "If it's X, change it to Y"
    // )

// Returns:
// - Ok(old)
// - Err(actual)

// Think of it like a vending machine:
// 1. You see the price is 1.50
// 2. You insert 1.50
// 3. The machine checks: "Is the price still 1.50?"
    // Yes -> Dispense
    // No (price changed) -> Reject, return money

// Why lock-free matters:
// - Work stealing: Uses lock-free queues in production (Chase-Lev deque)
// - Thread pools: Lock-free task queues reduce contention
// - High-performance systems: No blocking, predictable latency
// - Foundation of Rust atomics: Arc uses atomic reference counting

// The problem is to build a lock-free stack using atomic operations
// 1. Push without locks (use CAS to update the top pointer)
// 2. Pop without locks (use CAS to remove from top)
// 3. Handle concurrent pushes and pop directly
// 4. Track CAS retries to see contention

// Traditional stack with locks:
    // Stack: [3] -> [2] -> [1] -> null
    //        ^
    //        top (protected by Mutex)

    // Push(4): Lock -> modify top -> unlock

// Lock-free stack:
    // Stack: [3] -> [2] -> [1] -> null
    //        ^
    //        top (AtomicPtr)

    // Push(4):
    // 1. Read current top (3)
    // 2. Create new node [4] -> [3]
    // 3. CAS: "If top is still 3, change it to 4"
    //    - Success: Done!
    //    - Failure: Someone else modified top, retry from step 1

// Scenario:
// Spawn 4 threads
// Each thread pushes 1000 values to the shared stack
// Each thread pops 1000 values from the shared stack
// Track CAS retries for each thread (show contention)
// Verify all values pushes were eventually popped

// Atomic types in Rust:
// AtomicPtr<T>: Atomic pointer (what we will use for stack top)
// AtomicUsize: Atomic integer (useful for counting)
// Ordering: Memory ordering guarantees 

// Memory ordering:
// Acquire: Reads all previous writes
// Release: Writes are visible to future reads
// AcqRelease: Both acquire and release (safe default for CAS)

// The ABA problem:
// Thread 1 reads top = A
// Thread 2 pops A, pops B, pushes A again (top = A again)
// Thread 1's CAS succeeds but stack structure changed
// Solution: Use AtomicPtr carefully

use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::ptr;
use crate::thread::JoinHandle;

// This first struct is a linked list
// A linked list is a data structure where elements are stored in separate nodes and each one points to the next one

// A vector has all elements stored together in contiguous memory:
    // [1][2][3][4][5] <- One continuous block

// For a linked list, each element is in its own node, scattered in memory:
    // Node1        Node2        Node3        Node4        Node5
    // [1] →        [2] →        [3] →        [4] →        [5] → null
    // ^            ^            ^            ^            ^
    // Different    Different    Different    Different    Different
    // memory       memory       memory       memory       memory
    // location     location     location     location     location

// A node is a small container that holds:
// - Data (the actual value we are storing)
// - Pointer (the address of the next node)

// This will be one element in the linked list
struct Node<T> {
    value: T, // The data we are storing
    next: *mut Node<T>, // Pointer to the next node (null means "no next node" or end of the list)
    // It is an address in memory where a Node lives and we can use that address to access and modify the node
    // This is a regular raw pointer, NOT atomic

    // *mut is a raw mutable pointer to another Node<T>, not a reference or a smart pointer (like Box or Arc)
    // Just a memory address
    // A pointer that I can read from and write to, but Rust will not protect me
    // We need to use raw pointers since we need to do atomic operations on the pointer itself
    // You can't do atomic operations on Box or references, only raw pointers wrapped in AtomicPtr
}
// To traverse through a linked list, you start at the top and follow pointers
    // while current is not null {
    //     println!("{}", current.value);  // Print: 5
    //     current = current.next;         // Move to next node
    // }

// We used linked list so:
// - Each element is independent
// - Adding/removing from top is just pointer manipulation
// - No shifting, no reallocation
// - Perfect for lock-free because you only modify one pointer (top)

// Real world analogy: Linked list = treasure hunt
    // Start here → "Go to the big tree" → "Go to the red house" → "Go to the park" → End
    //              (pointer)                (pointer)              (pointer)

// As a refresher, a pointer is just a memory address - it tells you where something is stored in memory

// Memory:
// Address 0x1000: [value: 5]
// Address 0x2000: [value: 10]

// A pointer to 0x2000 -> "The value at address 0x2000 is 10"

// -----

// This is the stack itself
// It only needs an atomic pointer to the top node
struct LockFreeStack<T> {
    // An atomic pointer is a pointer that can be safely modified from multiple threads using atomic operations (like CAS)
    // It provides atomic operations - without atomic operations, actions can be interrupted by other threads, causing data races
    // Multiple threads can safely read/modify simultaneously
    top: AtomicPtr<Node<T>>, // Thread safe atomic pointer
    // The top pointer points to a node that contains data (where the first node is)

    // Multiple threads can simultaneously try to update top using CAS
    // If 2 threads collide, one succeeds and the other retries - no locks needed

    // We wrap top in AtomicPtr because:
    // 1. Multiple threads will modify top simultaneously
    // 2. Without atomics -> data races, lost updates, corruption
    // 3. With AtomicPtr -> CAS lets threads compete safely
    // 4. No locks needed -> threads never block, just try on collision

    // The AtomicPtr ensures that, when we update the top pointer, we can detect if another thread changed it and retry if needed
    // This is the foundation of lock-free programming

    // The top node will always be wrapped in an atomic pointer, the rest won't 

    retry_count: AtomicUsize,
}

// When a struct is generic, its impl must also be generic over the same type parameters
// If a type has <T>, the impl needs <T> too
// T is shorthand for any type
// T is a type parameter (it stands for some concrete type chosen by the user)
// The code is monomorphized (compiled separately) for each concrete type
// We can restrict T with trait bounds, but we are not doing that here
// We will specify T when we instantiate the struct
impl<T> LockFreeStack<T> {
    // Create a new empty stack
    fn new() -> Self {
        Self {
            // AtomicPtr initialized with null raw mutable pointer (empty stack)
            // It is initialized to not point to any node yet
            // null pointer = points to nothing
            // empty stack = top points to nothing
            // As we put more nodes on top, the first node we made (last one in the list) will eventually point to this, meaning the stack is empty
            top: AtomicPtr::new(ptr::null_mut()), // ptr::null_mut() creates a null mutable pointer
            // We are initializing an AtomicPtr with a null pointer, which is a common way to represent "no value yet"
            retry_count: AtomicUsize::new(0),
        }
    }

    // Check if the stack is empty
    // If it points to an existing node in memory, stack is not empty
    fn is_empty(&self) -> bool {
        // Load the current top pointer
        // If it's null, then the stack is empty
        self.top.load(Ordering::Acquire).is_null()
        // .load() atomically reads the pointer value -> loads a raw pointer (*mut Node<T>) from the atomic variable
        // Acquire = See all previous writes
        // Returns the current pointer (could be null or point to a node)

        // .is_null() checks if the pointer is null 
        // true = empty stack (no valid memory address stored in top/no node in top)
        // false = has nodes (pointer points to some memory address)
    }

    // Here, we are pushing data onto the LockFreeStack
    // This function will take a generic value T and wrap it in a node
    fn push(&self, value: T) {

        // Step 1: Allocate a new node on the heap -----

        // Box::new() allocates a Node on the heap, returns Box<Node<T>>
        // Box::into_raw() converts Box<Node<T>> to *mut Node<T> raw pointer, returns *mut Node<T>
        // We need into_raw() since AtomicPtr only works with raw pointers, not Box<T>
        // It gives us ownership of the heap memory but as a raw pointer
        // We need this to be raw pointer since all nodes will be raw pointers
        let new_node = Box::into_raw(Box::new(Node {
            value,
            // We will update this in the loop before CAS
            next: ptr::null_mut(),
        }));
        // At this point, we have a new node (raw pointer) that points to Node { value: T, next: null }
        // It exists in memory, but not in the stack yet

        // Step 2: Loop until we successfully update top -----

        // We are using loop since multiple threads might be pushing simultaneously
        // CAS might fail if another thread modified top first
        // We keep trying until we succeed
        // Lock free - no blocking, just retry
        loop {

            // Step 3: Read the current top pointer -----

            // Atomically read the current value of top
            // Returns a *mut Node<T> (could be null if empty or an address)
            // Acquire = see all previous writes from other threads
            let current_top = self.top.load(Ordering::Acquire);

            // Step 4: Set our new node's next field to point to the current top -----

            // We need unsafe since dereferencing raw pointers is always unsafe
            // We must promise that the pointer is valid
            unsafe {
                // (*new_node) is dereferencing the raw pointer to access the node
                // new_node is *mut Node<T>
                // (*new_node) is Node<T>
                // .next is accessing the field of the Node
                (*new_node).next = current_top;
            }

            // Step 5: Try to make top point to our new node using CAS

            // .compare_exchange() is an atomic operation used to safely update a value only if it currently has the expected value
            // If the current value == expected, update succeeds
            // If the current value != expected, update fails and nothing changes
            // So, if, for this iteration, the current top is the expected current top, we place our new node
            // But, if the current top is not the expected current top (it has changed), we:
            // - Go to the top of the loop
            // - Update the top in our node to the newest current top
            // - Try this loop again
            match self.top.compare_exchange(
                current_top, // Expect value
                new_node, // New value
                Ordering::Release, // Success ordering
                Ordering::Acquire, // Failure ordering
            ) {
                Ok(_) => {
                    // Success - our node is now the top of the stack
                    return;
                }
                Err(_) => {
                    self.retry_count.fetch_add(1, Ordering::Relaxed);
                    // Failed - someone else chaned top, retry
                    continue;
                }
            }
        }
    }
    // 1. We are creating a Node struct with:
        // value: T (the actual data)
        // next: *mut Node<T> (a null pointer since it points to nothing initially)
    // The node is allocated on the heap and then converted to a raw pointer (Box::into_raw) 
    // new_node itself is a *mut Node<T> (raw pointer) that points to the node we just created
        //     **Memory:**
        // ```
        // Heap address 0x1000:
        // ┌─────────────────┐
        // │ Node            │
        // │  value: T       │ ← The data
        // │  next: null     │ ← Empty/null pointer (points to nothing)
        // └─────────────────┘
        //        ↑
        //        │
        // new_node (0x1000) ← Raw pointer to this Node
    // new_node = raw pointer to the Node (type: *mut Node<T>)
    // The Node = struct with value: T and next: null
    // next field = null pointer initially
    // We have to make the node a raw pointer since it will be added to the top, which requires an atomic pointer
    // and atomic pointers only work with raw pointers

    // 2. We are using a loop since:
        // Multiple threads might be pushing simultaneously
        // CAS might fail if another thread modified top first
        // We keep trying until we succeed
        // Lock free - no blocking, just retry
    
    // 3. Acquire the current value of top using .load()
    // This will return a raw pointer (*mut Node<T>), which can be null if empty or an address
    // If the stack is empty:
        // top: AtomicPtr(null)
        // current_top = null (0x0)
    // If there are nodes in the stack:
        // top: AtomicPtr(0x2000) → Node { value: 3, next: 0x3000 }
        // current_top = 0x2000
    // Stack: top = [3] → [1] → null
    // current_top = address of Node [3] (e.g., 0x2000)
    
    // 4. Set our new nodes next field to point to the current top
    // We need to use the unsafe flag since we are using as raw pointer, so there are no safety guarantees
    // We dereference our current (new) node and access the next field and set it to the current top
    //  Before:
        // current_top = [3] → [1] → null
        // new_node = [5] → null
    // After setting next:
        // current_top = [3] → [1] → null
        // new_node = [5] → [3] (points to the same Node as current_top)
    // At this point, we have a raw pointer that points to a node with data and now a current top (which is either something or null)

    // 5. Now, we will try to put our new node on top using CAS
    // We are using .compare_and_exchange() for this.
    // .compare_and_exchange() is an atomic operation used to safely update a value only if it currently has an expected value
    // "Set this value to new only if it is currently equal to expected."
        // If the current value == expected, update succeeds
        // If the current value != expected, update fails and nothing changes
    // In our case, if we match on Ok(), meaning the expected value is our current top (nobody changed it), we update the top to our node
    // If we match on Err(), meaning the expect value is not our current top (someone changed it), we go to the top of the loop, 
        // assign the new current top to our node, and try again
    // Before CAS:
        // top = [3] → [1] → null
        // new_node = [5] → [3]
    // After CAS (Success):
        // top = [5] → [3] → [1] → null
    // 6. The complete flow visualization is:
        // a. Create new node: new_node = [7] → null
        // b. Load top: current_top = [5] → [3] → [1] → null
        // c. Link new_node: new_node = [7] → [5] (points to current top)
        // d. CAS attempt: If top is still [5], then [7] becomes our new top, if not, loop and retry with new top value

    // This function is for taking data out of the stack
    fn pop(&self) -> Option<T> {

        loop {
            // Step 1: Load the current top -----

            // As before, we are loading the current top using .load()
            // The current top can be a raw pointer (memory address) to another node or null
            // We are using Ordering::Acquire to see all previous writes 
            let current_top = self.top.load(Ordering::Acquire);


            // Step 2: Check if the stack is empty -----

            // Now, we need to check if the stack is empty
            // If the stack is empty, meaning .load() points to null (null (0x0)), we return nothing
            if current_top.is_null() {
                return None;
            }

            // Step 3: Read the next pointer from the current top node

            // We do this before CAS
            // We need to use unsafe since we are dealing with raw pointers
            // (*current_top) dereferences the raw pointer
            // We dereference the raw pointer to access the node's (current top) fields
            // next gets the raw pointer to the next node (or null if this was the last node)
            // We need this so we know what to point to after removing the top node
            let next = unsafe {
                (*current_top).next
            };

            // Step 4: Try to update top to point to the next (removing current_top) -----

            // We are using .compare_exchange()
            // If the top is current_top (the value we expect), we make the swap
            // If the top is not current_top, we try again (start the loop over)
            match self.top.compare_exchange(
                current_top, // Expected: top should still be current_top
                next, // New: make top point to next
                Ordering::Release, // Success ordering
                Ordering::Acquire, // Failure ordering
            ) {
                Ok(_) => {
                    // Success - we removed current top from the stack
                    // Now we need to extract the value and free the memory
                    
                    // Step 5: Convert raw pointer back to Box -----
                    
                    // This will take ownership and will drop (free) the node when it goes out of scope
                    // This converts from *mut Node<T> to Box<Node<T>>
                    // Takes ownership of the memory
                    // Will automatically free the memory when the Box is dropped
                    let node = unsafe {
                        Box::from_raw(current_top)
                    };

                    // Step 6: Extract and return the value ------
                    // The Box is dropped here, freeing the memory
                    // node goes out of scope and is dropped
                    // The box automatically frees the heap memory
                    return Some(node.value);
                }
                Err(_) => {
                    self.retry_count.fetch_add(1, Ordering::Relaxed);
                    // Failed - another thread modified the top
                    // Loop back and retry
                    continue;
                }
            }

        }

    }
    // Step 1: Read the current top
    // We read the current top pointer, whiich returns *mut Node<T> (could be null or an address)
        // Stack: top = [5] → [3] → [1] → null
        // current_top = address of Node [5] (e.g., 0x1000)
    // Step 2: Check if empty
    // If the stack is empty, top is null
    // We can't pop from an empty stack
    // Return None to indicate failure
        // Empty stack:
        // top = null

        // current_top.is_null() = true
        // Return None
    // Step 3: Read the next pointer
    // We dereference the raw pointer to access the node
    // current_top = *mut Node<T>
    // (*current_top) = Node<T>
    // We access the next field of the node
    // This is a raw pointer to a Node (*mut Node<T>), pointer to the next node or null
        // Stack: top = [5] → [3] → [1] → null
        //           ^     ^
        //           |     |
        //     current_top next (points to [3])
        // next = memory address of Node [3]
    // We do this before CAS since we need to know where to point top after removing the current node
        // Before pop:
        // top = [5] → [3] → [1] → null
        // After pop (we want):
        // top = [3] → [1] → null
        // ^
        // This is what we read as 'next'
    // Step 4: Try to update top to point to next (removing current_top)
    // We use .compare_exchange() to atomically update top
    // Atomically: "If top is STILL current_top, change it to next"
    // Success (Ok): current_top was removed from stack
    //   - We now own the removed node
    //   - Must extract value and free memory using Box::from_raw
    //   - Box automatically frees memory when dropped
    // Failure (Err): Another thread changed top
    //   - Loop back and retry with new top value

    // Memory management cycle:
    // push(): Box::new() → Box::into_raw() (Box → raw pointer, manual management)
    // pop():  Box::from_raw() → Box dropped (raw pointer → Box, automatic free)

    // When we push(), the new top node has to point to the previous top node
        // Before:
        // top = [3] → [1] → null

        // After push(5):
        // top = [5] → [3] → [1] → null
        //   ^^^^   ^^^^
        //   new    previous (new POINTS TO previous)

    // When we pop(), the new top node was pointed to by the previous top node
        // Before:
        // top = [5] → [3] → [1] → null
        //     ^^^^   ^^^^
        //     previous new (previous POINTED TO new)

        // After pop():
        // top = [3] → [1] → null

    fn retry_count(&self) -> usize {
        self.retry_count.load(Ordering::Relaxed)
}
}

fn main() {
    
    // Create a stack wrapped in Arc so multiple threads can share it
    let stack = Arc::new(LockFreeStack::<i32>::new());

    let num_threads = 4;
    let operations_per_thread = 1000;

    println!("Spawning {} threads...", num_threads);
    println!("Each thread will push {} values then pop {} values\n", 
             operations_per_thread, operations_per_thread);

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for thread_id in 0..num_threads {

        // Making a clone of the stack
        // This is because each thread will need its own stack
        // We will be moving the stack into the thread so the thread can own it and use it after the loop iteration ends
        let stack_clone = Arc::clone(&stack);

        let handle = thread::spawn(move || { // Thread starts running immediately
            // Spawning a thread is near-instant, the work happens in the background

            // push phase
            // Each thread will push 1000 values onto the stack
            for i in 0..operations_per_thread {
                let value = thread_id * 10000 + i;
                stack_clone.push(value);
            }

            // pop phase
            // Each thread will pop 1000 values off of the stack
            let mut popped_count = 0;
            for _ in 0..operations_per_thread {

                // Since .pop() returns an Option, we need to check if it is Some() to increase the pop count
                // If it is the Some() variant, that means something was taken from the stack and it was not null
                if stack_clone.pop().is_some() {
                    popped_count += 1;
                }
            }

                        
            println!("Thread {} completed: {} pushes, {} pops", 
                     thread_id, operations_per_thread, popped_count);
        });

        // This handle push to the handles vector happens immediately, not when the thread finishes
        // Think of like:
            // Starting a task and getting a receipt
            // The task is happening now
            // The receipt lets you check on it later
        handles.push(handle); 
    }
    // After each iteration of the thread spawning loop, the thread starts doing its work in the background

    // Wait for all threads to finish - joining loop
    for handle in handles {
        handle.join().unwrap();
    }

    println!("\n=== Results ===");
    println!("Total operations: {}", num_threads * operations_per_thread * 2);
    println!("Total CAS retries: {}", stack.retry_count());
    println!("Stack is empty: {}", stack.is_empty());
    
    // Verify stack is empty
    if stack.is_empty() {
        println!("✓ All values pushed were successfully popped!");
    } else {
        println!("✗ Stack still has elements (bug!)");
    }
}
