#[derive(Copy, Debug, PartialEq, Clone)]
#[allow(dead_code)]
enum BankError {
    InsufficientFunds,
    InvalidAmount,
    AccountNotFound,
}

// The <T> means "this struct can hold any type T for the balance field"
// When we create instances of the struct, we will specify what type T is
// When you implement methods on a generic struct, you also use <T>
#[allow(dead_code)]
struct Account<T> {
    account_number: u32,
    holder_name: String,
    balance: T,
}

// This is a generic struct that holds a vector of accounts, where all accounts use the same T for their balance
// All accounts in a single bank must use the same type
// You cant mix Account<f64> with Account<i32>
// So if we instantiate a Bank struct like Bank<f64> = Bank::new(), we can only add Account<f64> to it
// This enforces type consistency - so we don't accidentally mix numeric types in the same bank system
struct Bank<T> {
    accounts: Vec<Account<T>>,
}

// You need <T> on both Account and Bank since they are connected
// Account<T> - each individual accounts holds a balance of type T
// Bank<T> - A bank holds a vector of accounts: Vec<Account<T>>
// The Bank's generic type T must match all accounts it contains
// If you only put <T> on Account and not Bank, the compiler wouldn't know what type the Bank's vector should hold and you'd lost type safety 
// In other words, Bank<T> determines what type T is and that same T is used for all the Account<T> structs inside it

// This says: "I'm implementing methods for Bank with any generic type T."
// The return type Bank<T> means "return a Bank struct with the same type T."
// So when someone calls Bank::<f64>::new(), it returns Bank<f64>
// The <T> in this impl block must match the struct definition
// Since Bank<T> is generic, the impl must also be generic with the same type parameter
impl<T: std::ops::AddAssign + std::ops::SubAssign + std::cmp::PartialOrd + Copy> Bank<T> {
    // The above are called trait bounds - they tell the compiler what capabilities a generic type must have
    // They're constraints on generic types that say: "I can be any type, but it must support these specific operations/traits."
    // Our above example says: "I am implementing Bank for any type T as long as T can do +=, -=, comparisons with < and can be copied"
    // Without trait bounds, Rust wouldn't knmow if T supports those operations 
    // Trait bounds guarantee that whatever types you use will have those capabilities
    fn new() -> Bank<T> {
        Bank {
            accounts: Vec::new(),
        }
    }

    fn create_account(&mut self, account_number: u32, holder_name: String, balance: T) -> Result<String, String> {
        if holder_name.is_empty() {
            Err("Holder name cannot be empty.".to_string())
        } else {
            self.accounts.push(
                Account {
                    account_number,
                    // We need to clone here since we are moving holder_name into Account
                    // If we were to try and use it later in the format string, it would fail
                    holder_name: holder_name.clone(),
                    balance,
                }
            );

            Ok(format!("Successfully created account for: {}", holder_name))
        }
    }

    fn deposit(&mut self, account_number: u32, amount: T) -> Result<(), BankError> {
        // This line searches through the bank's account vector to find an account (struct) with a matching account number
        // iter_mut() lets you loop through accounts and modify them
        // .find() stops at the first match and returns an Option
        // Then we match on that Option to handle if the account was found or not
        match self.accounts.iter_mut().find(|account| account.account_number == account_number) {
            Some(account) => {
                account.balance += amount;
                // The unit type () inside of Ok() means "this operation succeeded but has no return value"
                // This is used when the function succeeds but doesn't need to return any meaningful data
                // It is a good side effect for operations like deposits and deletions
                Ok(())
            }
            None => Err(BankError::AccountNotFound)
        }
    }

    fn withdrawal(&mut self, account_number: u32, amount: T) -> Result<T, BankError> {
        match self.accounts.iter_mut().find(|account| account.account_number == account_number) {
            Some(account) => {
                if account.balance < amount {
                    Err(BankError::InsufficientFunds)
                } else {
                    account.balance -= amount;
                    Ok(account.balance)
                }
            }
            None => Err(BankError::AccountNotFound)
        }
    }
}

fn main() {
    
    // Creating a bank account with a type of f64
    let mut bank_account_f64: Bank<f64> = Bank::new();
    // Here, we are saying: "Create a Bank where T is f64."
    // Then Bank<f64> has a field accounts: Vec<Account<f64>> - All accounts in that vector must be Account<f64>
    // Bank<T> controls what type all the accounts will be
    // It's the parent type that constrains everything inside

    match bank_account_f64.create_account(123, "one".to_string(), 100.00) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match bank_account_f64.create_account(456, "two".to_string(), 200.00) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match bank_account_f64.create_account(789, "three".to_string(), 300.00) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match bank_account_f64.create_account(000, "".to_string(), 400.00) {
        Ok(message) => println!("{}", message),
        Err(error) => println!("{}", error),
    }

    match bank_account_f64.deposit(123, 100.00) {
        Ok(()) => println!("Deposit successful!"),
        Err(error) => println!("Error: {:?}", error),
    }

    // We can also use if let if we only care about the success case
    // if let Ok(()) = bank_account_f64.deposit() {
    // ... 
    // }

    // In this line of code, we are just checking of our deposit was successful 
    // Here, we only care about the success case, which is why we are using if let
    if let Some(account) = bank_account_f64.accounts.iter().find(|account| account.account_number == 123) {
        println!("New balance: {}", account.balance);
    }

    match bank_account_f64.deposit(111, 200.00) {
        // Here we are saying: "If the deposit returned Ok(()), print the success message."
        // It's matching on a specific pattern of Ok containing the unit type ().
        Ok(()) => println!("Deposit successful!"),
        Err(error) => println!("Error: {:?}", error),
    }

    match bank_account_f64.withdrawal(456, 100.00) {
        Ok(balance) => println!("Withdrawal successful. Remaining balance: {}", balance),
        Err(error) => println!("Error: {:?}", error),
    }

    match bank_account_f64.withdrawal(789, 400.00) {
        // When we do Ok(something), Rust checks if the result is the Ok variant
        // Unwraps it and extracts whatever is inside
        // Binds that value to the variable name something
        // Then you can use something in the code block
        Ok(balance) => println!("Withdrawal successful. Remaining balance: {}", balance),
        Err(error) => println!("Error: {:?}", error),
    }

    match bank_account_f64.withdrawal(111, 200.00) {
        Ok(balance) => println!("Withdrawal successful. Remaining balance: {}", balance),
        Err(error) => println!("Error: {:?}", error),
    }

    // Creating a bank account with a type of i32
    let mut _bank_account_i32: Bank<i32> = Bank::new();
    // This can also be done with
    // let mut bank_account_i32 = Bank::<i32>::new();
    // But the first is more concise and idiomatic

    // Generally, if we have a generic struct, then our impl must also be generic
    // The <T> in the impl tells Rust: "These methods work for my struct with type <T>"
    // If you forgot the <T> and just wrote impl Struct, the compiler would error

    // You can implement methods only for a specific type doing something like:
        // impl MyStruct<String> {...}
    // This implements methods only when T is a string, but typically you want generic impl blocks
    
    // General rule: Generic struct = Generic impl
}
