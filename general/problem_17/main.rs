#[allow(dead_code)]
struct Transaction { // Concrete type with known size at compile time due to the fields
    id: u32,
    amount: f64,
    transaction_type: TransactionType,
    timestamp: String,
}

#[derive(PartialEq)]
enum TransactionType {
    Deposit,
    Withdrawal,
    Transfer { to_account: u32}, // This variant holds data
}

#[allow(dead_code)]
struct BankAccount { // Concrete type with known size at compile time due to the fields
    account_number: u32,
    holder_name: String,
    balance: f64,
    transactions: Vec<Transaction>,
    next_transaction_id: u32,
}

impl BankAccount {
    fn new(account_number: u32, holder_name: String, initial_balance: f64) -> Self {
        Self {
            account_number,
            holder_name,
            balance: initial_balance,
            transactions: Vec::new(), // This will hold a vector of Transaction structs
            next_transaction_id: 0,
        }
    }

    fn deposit(&mut self, amount: f64) -> Result<u32, String> {
        if amount < 0.0 {
            return Err("Deposit amount cannot be less than 0.".to_string())
        }

        self.next_transaction_id += 1;

        self.balance += amount;

        let transaction = Transaction {
            id: self.next_transaction_id,
            amount,
            transaction_type: TransactionType::Deposit,
            timestamp: "01-01-1999".to_string(),
        };

        self.transactions.push(transaction);

        Ok(self.next_transaction_id)
    }

    fn withdraw(&mut self, amount: f64) -> Result<u32, String> {
        if amount > self.balance {
            // We do not need .clone() on amount since it implements the Copy trait
            // That means it is automatically copied when you use it 
            // Also, self.balance is a f64 and it implements Copy
            // That means we do not need to use .clone() on it either
            return Err(format!("Insufficient balance. Cannot withdraw {} from {}.", amount, self.balance))
            // Using format!() creates a String, so the return annotation is satisfied
        }

        self.next_transaction_id += 1;

        self.balance -= amount;

        let transaction = Transaction {
            id: self.next_transaction_id,
            amount,
            transaction_type: TransactionType::Withdrawal,
            timestamp: "02-02-2000".to_string(),
        };

        self.transactions.push(transaction);

        Ok(self.next_transaction_id)
    }

    fn transfer(&mut self, amount: f64, to_account: u32) -> Result<u32, String> {
        if amount > self.balance {
            return Err(format!("Insufficient balance. Cannot transfer {} from {}.", amount, self.balance))
        }

        self.next_transaction_id += 1;

        self.balance -= amount;

        let transaction = Transaction {
            id: self.next_transaction_id,
            amount,
            transaction_type: TransactionType::Transfer{ to_account },
            timestamp: "03-03-2001".to_string()
        };

        self.transactions.push(transaction);

        Ok(self.next_transaction_id)
    }

    fn get_balance(&self) -> f64 {
        self.balance
    }
    
    // .find() searches through the transactions and returns an Option
    // If it finds a matching transaction, it returns Some(&Transaction)
    // If it doesn't find a match, it returns None
    // We are returning a reference to a transaction that exists on self.transactions
    // We are not moving or copying the transaction out - we are just giving a pointer to where it lives
    fn get_transaction(&self, id: u32) -> Option<&Transaction> {
        self.transactions.iter().find(|transaction| transaction.id == id)
    }

    // This takes a reference to the account's transactions vector
    // It passes it to TransactionFilter::new()
    // Creates a new filter that starts with all transactions from this account
    // Returns that filter so it can be used for chaining
    fn filter_transactions(&self) -> TransactionFilter<'_>  { // This tells Rust to figure out the lifetime automatically based on the input parameters
        // TransactionFilter requires a lifetime parameter - when we return it, we must specify what that lifetime is

        // This is where the lifetime annotation a comes into play
        // &self.transactions is a reference that lives as long as self (BankAccount)
        // TransactionFilter accepts that reference
        // The a is now tied to the lifetime of the BankAccount
        // Result: The TransactionFilter cannot outlive the BankAccount it borrowed from
        TransactionFilter::new(&self.transactions) // if we put a semicolon here, it turns it into a statement which returns nothing, but we need it to be an expression
        // As long as the BankAccount exists, the TransactionFilter can safely reference it's transactions 
        // If you try to use the filter after the account is gone, Rust's compiler catches it at compile time
    }
}

// Here a is a lifetime annotation - it is telling Rust how long these references will live
// We need it since we have a vector of references to Transactions
// The a says: "All the &Transaction references in this vector must live at least as long as the a lifetime"
// It prevents us from creating a TransactionFilter where the references point to transactions that have been deallocated
// "The TransactionFilter (and it's vector of references) can only live as long as the Transactions it points to."

