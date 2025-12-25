struct Book {
    title: String,
    author: String,
    copies_available: u32,
    borrowed_by: Vec<String>,
}

struct Library {
    books: Vec<Book>,
}

fn borrow_book(library: &mut Library, book_title: String, member_name: String) -> Result<String, String> { 
    // library.books is a Vec<Book> so .iter_mut() gives you mutable references to each book
    // Then .find() searches through them and returns Option<&mut Book>
    // Since we need to modify the book fields and push, we need mutable references
    match library.books.iter_mut().find(|item| item.title == book_title) {
        Some(book) => {
            if book.copies_available < 1 {
                // Since we are returning a Result<String, String>, we need to convert to string
                Err("That book is not currently available.".to_string())
            } else {
                // Since book.borrowed_by is a vector, we need to update it by pushing 
                book.borrowed_by.push(member_name);
                book.copies_available -= 1;
                // Since we are returning a Result<String, String>, we need to convert to string
                Ok("Book borrowed successfully.".to_string())
            }
        }
        None => Err("That book does not exist in the library's system.".to_string())
    }
}

// Result<T, E> means:
// Ok(T) - success, here's the value
// Err(E) - failure, here's the error
fn return_book(library: &mut Library, book_title: String, member_name: String) -> Result<String, String> {
    match library.books.iter_mut().find(|item| item.title == book_title) {
        Some(book) => {
            // .position() is similar to .find(), but rather than returning the item in the collection itself, it returns 
            // an index (position) of that item in the vector
            // With .find(), you get the value. With .position(), you get where it is in the list
            if let Some(index) = book.borrowed_by.iter().position(|name| name == &member_name) {
                // .remove() is a vector method that removes an item at a specific index and returns it
                book.borrowed_by.remove(index);
                book.copies_available += 1;
                Ok("Book returned successfully.".to_string())
                // When using if let, we do not need an else block - it is optional
                // if let just says "if this matches, do this. Otherwise, do nothing"
                // In our case we probably want an else block because we want to handle the error case where the member didn't borrow the book
                // if let doesn't require one - it just depends on your logic
            } else {
                Err("Member did not borrow this book.".to_string())
            }
        }
        None => Err("That book does not exist in the library's system.".to_string())
    }
}

// Using an Option<T> means the function can return:
// Some(T) - success, here is the value of type T
// None - failure, no value to return
fn get_book_info(library: &Library, book_title: String) -> Option<(String, u32)> {
    match library.books.iter().find(|item| item.title == book_title) {
        // We need .clone() on the book.author because it is a String and we can't move it 
        // out of the book
        // .clone() makes a copy of the string so you can return it
        Some(book) => Some((book.author.clone(), book.copies_available)),
        // We are returning None here since it is an Option, not a Result
        None => None
    }
}

fn main() {
    // Here we are making a mutable Library struct
    // This struct takes a vector of Book structs
    // Within each Book struct, there are 4 fields (title, author, copies_available, and borrow_by)
    let mut library = Library {
        books: vec![
            Book {
                title: "Rust Programming".to_string(),
                author: "Steve Klabnik".to_string(),
                copies_available: 3,
                borrowed_by: vec![],
            },
            Book {
                title: "The Pragmatic Programmer".to_string(),
                author: "David Thomas".to_string(),
                copies_available: 1,
                borrowed_by: vec![],
            },
            Book {
                title: "Clean Code".to_string(),
                author: "Robert Martin".to_string(),
                copies_available: 0,
                borrowed_by: vec!["Alice".to_string()],
            },
        ],
    };

    // Using match statements to print the result for each test
    match borrow_book(&mut library, "Rust Programming".to_string(), "Bob".to_string()) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match borrow_book(&mut library, "Clean Code".to_string(), "Charlie".to_string()) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match borrow_book(&mut library, "Python Basics".to_string(), "Diana".to_string()) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match get_book_info(&library, "The Pragmatic Programmer".to_string()) {
        // Since get_book_info() returns an Option<(String, u32)>, 
        // we need to destructure the tuple inside the Some arm 
        // The (author, copies) destructures the tuple so you can access each part separately
        Some((author, copies)) => println!("Author: {}, Copies available: {}", author, copies),
        None => println!("Book not found.")
    }

    match return_book(&mut library, "Rust Programming".to_string(), "Bob".to_string()) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match return_book(&mut library, "Rust Programming".to_string(), "Eve".to_string()) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }
}
