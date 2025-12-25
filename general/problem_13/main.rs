#[allow(dead_code)]
struct Task {
    id: u32,
    title: String,
    description: String,
    status: Status,
    priority: Priority,
}

// Copy and Clone both create copies, but they work differently
// Copy automatically copies the value when it is used - it happens implicitly behind the scenes 
// You don't have to do anything - it just happens

// Clone manually creates a copy when you explicitly call .clone(). It doesn't happen automatically 
// you have to write .clone()

// For small, simple types, like ints/floats/bools/simple enums, use Copy 
// Copy is best used for anything that's cheap to copy and should copy automatically

// For larger, more complex types like strings/vectors/structs, use Clone
// These are expensive to copy, so you want explicit control 
// You only call .clone() when you actually need a copy
#[derive(PartialEq, Clone, Debug, Copy)]
#[allow(dead_code)]
enum Status {
    NotStarted,
    InProgress,
    Completed,
}

#[derive(PartialEq, Clone, Debug, Copy)]
enum Priority {
    Low, Medium, High
}

struct TaskManager {
    tasks: Vec<Task>,
    next_id: u32,
}

// As a note, you can also use Self here instead of TaskManager
// Self is just an alias for the implementing type
// Self is often preferred because its continues to work after you rename the struct
// It's idiomatic in Rust for constructors and builder-like methods
impl TaskManager {
    fn new() -> TaskManager {
        TaskManager { 
            tasks: Vec::new(),
            next_id: 1,
        }
    }

    // Enums can't be empty the way strings can
    // An enum is always one of its variants - it always has a value
    // When someone calls add_task(), they have to pass a Priority enum value - they can't pass nothing
    // Strings are different because you can have an empty string such as ""
    fn add_task(&mut self, title: String, description: String, priority: Priority) -> Result<String, String> {
        if title.is_empty() || description.is_empty() {
            Err("Both title and description are required.".to_string())
        } else {
            let current_id = self.next_id;
            self.next_id += 1;

            self.tasks.push(Task {
                id: current_id,
                title, 
                description,
                status: Status::NotStarted,
                priority
            });

            Ok("Task added.".to_string())
        }
    }

    // Here, we search through the vector of tasks in TaskManager.tasks field
    // If we find a task (struct) where the id field matches the input task_id,
    // we update the task status
    // If not, raise an error
    fn update_task_status(&mut self, task_id: u32, new_status: Status) -> Result<String, String> {
        match self.tasks.iter_mut().find(|task| task.id == task_id) {
            Some(task) => {
                // We need to clone here since we are missing new_status when we assign it to task.status
                // Thus, we will not be able to use it again in the format!() macro
                // When we do this, Rust takes ownership of new_status and moves it into the task
                // After that, new_status no longer exists - it's been moved 
                // .clone() creates a copy of new_status so you can move one copy into the task and still have the original
                task.status = new_status.clone();
                Ok(format!("Successfully updated the task status to: {:?}", new_status))
            }
            None => Err("Task not found.".to_string())
        }
    }

    fn get_tasks_by_priority(&self, priority: Priority) -> Vec<&Task> {
        // We are iterating over the self.tasks vector to find any structs (Tasks)
        // that match the provided priority
        // Remember .filter() returns all matches found
        self.tasks.iter().filter(|task| task.priority == priority).collect()
    }

    fn get_task_by_id(&self, task_id: u32) -> Option<&Task> {

        // .find() already returns an Option, so we can just return it directly without a match statement
        self.tasks.iter().find(|task| task.id == task_id)

        // match self.tasks.iter().find(|task| task.id == task_id) {
        //     Some(task) => Some(task),
        //     None => None,
        // }
    }

    // if let is used when you only care about one specific case and want to ignore the others
    // Use match when you need to handle all cases explicitly 
    // Use if let whe nyou only care about one case and don't want to do anything with the others
    // if let Some(task) = self.tasks.iter().find(|task| task.id == task_id) {println!(...)}
    // if let is not only for Options, you can use it with Results, Enums, or any pattern really
    // The main idea is that you only care about one specific case and want to ignore all others
    // It is a general pattern-matching tool, not specific to Option
}

fn main() {
    let mut task_manager_1: TaskManager = TaskManager::new();

    match task_manager_1.add_task("title1".to_string(), "description1".to_string(), Priority::Low) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match task_manager_1.add_task("title2".to_string(), "description1".to_string(), Priority::Medium) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match task_manager_1.add_task("".to_string(), "".to_string(), Priority::High) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    let low_priority_tasks = task_manager_1.get_tasks_by_priority(Priority::Low);

    for task in low_priority_tasks {
        println!("ID: {}, Title: {}, Priority: {:?}", task.id, task.title, task.priority);
    }

    let medium_priority_tasks = task_manager_1.get_tasks_by_priority(Priority::Medium);

    for task in medium_priority_tasks {
        println!("ID: {}, Title: {}, Priority: {:?}", task.id, task.title, task.priority);
    }

    match task_manager_1.get_task_by_id(1) {
        Some(task) => println!("Task found. ID: {}, Title: {}, Priority: {:?}", task.id, task.title, task.priority),
        None => println!("This task does not exist within the system."),
    }

    match task_manager_1.get_task_by_id(99) {
        Some(task) => println!("Task found. ID: {}, Title: {}, Priority: {:?}", task.id, task.title, task.priority),
        None => println!("This task does not exist within the system."),
    }

    match task_manager_1.update_task_status(1, Status::InProgress) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match task_manager_1.update_task_status(99, Status::InProgress) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match task_manager_1.get_task_by_id(1) {
        Some(task) => println!("{}", task.id),
        None => println!("Task not found."),
    }

    match task_manager_1.get_task_by_id(2) {
        Some(task) => println!("{}", task.id),
        None => println!("Task not found."),
    }
}