// The actual Transactions live in BankAccount.transactions (the owner)
// The TransactionFilter.filtered just holds references to those transactions (borrowing)
// a ensures: "You can't use TransactionFilter after the original transactions are gone"
// The lifetime a ties TransactionFilter's lifetime to the BankAccount's lifetime, ensuring references always point to valid data
struct TransactionFilter<'a> {
    filtered: Vec<&'a Transaction>,
}

// Method chaining is when call multiple methods in sequence on the same object: ob.method1().method2().method3()
// There are 3 self types in Rust:
// &self - immutable borrow 
// &mut self - mutable borrow
// self - takes ownership -> you consume the object and return a new/modified version
// self enables chaining
// The pattern for chainable methods is:
// 1. Consume self (take ownership)
// 2. Do some transformation on self's data
// 3. Return a new Self with the transformed data
// This works because each method takes ownership of the previous result 
// Each method returns Self, making it available for the next method

// Key rules: Methods that chain must take self, they usually return Self (so you can keep chaining)
// the final method in the chain returns something else
// A common pattern is a builder pattern

// The cycle is: Consume the incoming object - take ownership with self
// Transform/modify its data 
// Return Self (a new instance) - create and return a new instance with the transformed data
// The new instance can be consumed by the next method in the chain
impl<'a> TransactionFilter<'a> {
    // &Vec<Transaction> is a single reference to the entire vector - you can only access the vector as a whole 
    // You can't easily filter, transform, or work with individual transactions
    // Its like having a reference to the box, not the items inside
    // For our method chaining and filtering to work, we need pointers to the inner elements of the vector not to the whole vector
    fn new(transactions: &'a Vec<Transaction>) -> Self {
        // Here we are iterating through the transactions and collecting references to each individual transaction
        // This creates an iterator that yields &Transaction and collect gathers the references into a Vector
        let filtered: Vec<&'a Transaction> = transactions.iter().collect(); // since .iter() does not take ownership, it yields references for each item
        // We turn this into a vector of references to individual transactions 
        // You can filter, map, and manipulate individual references
        // It's like having a list of pointer's to each item

        // This creates and returns the new TransactionFilter with all transactions initially included
        Self { filtered }
    }

    fn by_type(self, transaction_type: TransactionType) -> Self {
        // Takes ownership of self (consumes Transaction Filter) and returns a new filtered version
        // This allows method chaining

        let filtered: Vec<&'a Transaction> = self.filtered
            // We are using .into_iter() instead of .iter() because we are consuming self, not borrowing it
            .into_iter()
            // .filter() keeps only transactions that match our desired type
            // The closure receives each &Transaction reference
            .filter(|transaction| {
                // We match on a tuple of 2 references to compare enum variants
                // the first is the type from the transaction we are checking 
                // and the second is the type we want to filter by (the parameter)
                match (&transaction.transaction_type, &transaction_type) {
                    // If both are Deposit, keep this transaction
                    (TransactionType::Deposit, TransactionType::Deposit) => true,
                    // If both are Withdrawal, keep this transaction
                    (TransactionType::Withdrawal, TransactionType::Withdrawal) => true,
                    // If both are Transfer, keep this transaction
                    // The { .. } ignores the to_account data - we only care that both are transfer variants
                    (TransactionType::Transfer { .. }, TransactionType::Transfer { .. }) => true,
                    // If the variants don't match, filter it out
                    _ => false,
                }
            })
            .collect();
        
        // Return a new TransactionFilter with the filtered results
        // This new Self can be used for further chaining
        Self { filtered }
    }

    // fn above_amount_two(self, min_amount: f64) -> Result<Self, String> {
    //     if min_amount < 0.0 {
    //         return Err("Minimum amount cannot be less than 0.".to_string())
    //     }

    //     let filtered: Vec<&'a Transaction> = self.filtered
    //         .into_iter()
    //         .filter(|transaction| transaction.amount > min_amount)
    //         .collect();

    //     Ok(Self { filtered })
    // }

    fn above_amount(self, min_amount: f64) -> Self {
        let filtered: Vec<&'a Transaction> = self.filtered
            .into_iter()
            .filter(|transaction| transaction.amount >= min_amount)
            .collect();

        Self { filtered }
    }

    fn below_amount(self, max_amount: f64) -> Self {
        let filtered: Vec<&'a Transaction> = self.filtered
            .into_iter()
            .filter(|transaction| transaction.amount <= max_amount)
            .collect();

        Self { filtered }
    }

    fn collect(self) -> Vec<&'a Transaction> {
        self.filtered
    }
}

