// In this problem, we will learn about work stealing
// Work stealing is an advanced load balancing technique where idle workers can "steal" tasks from busy workers' queues

// What is work stealing?
// With a traditional thread pool (what we built in problem 38):
// - Single shared queue
// - All workers compete for the same lock
// - Lock contention when many workers

// Work stealing pool:
// - Each worker has its own local queue (deque)
// - Workers primarily work on their own queue (no contention)
// - Idle workers steal from the END of busy worker's queues
// - Much better performance for parallel workloads

// Traditional Pool:
// Main Thread -> [Shared Queue] <- Worker1, Worker2, Worker3 (all fighting for lock)

// Work Stealing Pool:
// Main Thread -> Distributes tasks
// Worker1: [Local Queue] <- steals from Worker2's end if empty
// Worker2: [Local Queue] <- steals from Worker3's end if empty  
// Worker3: [Local Queue] <- steals from Worker1's end if empty

// Why work stealing matters:
// - Less contention: Workers primarily use their own queue
// - Better cache locality: Workers keep working on related tasks
// - Load balancing: Busy workers get help from idle workers
// - Rayon uses this internally

// The Problem:
// Build a work stealing thread pool where:
// 1. Each worker has its own VecDeque<Task>
// 2. Workers pop from the BACK of their own queue (LIFO - Last In, First Out) - .pop_back()
// 3. Thieves steal from the FRONT of other workers' queues (oldest tasks) - .pop_front()
// 4. Track steal attempts (successful and failed)
// 5. Demonstrate load imbalance and how stealing fixes it

// Scenario:
// - Create 4 workers
// - Give Worker 0: 20 slow tasks (200ms each)
// - Give Worker 1,2,3: 5 fast tasks each (50ms each)
// - Without stealing: Worker 0 takes forever, others finish quickly and sit idle
// - With stealing: Workers 1,2,3 steal from Worker 0 -> balanced workload

// Key concepts:
// Local work (LIFO): [Task1, Task2, Task3, Task4]
//                     front ←            → back
//                     Worker pops from back → gets Task4 (most recent, cache-hot)
//
// Stealing: Thieves steal from front → gets Task1 (oldest, least likely cache-hot) (FIFO - First In, First Out) - .pop_front()
// This prevents conflict: worker works on recent tasks, thieves take old tasks

// Per-worker queues: Vec<Arc<Mutex<VecDeque<Task>>>>
// Steal strategy: Try to steal from (worker_id + 1) % num_workers, then try others
// Randomization: In production, randomly choose victim to steal from

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::thread;
use std::time::Duration;

#[allow(dead_code)]
enum Task {
    Compute { id: usize, workload: u64 }, // workload = milliseconds to sleep
    Shutdown, // Poison pill to stop workers
}

// Creating a Stats struct to store thread-local statistics
#[allow(dead_code)]
struct Stats {
    local_tasks_completed: usize,
    stolen_tasks_completed: usize,
    steal_attempts: usize,
    failed_steals: usize,
}

// In problem 40, we had a global TaskQueue
// tasks: Mutex<VecDeque<Task>>  ← ONE queue, ALL workers compete
// All workers fight for the SAME lock on the SAME queue
// One TaskQueue instance shared by all workers via Arc<TaskQueue>

// In this problem, we have per-worker queues (work stealing)
// Each worker has their OWN queue with their OWN lock
// Workers can steal from OTHER worker's queues when idle
// Multiple WorkerQueue instances, one per worker
struct WorkerQueue {
    tasks: Mutex<VecDeque<Task>>, // Each worker has one of these

    // If a field of a struct is wrapped in Mutex, you must .lock() to access it, even if the struct itself isn't wrapped in Mutex
}

impl WorkerQueue {
    fn new() -> Self {
        Self {
            tasks: Mutex::new(VecDeque::new()),
        }
    }
    fn push_local(&self, task: Task) {
        // Push to back
        // Owner thread of the queue takes from back (LIFO)
        // Threads who steal take from the front
        self.tasks.lock().unwrap().push_back(task);
    }

    fn pop_local(&self) -> Option<Task> {
        // Pop from the back 
        // Owner thread of the queues takes from back (LIFO)
        // Threads who steal take from the front
        self.tasks.lock().unwrap().pop_back()
    }

