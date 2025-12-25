// New problem: Worker thread pool with task queue

// Concept: Build a resuable worker pool that processes tasks from a shared queue - the foundation of most concurrent systems

// What we will learn:
// - Worker pool pattern (spawn once, reuse threads)
// - Shared task queue with Arc<Mutex<VecDeque<T>>>
// - Tracking task completion with atomics
// - Graceful shutdown (poison pill pattern)
// - This is what libraries like Rayon/Tokio do

// The pattern:
// 1. Main thread adds tasks to a shared queue
// 2. The workers pull tasks from the shared queue and work on them
// 3. When workers finish the tasks from the shared queue, the main thread is notified

// Our task is to build a processing system where:
// 1. Main thread creates a shared task queue
// 2. Spawn 4 worker threads that pull tasks from the queue
// 3. Workers process tasks concurrently
// 4. Track how many tasks each worker completes
// 5. Gracefully shut down all workers when the queue is empty 

#[allow(dead_code)]
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
enum Task {
    Process { id: usize, value: i32}, // struct-like variant
    Shutdown, // Special task to tell workers to exit
}

struct TaskQueue {
    // VecDeque is a double-ended queue from Rust's standard library
    // It is a growable ring buffer that lets you efficiently: 
    // Push front and back
    // Pop front and back
    // VecDeque is ideal for: task queues, schedulers, and producer/consume patterns
    // We wrap it in Mutex so multiple threads can share tasks, but only one thread can modify/read at a time
    tasks: Mutex<VecDeque<Task>>, // This is a VecDeque of Task wrapped in Mutex
    // We do not need Arc here since it is dereferenced automatically and it allows it to accept Arc<Mutex>> and Mutex<>
    total_completed: AtomicUsize,
}
// VecDeque = Vector Double-Ended Queue
// It is like Vec but you can efficiently add/remove from both ends (front and back)
// Vec can only efficiently work with the back
// For a queue, we use this because it follows First In, First Out (FIFO)
// Add to back, remove from front (push back, pop front)
// We use FIFO so tasks are processed in the order they were added

// In our example, the main thread will push tasks to the back
// And workers will take the oldest task from front 

impl TaskQueue {
    // Associated function
    // Does not need self to work
    // Creates an instance of the struct
    fn new() -> Self {
        Self {
            tasks: Mutex::new(VecDeque::new()),
            // AtomicUsize provides interior mutability - can be modified through &self
            // No need for Mutex here since atomic operations are inherently thread safe
            // Multiple threads can safely increment this counter concurrently
            total_completed: AtomicUsize::new(0),
        }
    }

    // The main thread will be calling this to add tasks to the queue
    fn add_task(&self, task: Task) {
        // We need .lock() since we need to acquire the lock since it is wrapped in Mutex
        // We need .unwrap() in case a thread panics
        // .push_back() adds the task to the back of the queue
        // Lock is held during this operation
        // Lock releases when the statement ends
        // Before: [Task1, Task2, Task3]
        // After:  [Task1, Task2, Task3, NewTask] ← added to back
        self.tasks.lock().unwrap().push_back(task);
    }

    // The worker threads will be calling this to get tasks from the queue
    fn get_task(&self) -> Option<Task> {
        // .pop_front() removes and returns the task from the front of the queue
        // Returns Some(task) if the queue has items
        // None if queue is empty
        // Lock releases when the statement ends
        // Before: [Task1, Task2, Task3, Task4]
        // After:  [Task2, Task3, Task4]
        // Returns Some(Task1)
        // If empty []: Returns None
        self.tasks.lock().unwrap().pop_front()
    }

    fn mark_completed(&self) {
        self.total_completed.fetch_add(1, Ordering::SeqCst);
    }

    fn completed_count(&self) -> usize {
        self.total_completed.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone)]
// Clone not strictly needed here, but Debug is useful for printing
struct WorkerStats {
    worker_id: usize,
    tasks_completed: usize,
}

fn process_task(task: &Task) {
    match task {
        Task::Process {id, value} => {
            // Simulate varying processing times
            let sleep_time = if value % 3 == 0 { 200 } else { 50 };
            thread::sleep(Duration::from_millis(sleep_time));
            println!("  Processed task {} (value={})", id, value);
        }
        Task::Shutdown => {}
    }
}

