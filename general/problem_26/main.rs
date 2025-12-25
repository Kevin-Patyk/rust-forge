#![allow(dead_code)]
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct Task {
    id: u32,
    description: String,
    completed: bool,
    priority: u32,
}

struct Project {
    name: String,
    // This is a vector of shared ownership pointers to mutable tasks
    // Vector of reference counted pointers to tasks with interior mutability
    // Rc allows for multiple owners
    // RefCell allows for interior mutability (mutation through shared references)
    tasks: Vec<Rc<RefCell<Task>>>,
    // This allows Task to have multiple owners and also be mutated from those multiple owners
}

struct TaskManager {
    // This allows Task to have multiple owners and also be mutated from this multiple owners
    tasks: Vec<Rc<RefCell<Task>>>,
    projects: Vec<Project>,
}

impl Task {
    fn new(id: u32, description: String, priority: u32) -> Self {
        Self {
            id,
            description,
            completed: false,
            priority,
        }
    }

    fn complete(&mut self) {
        self.completed = true;
    }

    fn is_high_priority(&self) -> bool {
        self.priority >= 4
    }
}

impl Project {
    fn new(name: String) -> Self {
        Self {
            name, 
            tasks: Vec::new(),
        }
    }

    fn add_task(&mut self, task: Rc<RefCell<Task>>) {
        self.tasks.push(task);
    }

    fn incomplete_count(&self) -> usize {

        // self.tasks.iter().map(|task| {
        //     let borrowed = task.borrow();
        //     !borrowed.completed as usize
        // }).sum()

        // The first way works, but this is more idiomatic Rust 
        // That filters out any completed tasks and counts them
        self.tasks.iter().filter(|task| !task.borrow().completed).count()
    }
}

impl TaskManager {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
            projects: Vec::new(),
        }
    }

    fn add_task(&mut self, task: Task) -> Rc<RefCell<Task>> {
        let wrapped = Rc::new(RefCell::new(task));

        // & makes it clear that you're cloning the pointer not the data
        // The & emphasizes "this is just incrementing a counter, not deep copying"
        self.tasks.push(Rc::clone(&wrapped));

        wrapped
    }

    fn add_project(&mut self, project: Project) {
        self.projects.push(project);
    }

    fn complete_task(&mut self, id: u32) -> Result<(), String> {
        // .find() will always returns the Task not a unit type
        // We added the ? operator to propagate the error if the Task is not found
        // .ok_or_else() converts an Option to a Result, with the closure being called lazily (if it is the None variant of Option)
        let task = self.tasks.iter().find(|task| task.borrow().id == id).ok_or_else(|| format!("Task with ID {} not found.", id))?;

        // We call the complete method to mark the Task as complete
        // DRY - Don't Repeat Yourself
        // We need .borrow_mut() here since we are changing the field
        task.borrow_mut().complete();
        // Now, we return the unit type wrapped in Ok()
        Ok(())
    }

    fn high_priority_tasks(&self) -> HighPriorityIter<'_> {
        let task_refs: Vec<&Rc<RefCell<Task>>> = self.tasks.iter().collect();

        HighPriorityIter { tasks: task_refs, index: 0 }
        // In our problem, we are using an iterator as a learning example
        // We could also, for example, just use .filter() since it is clean and simple
        // We need custom iterators for:
        // Complex filtering logic with state, such as skipping every other high priority task
        // Multi step iteration (phases)
        // Computed/generated items
        // Wrapping complex data structures
        // Performance critical with custom logic
        // Custom iterators are powerful when you need complex state or logic that combinators like .filter() and .map() can't express cleanly
    }

}

// The lifetime annotation a is saying: "The HighPriorityIter struct can only live as long as the Tasks it references"
// The Vec itself is owner by HighPriorityIter
// The references inside of the Vec must live for a
// a ties the iterator's lifetime to the original data in TaskManager
// "The HighPriorityIter cannot outlive the TaskManager it borrowed from because it holds references to the TaskManager's tasks." 
struct HighPriorityIter<'a> {
    // To make this a Vector of references, since TaskManager takes Vec<Rc<RefCell<Task>>>, we need to use .iter().collect()
    // This will create references to all of the inner elements 
    // .iter().collect() is a pattern to convert owned items into a Vec of references
    tasks: Vec<&'a Rc<RefCell<Task>>>,
    index: usize,
}

// I am implementing for a type with lifetime a
// Implementing the Iterator trait for my custom struct
// The iterator trait in Rust is the core abstraction for anything that can produce a sequence of values, one at a time
// Produces a sequence of values - next() is the core method that yields items
impl<'a> Iterator for HighPriorityIter<'a> {

    // Every iterator must define what type it yields
    // type Item =... is an associated type (part of the Iterator trait contract)
    // Each iteration returns a reference to an Rc<RefCell<Task>>>
    // The a ensures references live as long as the original data
    type Item = &'a Rc<RefCell<Task>>;