    fn steal(&self) -> Option<Task> {
        // Steal from the front 
        // When stealing from another thread, it takes from the other thread's front of the queue
        self.tasks.lock().unwrap().pop_front()

        // We will use it like: let stolen = worker_queues[0].steal();
        // Calling .steal() on Worker 0's queue
        // self = Worker 0's queue
        // But Worker 1 is doing the stealing

        // When we steal from Worker 0, we lock Worker 0's queue, pop from the front, and return the Task
        // The Task VALUE is now in Worker 1's local variable -> From Worker 0's VecDeque to Worker 1's local variable
        // Worker 0's queue no longer has this task
        // Worker 1 now owns this Task and executes this task in its own thread

        // 1. Lock the victim's queue
        // 2. Remove task from victim's VecDeque
        // 3. Unlock the victim's queue 
        // 4. Return the task value
        // 5. Thief thread now has the task and processes it
    }
}

fn main() {

    // Creating the worker queues
    // This gives us worker_queues[0], worker_queues[1], etc.
    // We are wrapping each in an Arc since they need to be able to be shared across threads (for stealing)
    // The tasks field for each WorkerQueue is already wrapped in a Mutex, so we will need locks to access them
    let worker_queues: Vec<Arc<WorkerQueue>> = (0..4) // We don't need .iter() on a range because ranges are already iterators - they implement the Iterator trait directly
        .map(|_| Arc::new(WorkerQueue::new()))
        .collect();
    // Here, we are creating a vector of 4 worker queues using the .map() method and collecting it into a vector
    // We are putting them all in a vector to make it easier to work with compared to having 4 separate variables
    // Benefits of Vec: can iterate with loops, can index by worker_id, easy to pass ALL queues to each worker thread

    // Pushing 20 slow tasks to the first worker queue
    for id in 0..20 {
        worker_queues[0].push_local(Task::Compute { id, workload: 200 });
    }

    // Pushing 5 fast tasks to the rest of the worker queues
    for id in 0..5 {
        worker_queues[1].push_local(Task::Compute { id, workload: 50 });
        worker_queues[2].push_local(Task::Compute { id, workload: 50 });
        worker_queues[3].push_local(Task::Compute { id, workload: 50 });
    }

    // The above will set up a scenario where Worker 0 is overloaded

    // When spawning threads, we need to wrap the entire Vec in an Arc
    // The inner arc allows multiple workers to share each individual queue
    // Needed for stealing: Worker 1 can access Worker 0's queue
    // The outer Arc is needed to allow multiple workers to share the list of all queues
    // Without this, can you can't move the Vec into multiple threads
    let all_queues = Arc::new(worker_queues);
    // We don't wrap the entire Vec in a Mutex because we are not modifying the vec itself, only the contents of the queues inside it, which are already wrapped in Mutex

    // As a note, you could wrap the entire Vec in Mutex instead of wrapping each WorkerQueue's tasks field
    // struct WorkerQueue { tasks: VecDeque<Task> }  // No Mutex on field
    // let all_queues = Arc::new(Mutex::new(vec![...]));  // Mutex on Vec
    //
    // But this creates coarse-grained locking where all threads fight for the same lock:
    // - Worker 0 locks the Vec to access queue[0]
    // - Workers 1, 2, 3 are BLOCKED, even though they want different queues
    // - Only ONE worker can access ANY queue at a time
    // - This defeats the entire purpose of work stealing!
    //
    // With fine-grained locking (Mutex on each queue's tasks):
    // - Worker 0 locks only queue[0].tasks
    // - Workers 1, 2, 3 can simultaneously lock their own queues
    // - Workers only contend when actually stealing from the same queue
    // - Much better performance: workers work in parallel on their own queues
    //
    // Coarse-grained: Arc<Mutex<Vec<WorkerQueue>>> → ONE lock, high contention
    // Fine-grained: Vec<Arc<WorkerQueue { tasks: Mutex<...> }>> → Multiple locks, low contention

    // Wrap the entire Vec in Arc so each worker thread can share access to all queues
    // Arc<Vec<Arc<WorkerQueue>>> structure:
    // - Outer Arc: shares the Vec itself across threads
    // - Inner Arc: shares each individual WorkerQueue across threads (for stealing)

    // Create a vector to store handles
    // Handles are a way of interacting with spawned threads
    let mut handles: Vec<JoinHandle<Stats>> = Vec::new();

    // Spawn 4 worker threads
    for worker_id in 0..4 {

        // Creating a new pointer to the same data
        // Allows multiple owners of the same Vec of WorkerQueue
        // This will be moved into each individual thread so it can be used after the loop iteration ends
        // We need to wrap the entire vector in Arc so that it can be shared across all threads
        let queue_clone = Arc::clone(&all_queues);

        // Moving the captured variables into the closure (thread) so they can be used once the loop iteration ends
        let handle = thread::spawn(move || {

            // Creating local variables to track thread-specific statistics
            // This will ultimately go into the Stats struct
            let mut local_tasks_completed = 0;
            let mut stolen_tasks_completed = 0;
            let mut steal_attempts = 0;
            let mut failed_steals = 0;

            // Creating a local queue for each thread
            // The local queue will be the result of indexing the worker queue vector using worker_id
            // So thread 0 will have queue 0, thread 1 will have queue 1, etc.
            let local_queue = &queue_clone[worker_id];

            // This loop will be for the local worker and stealing logic
            // We need to have local work and stealing work in the same loop because we want stealing to happen repeatedly whenever the local queue is empty

            // Worker strategy:
            // - Local work: pop_back() = LIFO (Last In, First Out) - most recent tasks (cache-hot)
            // - Stealing: pop_front() = FIFO (First In, First Out) - oldest tasks (less likely cache-hot)
            // This prevents conflict: worker processes recent work, thieves take old work

            // We are using if let since we only care about the success case (there is a task in the queue to be processed)
            // We could use a match statement but that is more verbose and unnecessary
            // If there is an empty queue (no task to be processed), we move onto the next step (trying to steal)
            // We are using a loop since we DON'T know how many tasks we will get
            // The worker doesn't know:
            // - How many tasks are in the local queue initially
            // - How many tasks it will steal from others
            // - When the Shutdown signal will arrive
            'worker_loop: loop { // We are labeling the main loop as worker_loop here so it is explicit we are breaking out of the main loop
                                // This is especially useful since we will have a nested loop within, so it will be more clear what we are exiting
                // === Phase 1: Try local work first ===
                if let Some(task) = local_queue.pop_local() {
                    match task {
                        Task::Compute { id, workload } => {
                            println!("Worker {}: Processed {}. Now sleeping for {}ms.", worker_id, id, workload);
                            thread::sleep(Duration::from_millis(workload));
                            local_tasks_completed += 1;
                        }
                        // If the task we receive is Shutdown, break the entire loop (the thread will have finished its work)
                        // This is part of the poison pill pattern
                        Task::Shutdown => {
                            println!("Worker {}: Shutting down.", worker_id);
                            break 'worker_loop;
                        }
                    }
                // We have the main logic and stealing logic inside of the same loop since we want stealing to happen repeatedly whenever the local queue is empty not just once
                } else {
                    // === Phase 2: Local empty, try stealing from other workers ===

                    // local queue empty, try stealing
                    let mut stole_task = false; // Track if we successfully stole anything

                    // If we do not have a task to process, we start looking for victims
                    for victim_id in 0..4 {
                        // Since threads should not steal from themselves, we always need to skip them
                        if victim_id == worker_id {
                            // continue is used to end this loop iteration and move on to the next one immediately
                            continue;
                        }
                        steal_attempts += 1;
                        // If .steal() results in Some(task), we bind it to task
                        // And then process the task
                        if let Some(task) = queue_clone[victim_id].steal() {
                            match task {
                                Task::Compute { id, workload } => {
                                    println!("Worker {}: Processed {}, stolen from Worker {}. Now sleeping for {}ms.", worker_id, id, victim_id, workload);
                                    thread::sleep(Duration::from_millis(workload));
                                    stolen_tasks_completed += 1;
                                    stole_task = true;
                                    // Found work, stop trying other victims
                                    // This will only break the inner (victim) loop
                                    // After processing a stolen task, you got back to the top of the worker loop and try your local queue again (from the top)
                                    break;
                                }
                                // If the task we receive is Shutdown, break the entire loop (the thread will have finished its work)
                                Task::Shutdown => {
                                    println!("Worker {}: Shutting down.", worker_id);
                                    break 'worker_loop; // We break the entire worker loop not just the victim loop
                                }
                            }
                        } else {
                            failed_steals += 1;

                            // This continue is redundant since the loop will continue automatically
                            // We could leave it here just to be explicit
                            // continue;

                            // We will not sleep here after EACH failed steal 
                            // We want the thread to briefly sleep after the entire for loop tries all of its victims
                            // If you put it to sleep after every failed steal, you add unnecessary sleep time for the thread
                        }
                    }
                    // === Phase 3: No work found anywhere, sleep briefly ===

                    // After trying ALL victims, if we didn't steal anything, sleep
                    if !stole_task {
                    // Check if ALL queues are empty before sleeping

                        // .all() is an iterator method that checks if ALL elements satisfy a condition
                        // Returns true if the condition is true for every element
                        // Returns false if any element fails the condition
                        // Short-circuits when it finds the first false
                        // It is iterator.all(|item| condition)

                        // (0..4) creates an iterator
                        // .all(|i| {...}) checks if the condition is true for all worker IDs, i = current worker ID
                        // .all() is not lazy - it's a consuming method that executes immediately - there is no separate consumption step
                        // Iterator methods will iterate over all elements, unless they have short circuiting
                        // Lazy iterator methods return another iterator, chain together, and don't execute until consumed
                        // Eager iterator methods return a concrete value (not an iterator) and execute immediately
                        let all_empty = (0..4).all(|i| {
                            queue_clone[i].tasks.lock().unwrap().is_empty() // Checking if all tasks vectors are empty for each queue
                        });
                        
                        if all_empty {
                            println!("Worker {}: All queues empty, exiting", worker_id);
                            break 'worker_loop;  // Exit when truly no work left
                        }

                        thread::sleep(Duration::from_millis(10))
                        // After the thread sleeps for 10 milliseconds, we go back to the start of the worker loop
                        // Sleep for 10 milliseconds to avoid busy-waiting (hammering CPU)
                        // Still responsive enough to check for new work frequently;
                    }
                }
            }
            // For this entire loop, we could use match statements instead of if let
            // but this might be more verbose
            // In the outer loop, if we encountered None we would go to the stealing logic None => {stealing}
            // In the inner loop, if we encountered None, we would just increment the failed steals

            // Create a thread-specific struct of stats
            Stats {
                local_tasks_completed,
                stolen_tasks_completed,
                steal_attempts, 
                failed_steals,
            }
        });

        // Pushing the handles to the vector so that we can iterate over it and call .join() later
        handles.push(handle);
    }

    println!("Waiting for all workers to finish...\n");

    // Create an empty vector to store the Stats struct from each thread
    let mut all_stats: Vec<Stats> = Vec::new();

    for handle in handles {
        // First, calling .join() to allow all threads to finish before continuing
        // We are storing this in a variable since the threads will produce a struct as output
        let stats = handle.join().unwrap();
        // .unwrap() panics if the thread panicked - acceptable for this example
        // In production, you'd handle the Result properly
        all_stats.push(stats);
    }   


    println!("=== Individual Worker Statistics ===");
    // We are using a for loop with a tuple allowing us to put 2 items in the loop
    // We are using .enumerate(), which gives us both the index and item in the vector (Stats struct)
    // worker_id = the index (0, 1, 2, 3)
    // stats = reference to the Stats struct at that index
    for (worker_id, stats) in all_stats.iter().enumerate() {
        // We can destructure what we are iterating over using tuples
        // You can also do this with for, match, if let, let, function parameters, etc.
        println!("Worker {}:", worker_id);
        println!("  Local tasks completed: {}", stats.local_tasks_completed);
        println!("  Stolen tasks completed: {}", stats.stolen_tasks_completed);
        println!("  Steal attempts: {}", stats.steal_attempts);
        println!("  Failed steals: {}", stats.failed_steals);
        println!();
    }
}

// Expected behavior:
// - Worker 0 starts with 20 tasks (200ms each) = 4000ms of work
// - Workers 1,2,3 start with 5 tasks each (50ms each) = 250ms of work
// - Without stealing: Worker 0 takes 4s alone, others idle after 250ms
// - With stealing: Workers 1,2,3 steal from Worker 0 → balanced workload → faster completion