fn main() {
    // Create a shared task queue
    // It needs to be wrapped in Arc so multiple threads can own it
    // Will have multiple pointers to the same data
    let queue = Arc::new(TaskQueue::new());

    // Create 20 tasks with different values
    // Since .add_task() requires a lock due to it being wrapped in Mutex,
    // this is just the main thread acquiring the lock over and over again (20 times in a row)
    // No threads can interfere since worker threads haven't spawned yet
    println!("Adding 20 tasks to queue...");
    for i in 0..20 {
        queue.add_task(Task::Process { id: i, value: i as i32 * 3});
    }
    // Now we will have a Mutex<VecDeque> of 20 Task structs
    // Everything will be added from the back for FIFO due to .push_back()
    // [Task1, Task2, Task3, ...] <- Task4

    // Creating an empty vector to store handles
    // Handles are a way of interacting with spawned threads
    // Workers immediately start pulling tasks as soon as they spawn
    // At this point, only workers compete for the queue lock
    // Main thread will briefly compete again when adding Shutdown tasks
    let mut handles: Vec<JoinHandle<WorkerStats>> = Vec::new();

    for worker_id in 0..4 {

        // Creating a clone of the queue
        // Incrementing the reference count
        // This creates a new pointer to the same data
        // This will be moved into the thread so the thread can continue using it even if the loop iteration ends
        // The thread's lifetime is independent and it needs to be able to use queue_clone after the loop iteration ends
        let queue_clone = Arc::clone(&queue);

        let handle = thread::spawn(move || {

            // Each thread will have its own completed count
            // So we are making it in the thread and then updating it every time the loop processes a task
            // We want each thread to have a unique WorkerStats struct and count of tasks completed
            let mut completed: usize = 0;

            // We are using a loop since we don't know how many tasks the thread will receive from the shared queue
            // The loop continues indefinitely until:
            // 1. A Shutdown task is received (break exits the loop)
            // 2. The thread has processed all its assigned work
            loop {
                // We need to store the result of .get_task() in a variable then check it
                // This is because .pop_front() removes the task from the queue
                // If we call it twice in a match statement, the first call gets the first task and removes it
                // The second call would pull from a different or empty queue
                let task = queue_clone.get_task();
                // .get_task() acquires the lock here and it is dropped at the semi-colon
                // So then another thread can get a Task for the queue
                // Minimal lock scope

                // Rather than using a nested match statement, we are matching on different variants of Some()
                // This is shorter than using a nested match statement
                // When we have Option<Task>, we can match on the nested structure in one step
                match task {
                    // This will match if the Option is Some AND the inner Task is the Process variant
                    // It "looks inside" the option in one pattern
                    Some(t @ Task::Process { id: _, value: _ }) => {
                        // The @ operator lets you bind a variable to a pattern while still matching on it
                        // Without the @ operator, we would not have access to the Task, we matched it but didnt capture it
                        // When we match on Some(variant), we are checking if the pattern matches but we don't have a variable holding the actual Task, so we need @
                        // "Match this pattern AND give me a variable that holds the matched value"
                        process_task(&t);
                        completed += 1;
                        queue_clone.mark_completed();
                    }
                    // If we do not have any tasks the contain Shutdown, this loop would continue forever
                    Some(Task::Shutdown) => {
                        // Break immediately exits the loop
                        // The worker thread will end after the break
                        break;
                    }
                    None => {
                        // Queue is temporarily empty - sleep and retry
                        // This prevents the worker from exiting if the queue is just momentarily empty
                        // Without this, workers would exit as soon as they see None
                        // even if more tasks are being added by other threads
                        // Common pattern for worker pools with unknown workload
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }
            }

            // Create a WorkerStats struct after the loop ends
            // If the thread encounters a shutdown as its first task, then the completed will be 0
            // Since the loop will break right away and the completed count will not get incremented
            WorkerStats {
                worker_id,
                tasks_completed: completed,
            }
            // In the thread, the WorkerStats struct will be created after the loop ends
            // The thread will only end when a Shutdown task is received since, if TaskQueue is None, it sleeps then loops again
        });

        // Pushing the handle to the vector so that we can use .join() on them later
        handles.push(handle);
    }

    // Send shutdown signal to all workers
    // If we did not do this, then the loop in each thread would continue forever
    println!("\nSending shutdown signals...");
    // We need 4 in total (one for each worker thread)
    for _ in 0..4 {
        queue.add_task(Task::Shutdown);
    }
    // This is called the "poison pill" pattern
    // 1. Producer adds real work to the queue
    // 2. When done, producer adds "poison pills" (Shutdown tasks)
    // 3. Workers process real work normally
    // 4. When a worker gets a poison pill, it exits

    // Waiting for all workers and collect states
    println!("Waiting for workers to finish...\n");
    let mut all_stats = Vec::new();

    // Here, we will wait for each worker thread to finish (they finish when they get Shutdown)
    // Collects the WorkerStats struct each thread returns
    // Stores them in the all_stats vector
    for handle in handles {
        let stats = handle.join().unwrap();
        all_stats.push(stats); // Vector of structs
    }

    println!("Statistics");
    for stats in &all_stats {
        println!("Worker {}: completed {} tasks", stats.worker_id, stats.tasks_completed);
    }

    println!("Total completed: {}", queue.completed_count());
}

// The main thread does not really compete for the lock
// It only competes during the brief moment it adds Shutdown tasks
// Most of the time it is:
// 1. Adding tasks alone (before workers exist)
// 2. Idle while workers process
// 3. Waiting on .join()

// This pattern demonstrates:
// ✅ Worker pool (spawn once, reuse threads)
// ✅ Shared queue (Arc<Mutex<VecDeque<T>>>)
// ✅ FIFO processing (first added = first processed)
// ✅ Atomic counters (lock-free progress tracking)
// ✅ Graceful shutdown (poison pill pattern)
// ✅ Concurrent execution (workers race for tasks)
//
// Real-world uses:
// - Web servers (workers handle HTTP requests)
// - Job queues (Sidekiq, Celery, etc.)
// - Thread pools (Rayon, Tokio internals)
// - Database connection pools
// - Task schedulers

// In the thread pool example, we used a thread pool with a channel (queue was hidden)
// "I want to send work to workers (channel abstraction)"
// This teaches channel-based communication, higher-level abstraction, type-safe job passing, producer consumer pattern

// In this problem, we used a thread pool with explicit queue (queue was visible)
// "I want to manage the queue myself"
// This teaches manual queue management, lower level control, FIFO semantics automatically, custom task types

// Both achieve the same goal (reusable workers) just different levels of abstraction
// In both cases, main thread adds work (send() or add_task())
// Work sits in a queue (hidden channel or explicit VecDeque)
// Workers take work from queue (either recv() or get_task())