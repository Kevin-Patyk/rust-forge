struct Item {
    name: String,
    quantity: u32,
    price: f64,
}

// Here you do not need the inventory vector the be mutable
// If you have a mutable reference, like you do with buy_item, you can pass it to a function that needs an immutable reference
// Rust allows you to downgrade from mutable to immutable
// But you can't do the other way around - you can't pass in immutable reference to a function expecting a mutable one

// The lifetime annotation 'a tells Rust that the returned reference comes from inventory_vector, not from item_name
// The returned reference is valid as long as iventory vector is valid
fn find_item<'a>(item_name: &str, inventory_vector: &'a Vec<Item>) -> Option<&'a Item> {
    // .find() is a vector method that searches through items and returns an Option
    // It takes a closure that checks each item 
    // .find() goes through each item one by one, runs the closure on it, and stops at the first item
    // where the closure returns true
    // If it goes through all items and never finds one where the closure returns true, it returns None
    // The .iter() part lets you loop through references instead of consuming the vector
    // If there are multiple items of the same name, it stops at the first match
    // If you needed to find all items with the same name, you'd just use .filter() wich returns all matches
    // The .find() method already returns an Option
    inventory_vector.iter().find(|item| item.name == item_name)
    // This will return an Option with Some(&Item { name: "...", quantity: ..., price: ... })

    // for item in &inventory_vector {
    //     if item.name == item_name {
    //         return Some(item);
    //     }
    // }
    // return None;
}

// Instead of returning a string slice (&str) in the Result: Result<f64, &str>,
// We are returning a String so that we own the data and have no lifetime issues
// With a string literal &str, Rust needs to know how long that reference is valid for
// Errors are also usually owned String values not borrowed &str
fn buy_item(item_name: &str, inventory_vector: &mut Vec<Item>, quantity: u32) -> Result<f64, String> {
    match find_item(item_name, inventory_vector) {
        // This is pattern matching
        // It says: "If the Option is Some, extract the value inside and call it item"
        // So now item is now the actual &mut Item not wrapped in Some() anymore
        Some(item) => {
            if item.quantity < quantity {
                Err("Not enough stock.".to_string())
            } else {
                // .iter_mut() gives us mutable references, so you can modify items
                // Since we are modifying the quantity, we need to use .iter_mut()
                // If we used .iter(), Rust would give you an error because you're trying to modify something through an immutable reference
                match inventory_vector.iter_mut().find(|item| item.name == item_name) {
                    Some(item) => {
                        item.quantity -= quantity;
                        // This is the total price
                        // This is what is being returned when the purchase is successful
                        // It wraps the total price inside the Ok variant of the Result enum
                        Ok(item.price * quantity as f64)
                    }
                    None => Err("Item not found.".to_string())
                }
            }
        }
        None => Err("Item not found.".to_string())
    }
}

fn main() {
    let mut items: Vec<Item> = vec![
        Item{name: "Cereal".to_string(), quantity: 17, price: 3.44},
        Item{name: "Tissues".to_string(), quantity: 22, price: 1.15},
        Item{name: "Water".to_string(), quantity: 0, price: 0.89},
        Item{name: "Sticker".to_string(), quantity: 73, price: 0.35},
        Item{name: "Snickers".to_string(), quantity: 44, price: 3.44},        
    ];

    // We are not using .unwrap() here to extract the result
    // .unwrap() will panic and crash if the Result is an Err
    // Since we are testing error cases, .unwrap() will crash
    // We should use match to handle both Ok() and Err() cases.
    match buy_item("Cereal", &mut items, 7) {
        Ok(total_price) => println!("Purchase successful! Total: ${}", total_price),
        Err(error) => println!("Error: {}", error),
    }

    match buy_item("Water", &mut items, 1) {
        Ok(total_price) => println!("Purchase successful! Total: ${}", total_price),
        Err(error) => println!("Error: {}", error),
    }
}
