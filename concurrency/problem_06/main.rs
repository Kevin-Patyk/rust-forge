#![allow(dead_code)]
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// In this problem, we will learn about deadlocks - one of the most dangerous concurrency bugs
// A deadlock occurs when two or more threads are waiting for each other to release resources, creating a cycle where none can proceed

// Real world analogy:
// Two people approach a narrow doorway from opposite sides
// Person A steps through with the left foot, Person B steps through with right foot
// Both are stuck - each needs the other to back up first
// Nobody can move forward

// Classic deadlock scenario:
// Thread 1: Locks A -> (waits for B) -> Locks B
// Thread 2: Locks B -> (waits for A) -> Locks A
// Thread 1 holds A, waiting for B
// Thread 2 holds B, waiting for A
// Both hold forever

// In this problem, we will create a program that demonstrates a deadlock and then fix it
// Scenario: Bank transfer between 2 accounts
// Thread 1: Transfer from Account A to Account B
// Thread 2: Transfer from Account B to Account A

struct Account {
    id: u32,
    balance: f64,
}

// We only need to borrow Mutex, not the Arc
// Arc is for ownership, not for the function logic
// Arc dereferences automatically (Deref trait)
// Arc is about who owns the data, transfer() just needs to use it temporarily, it doesn't need to clone the Arc or extend its lifetime
// Also allows the function to take Arc<Mutex<Account>> and just Mutex<Account>
fn transfer(from: &Mutex<Account>, to: &Mutex<Account>, amount: f64) {

    // Acquire the lock here and unwrap
    let mut from_account = from.lock().unwrap();
    println!("Locked account {}", from_account.id);

    // Simulate some processing time (makes deadlock more likely)
    thread::sleep(Duration::from_millis(10));

    // Acquire the lock here and unwrap
    let mut to_account = to.lock().unwrap();
    println!("Locked account {}", to_account.id);

    // Perform the transfer
    from_account.balance -= amount;
    to_account.balance += amount;

    println!("Transferred {} from account {} to account {}", 
            amount, from_account.id, to_account.id);
}
// With the above implementation, the program will hang forever (deadlock)
// If it does not hang, it can complete due to a race condition

// There are several strategies to prevent deadlocks
// Strategy 1: lock ordering (always lock accounts in a consistent order, such as lower ID first)
// Strategy 2: try_lock() with timeout (try to acquire locks, back off if can't get both)
// Strategy 3: Single lock (use one mutex to protect both accounts)

// In this function, we will do lock ordering
fn transfer_2(from: &Mutex<Account>, from_id: u32, to: &Mutex<Account>, to_id: u32, amount: f64) {
    // Always lock the account with the lower ID first
    // This prevents a circular wait
    // First, peek at the IDs (without locking)
    // We need to determine which account has the lower ID
    // Problem: We can't access from.id without locking first!
    // Solution: Pass the IDs separately as parameters, OR lock both but in a consistent order
    // We're choosing to pass IDs as parameters to avoid locking just to check IDs
    // We are doing this to ensure that each thread locks in the same order (such as always A then B)
    // The deadlock happens because the relative order of locks is opposite between threads, even though each individual thread follows the same from -> to pattern
    
    // The code below ensures that all threads must acquire locks in the same global order, regardless of what they're doing with the data
    // We need the logic to enforce global order as the function can be called with from and to backwards
    // The logic is necessary because the function parameters (from and to) are different for each thread, but we need a consistent global locking order for threads
    let (first, second, is_forward) = if from_id < to_id {
        (from, to, true)
    }  else { 
        (to, from, false)
    };

    let mut first_guard = first.lock().unwrap();
    let mut second_guard = second.lock().unwrap();
    
    // is_forward tells us if we're doing the original transfer direction (from -> to)
    // or the reversed direction (to -> from)
    // Since we might have swapped first/second for lock ordering, we need to know
    // which account is the source and which is the destination
    if is_forward {
        // from -> to
        first_guard.balance -= amount;
        second_guard.balance += amount;
    } else {
        // to -> from 
        second_guard.balance -= amount;
        first_guard.balance += amount;
    }
}

// You can also prevent deadlocks using .try_lock()
// .try_lock() returns a result:
// Ok(guard) if the lock acquired successfully
// Err(_) if the lock is currently held by another thread
// If the lock for both threads in acquired by the same thread, we do the work
// If one thread has Lock A and one Lock B, both locks are released (using drop()) and the threads retry to get both locks
// until one thread acquires both locks and can execute its work
// Essentially, try_lock() prevents deadlock by allowing threads to give up and retry rather than blocking forever

