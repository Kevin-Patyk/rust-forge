// A trait defines a contract that multiple types can implement
// In order for a type to have this trait, it must implement both of these signatures
// The types that implement these traits can have different implementations in the underlying function 
// but will still be the same trait
trait MenuItem {
    fn get_price(&self) -> f64;
    fn get_name(&self) -> String;
}

// This is an attribute that automatically implements the PartialEq trait for your enum
// PartialEq is what allows you to do the == operator to compare values
// Without it, you would get an error saying you can't compare enum variants
#[derive(PartialEq)]
enum OrderStatus {
    Pending,
    Preparing,
    Ready,
    Completed,
}
// enums are a way of saying a value is one of a possible set of values
// with enums, we usually use match statements to handle the different cases safely

struct Burger {
    name: String,
    price: f64,
}

// Here we are implementing a trait for a type
// We are using &self here to borrow the struct instead of taking ownership
// It does not need to be mutable since we only need to read it
// We want to be able to call this method multiple times on the same item without taking ownership
// self also refers to the instance of the type we are calling the method on
impl MenuItem for Burger {
    fn get_price(&self) -> f64 {
        self.price
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
}

struct Drink {
    name: String,
    price: f64,
}

// Since we are implementing this trait for Burger and Drink,
// you can store them together in a Vec<Box<dyn MenuItem>>
// The dyn MenuItem part means "any type that implements the MenuItem trait."
impl MenuItem for Drink {
    fn get_price(&self) -> f64 {
        self.price
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
}

// This struct will take a vector of items to order, customer name, and the OrderStatus enum
#[allow(dead_code)]
struct Order {
    items: Vec<Box<dyn MenuItem>>,
    customer_name: String,
    status: OrderStatus,
}

// This function is taking an item (Burger or Drink) that implements the MenuItem trait
// and is adding it to the Vec<Box<dyn MenuItem>> vector in Order.items
// But only if the order status criteria is met
fn add_item_to_order(order: &mut Order, item: Box<dyn MenuItem>) -> Result<String, String> {
    if order.status == OrderStatus::Ready || order.status == OrderStatus::Completed {
        Err("Cannot add items to a completed or ready order.".to_string())
    } else {
        let item_name = item.get_name();
        order.items.push(item);
        Ok(format!("{} added to order.", item_name))
    }
}

// Here, we are using &Order because we don't need to take ownership - we are just reading the prices
// That way we can call it multiple times on the same order
fn calculate_total(order: &Order) -> f64 {
    // .iter() loops through the items - it creates an iterator that goes through each item one by one
    // .map() transforms each item into its price
    // .sum() adds them all up
    order.items.iter().map(|item| item.get_price()).sum()
    // This method is more functional - instead of manually looping and updating a variable, you're chaining methods together to
    // transform and combine data

    // This is an alternate way

    // let mut total: f64 = 0.0;
    // for item in order.items {
    //     total += item.get_price();
    // }
    // total
}

// Since Order.status is an enum, we are using a match statement to find out the order status
// Then we are ensuring a linear order status by only updating the status if the new status
// is the next in the sequence
fn update_order_status(order: &mut Order, new_status: OrderStatus) -> Result<String, String> {
    match order.status {
        OrderStatus::Pending => {
            if new_status == OrderStatus::Preparing {
                order.status = new_status;
                Ok("Order status has been changed from Pending to Preparing.".to_string())
            } else {
                Err("Invalid attempt to change order status.".to_string())
            }
        }
        OrderStatus::Preparing => {
            if new_status == OrderStatus::Ready {
                order.status = new_status;
                Ok("Order status has been changed from Preparing to Ready.".to_string())
            } else {
                Err("Invalid attempt to change order status.".to_string())         
            }
        }
        OrderStatus::Ready => {
            if new_status == OrderStatus::Completed {
                order.status = new_status;
                Ok("Order status has been changed from Ready to Completed.".to_string())
            } else {
                Err("Invalid attempt to change order status.".to_string())
            }
        }
        OrderStatus::Completed => {
            Err("The order has already been completed. Cannot further update the order status.".to_string())
        }
    }
}

fn main() {
    let mut first_order = Order {
        items: Vec::new(),
        customer_name: "Alice".to_string(),
        status: OrderStatus::Pending,
    };

    // Trait objects need box because the compiler doesn't know the size of the concrete type at compile time
    // Box allocates memory on the heap and gives you a pointer to it
    // That way, Rust can say "I don't know if this is a Burger or Drink, but I know it's a pointer, so I know the size."
    // Without Box, the compiler can't figure out how much memory to allocate.
    match add_item_to_order(&mut first_order, Box::new(Burger{name: "Classic Burger".to_string(), price: 12.99})) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match add_item_to_order(&mut first_order, Box::new(Drink{name: "Cola".to_string(), price: 3.50})) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match update_order_status(&mut first_order, OrderStatus::Preparing) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match update_order_status(&mut first_order, OrderStatus::Ready) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match update_order_status(&mut first_order, OrderStatus::Completed) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match add_item_to_order(&mut first_order, Box::new(Burger{name: "Sprite".to_string(), price: 3.50})) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    let mut second_order = Order {
        items: Vec::new(),
        customer_name: "Bob".to_string(),
        status: OrderStatus::Pending
    };

    match add_item_to_order(&mut second_order, Box::new(Burger{name: "Spicy Burger".to_string(), price: 14.99})) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match add_item_to_order(&mut second_order, Box::new(Drink{name: "Lemonade".to_string(), price: 2.99})) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    let second_order_total = calculate_total(&second_order);
    println!("Second order total: {}", second_order_total);

    let mut third_order = Order {
        items: Vec::new(),
        customer_name: "Charlie".to_string(),
        status: OrderStatus::Pending,
    };

    match update_order_status(&mut third_order, OrderStatus::Completed) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }
}