    // This is the next() method signature
    // We need &mut self since we need to change the internal state
    // Returns Option<Self::Item> where Some(item) means here is the next item
    // None means the iteration is done
    fn next(&mut self) -> Option<Self::Item> {
        
        // Keep looping as long as there are more tasks to check
        // self.index tracks our current position 
        while self.index < self.tasks.len() {
            // This gets the current task - access the task at the current position
            let task = self.tasks[self.index];
            // Increment the index
            // Move to the next position for next time
            // This happens before the check, so we don't get stuck on the same task
            self.index += 1;

            // Now, we check if a Task is high priority
            // If it is, we immediately return (exit the function immediately) and return Some(task) 
            if task.borrow().is_high_priority() {
                return Some(task);
            }
            // If not high priority, the loop continues
            // When 
        }
        // If while loop finishes, we have checked all the tasks
        // Return None to signal the iteration is complete
        None
    }

    // The iterator doesn't return everything at once - it returns one at a time on demand
    // After you create the iterator, you need to call .next() on it or something that uses .next() internally
    // In our case, each .next() call returns Option<&Rc<RefCell<Task>>>
    // On the first iteration (first call to .next()), it will look for a high priority task and if it finds one, it returns Some()
    // On the second iteration (second call to .next()), it will again look for a high priority task and if it finds one, it return Some()
    // If there are no more high priority tasks, it returns None
    // You can use the Iterator in for loops, collecting into a Vector, and manual iterations
    // The iterator is lazy - it doesn't find all high-prio tasks up front - it does it one at a time as you request by calling .next()
    // This is memory efficient and allows for early termination
    // Pattern: Filtering/transforming during iteration
}


// This function takes the task_id
// And it returns something that implements the FnOnce trait, which takes &mut TaskManager as input and returns Result<(), String> as output
// Can only be called once
fn create_completer(task_id: u32) -> impl FnOnce(&mut TaskManager) -> Result<(), String> {

    // task_id is the object that is captured
    // manager is the closure's parameter
    // manager.complete_task(task_id) is using the captured value
    move |manager| manager.complete_task(task_id) 

    // This function takes task_id as a parameter
    // The closure captures task_id by VALUE cause of move
    // move transfers ownership of the task_id into the closure
    // Allows the closure to outlive the function scope
    // The closure owns the captured data
}

// The move keyword transfers ownership of captured variables from the surrounding scope into the closure
// Types that implement Copy: the variable is copied into the closure
// Type that do not implement Copy: variable is moved into the closure and the closure now owns it
// Without move, closure borrows variables (references)
// move = "Take ownership of everything I captured from the surrounding scope."

fn main() {
    let mut task_manager = TaskManager::new();

    // Add tasks to manager - it wraps them and returns Rc<RefCell<Task>>
    let task1 = task_manager.add_task(Task::new(1, "task1".to_string(), 1)); // 2 additions to the reference count (task_manaher.tasks) and the variable that holds it
    let task2 = task_manager.add_task(Task::new(2, "task2".to_string(), 2));
    let task3 = task_manager.add_task(Task::new(3, "task3".to_string(), 3));
    let task4 = task_manager.add_task(Task::new(4, "task4".to_string(), 4));
    let task5 = task_manager.add_task(Task::new(5, "task5".to_string(), 5));

    let mut project1 = Project::new("project1".to_string());
    let mut project2 = Project::new("project2".to_string());
    let mut project3 = Project::new("project3".to_string());

    project1.add_task(Rc::clone(&task1));
    project1.add_task(Rc::clone(&task4));

    project2.add_task(Rc::clone(&task2));
    project2.add_task(Rc::clone(&task4));  // task4 in multiple projects!
    project2.add_task(Rc::clone(&task5));

    project3.add_task(Rc::clone(&task3));
    project3.add_task(Rc::clone(&task5));  // task5 in multiple projects!

    println!("Reference count for task 4 {}", Rc::strong_count(&task4)); // Should be 4
    println!("Reference count for task 5 {}", Rc::strong_count(&task5)); // Should be 4

    task_manager.add_project(project1);
    task_manager.add_project(project2);
    task_manager.add_project(project3);

    println!("Task 4 status: {}", task4.borrow().completed);

    for project in &task_manager.projects {
        println!("Project {}, incomplete tasks {}", project.name, project.incomplete_count())
    }

    match task_manager.complete_task(4) {
        Ok(()) => println!("Task 4 completed successfully!"),
        Err(e) => println!("Error: {}", e),
    }

    println!("Task 4 completed status: {}", task4.borrow().completed);

    // Since task4 is in multiple projects, now that it is complete, it will show 1 less incomplete tasks for the projects it is in
    // This proves shared and mutable ownership with Rc<RefCell<>> - one update affects all references
    for project in &task_manager.projects {
        println!("Project '{}' incomplete tasks: {}", project.name, project.incomplete_count());
    }

    // Using the custom iterator with a for loop (which automatically calles .next())
    for task in task_manager.high_priority_tasks() {
        let borrowed = task.borrow();
        println!("Task {}: {} (Priority: {})", borrowed.id, borrowed.description, borrowed.priority)
    }

    // Alternative - collect them
    let high_pri_tasks: Vec<_> = task_manager.high_priority_tasks().collect();
    println!("Found {} high-priority tasks", high_pri_tasks.len());

    // If we do just task_manager.high_priority_tasks() in a println!, it will create a new iterator each time 
    // This needs to be mutable since .next() modifies the internal state
    let mut iter = task_manager.high_priority_tasks();

    println!("{:?}", iter.next());
    println!("{:?}", iter.next());
}