// Pros: 
// No deadlock possible
// No need to know lock order in advance
// Flexible

// Cons:
// More CPU usage (retry loop)
// Non-deterministic try every time
// Potential for "livelock" -> both keep backing off
// More complex code

// Lock ordering is generally preferred because it is more efficient, deterministic, and simpler reasoning about code

fn main() {
    // Create 2 accounts that allow for multiple ownership and mutability across threads
    // Account A and Account B have their own separate locks
    let account_a = Arc::new(Mutex::new(Account { id: 1, balance: 1000.0 })); // Lock A
    let account_b = Arc::new(Mutex::new(Account { id: 2, balance: 1000.0 })); // Lock B
    // Each Mutex allows only one thread to at a time to access the DATA IT PROTECTS
    // But we have 2 separate Mutexes, so:
    // Lock A allows one thread at a time to access Account A
    // Lock B allows one thread at a time to access Account B
    // Different locks = different threads can hold them simultaneously
    // But if Thread 1 gets Lock A and Thread 2 gets Lock B, then they are in a deadlock since neither thread can move forward
    // The deadlock occurs when threads need multiple locks and acquire them in a different order

    // Key insight: Multiple locks can be held by different threads simultaneously
    // Thread 1 can hold Lock A
    // Thread 2 can hold Lock B
    // Both at the same time!
    // But if Thread 1 holds Lock A and needs Lock B,
    // while Thread 2 holds Lock B and needs Lock A,
    // they create a circular wait = deadlock!

    // With lock ordering, if both threads acquire Lock A before Lock B:
    // Thread 1: Gets Lock A -> Gets Lock B -> Does work -> Releases both
    // Thread 2: Tries Lock A (blocked, Thread 1 has it) -> Waits ... -> Gets Lock A -> Gets Lock B -> Does work
    // Thread 2 can't even start trying for Lock B until it has Lock A first
    // Thread 2 can't proceed to Lock B until it has Lock A - this prevents the circular wait

    // The key insight is that lock ordering forces threads to queue up at the first lock, preventing them from holding locks in a way that creates a cycle

    // Without lock ordering, it's a race
    // You can get lucky and either Thread 1 or Thread 2 can get both locks, so there is no deadlock
    // But, without lock ordering, whether you deadlock depends on thread scheduling (controlled by the OS)
    // Lock ordering eliminates the randomness

    // Clone both accounts
    // This increases the reference count
    // Makes a new pointer to the same data
    let acc_a_clone = Arc::clone(&account_a);
    let acc_b_clone = Arc::clone(&account_b);

    // Thread 1: A -> B
    let handle1 = thread::spawn(move || {
        transfer(&acc_a_clone, &acc_b_clone, 100.0)
    });

    // Clone both accounts again
    // This increases the reference count
    // Makes a new pointer to the same data
    let acc_a_clone2 = Arc::clone(&account_a);
    let acc_b_clone2 = Arc::clone(&account_b);

    // Thread 2: B -> A (Reverse Order - Deadlock)
    let handle2 = thread::spawn(move || {
        transfer(&acc_b_clone2, &acc_a_clone2, 50.0)
    });

    // The deadlock occurs when we try the transfers in the handles
    // Both account_a and account_b are locked at the exact same time 
    // So, each thread holds a resource the other needs
    // Even if, in the code we are locking them linearly, order differes between threads
    // When we use sleep, both threads can grab their first lock before trying for the second
    // The solution is that both threads must lock in the same order (such as always A before B, regardless of transfer direction)
    // If Thread 1 acquires Lock A and Thread 2 acquires Lock B, none can move forward

    // Allowing both threads to finish before proceeding
    // Why .join()?
    // Spawned threads branch off from the main thread
    // They then rejoin the main flow (join)
    // "Wait for this thread to finish and merge back"
    handle1.join().unwrap();  
    handle2.join().unwrap();

    println!("Final balances:");
    // We have to acquire the lock before printing since the Mutex always guards the data
    // Even if no spawned threads are running
    println!("Account A: {}", account_a.lock().unwrap().balance);
    println!("Account B: {}", account_b.lock().unwrap().balance);
}

// Summary:
// - Without lock ordering: Deadlock possible (depends on thread timing)
// - With lock ordering: No deadlock (threads acquire locks in same sequence)
// - Lock ordering breaks the circular wait condition
// - Thread 2 must wait for Lock A before trying Lock B
// - This prevents Thread 2 from holding Lock B while Thread 1 holds Lock A