fn main() {
    // This creates a new bank account with the input parameters going into the fields
    // and initializing the other 2 fields (transactions and next_transaction_id) with
    // default values that we set inside the new() method
    // new() is an associated function (also called a constructor) since it does not 
    // require an instance (self) to work on - it refers to the type itself (Self)
    let mut account = BankAccount::new(1001, "Alice".to_string(), 1000.0);

    // Process various transactions
    println!("=== Processing Transactions ===");
    
    match account.deposit(200.0) {
        Ok(id) => println!("Deposit successful! Transaction ID: {}", id),
        Err(e) => println!("Error: {}", e),
    }

    match account.deposit(150.0) {
        Ok(id) => println!("Deposit successful! Transaction ID: {}", id),
        Err(e) => println!("Error: {}", e),
    }

    match account.withdraw(50.0) {
        Ok(id) => println!("Withdrawal successful! Transaction ID: {}", id),
        Err(e) => println!("Error: {}", e),
    }

    match account.transfer(100.0, 1002) {
        Ok(id) => println!("Transfer successful! Transaction ID: {}", id),
        Err(e) => println!("Error: {}", e),
    }

    match account.deposit(75.0) {
        Ok(id) => println!("Deposit successful! Transaction ID: {}", id),
        Err(e) => println!("Error: {}", e),
    }

    match account.withdraw(5000.0) {
        Ok(id) => println!("Withdrawal successful! Transaction ID: {}", id),
        Err(e) => println!("Error: {}", e),  // Should fail - insufficient funds
    }

    println!("\nCurrent balance: ${}\n", account.get_balance());

    // Test method chaining filters
    println!("=== Testing Filters ===\n");

    // All deposits
    println!("[All Deposits]");
    // This takes a reference to the account's transactions and creates a TransactionFilter
    // with a lifetime tied to the account
    // The TransactionFilter (and its references) cannot outlive the account
    // We then chain methods to filter by deposit type and collect the results
    let deposits = account.filter_transactions() // Creates TransactionFilter with all transactions
        .by_type(TransactionType::Deposit) // filters only to keep deposits
        .collect(); // returns final Vec<&Transaction>
    
    // Here, we are iterating over the Vec of reference provided by the .collect() final method
    // We are using &deposits because we want to borrow the vector instead of consuming it
    // deposits is Vec<&Transaction> - a vector of references to transactions
    // The & is about ownership/borrowing the vector, not about the fact that the vector contains references
    for t in &deposits {
        println!("ID: {}, Amount: ${}", t.id, t.amount);
    }

    // All withdrawals
    println!("\n[All Withdrawals]");
    let withdrawals = account.filter_transactions()
        .by_type(TransactionType::Withdrawal)
        .collect();
    
    for t in &withdrawals {
        println!("ID: {}, Amount: ${}", t.id, t.amount);
    }

    // All transfers
    println!("\n[All Transfers]");
    let transfers = account.filter_transactions()
        .by_type(TransactionType::Transfer { to_account: 0 })
        .collect();
    
    for t in &transfers {
        match &t.transaction_type {
            TransactionType::Transfer { to_account } => {
                println!("ID: {}, Amount: ${}, To: {}", t.id, t.amount, to_account);
            }
            _ => {}
        }
    }

    // Transactions above $100
    println!("\n[Transactions Above $100]");
    let large = account.filter_transactions()
        .above_amount(100.0)
        .collect();
    
    for t in &large {
        println!("ID: {}, Amount: ${}", t.id, t.amount);
    }

    // Deposits between $50 and $200
    println!("\n[Deposits Between $50-$200]");
    let mid_deposits = account.filter_transactions()
        .by_type(TransactionType::Deposit)
        .above_amount(50.0)
        .below_amount(200.0)
        .collect();
    
    for t in &mid_deposits {
        println!("ID: {}, Amount: ${}", t.id, t.amount);
    }

    // Calculate statistics
    println!("\n=== Statistics ===");

    let total_deposits: f64 = account.filter_transactions()
        .by_type(TransactionType::Deposit)
        .collect()
        .iter()
        .map(|t| t.amount)
        .sum();
    println!("Total deposits: ${}", total_deposits);

    let total_withdrawals: f64 = account.filter_transactions()
        .by_type(TransactionType::Withdrawal)
        .collect()
        .iter()
        .map(|t| t.amount)
        .sum();
    println!("Total withdrawals: ${}", total_withdrawals);

    let transfer_count = account.filter_transactions()
        .by_type(TransactionType::Transfer { to_account: 0 })
        .collect()
        .len();
    println!("Number of transfers: {}", transfer_count);

    let all = account.filter_transactions().collect();
    if let Some(largest) = all.iter().max_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap()) {
        println!("Largest transaction: ${}", largest.amount);
    }

    // Test get_transaction
    println!("\n=== Get Specific Transaction ===");
    match account.get_transaction(1) {
        Some(t) => println!("Found ID 1: ${}", t.amount),
        None => println!("Not found"),
    }

    match account.get_transaction(999) {
        Some(t) => println!("Found ID 999: ${}", t.amount),
        None => println!("Not found"),
    }
}
