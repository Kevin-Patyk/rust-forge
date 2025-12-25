// A struct is a custom data type that groups related data together
// Its like a blueprint for creating an object that can hold multiple pieces of information
// Structs can implement methods on them 
struct CharacterCounter;
struct WordCounter;
struct AverageWordLength;
// These structs are empty with no fields - they don't hold any data
// We are purely using them as a way to implement different versions of the TextAnalyzer trait 
// This is a common pattern in Rust when you want to group related behavior together without needing to store state

// A trait is a collection of methods that a type can implement
// Its like a contract or interface that says: if you implement this trait, you must have these methods with these signatures
// This allows different types to share the same behavior, enabling you to write code that works with any type that implements
// that particular trait
// This is how Rust achieves polymorphism 
trait TextAnalyzer {
    // We are using &self - this means the method borrows a reference to the struct instance instead of taking ownership
    // This lets you call the method multiple times on the same instance without taking ownership/losing it
    // In most cases, we use &self because you need to read data from the struct
    // Using self is rare for trait methods because there aren't many situations where you would consume it entirely
    fn analyze(&self, text: &str) -> String;
}

impl TextAnalyzer for CharacterCounter {
    fn analyze(&self, text: &str) -> String {
        let count: usize = text.len();
        // We are using format!() here since it returns a string while println!() prints to the console and returns nothing
        // Since the analyze() method has a return type of -> String, we need to return an actual string value
        format!("Character count: {}", count)
    }
}

impl TextAnalyzer for WordCounter {
    fn analyze(&self, text: &str) -> String {
        // Here we are using the functional style with .map() and .collect() since it is generally preferred 
        // because its more idiomatic and since this is not complex logic
        let cleaned_words: Vec<String> = text
        .split_whitespace()
        // We using .map() here because we want to take each word in text and transform each individual word
        // If we used .filter() after, we would filter out entire words based on the condition
        .map(|word| word.chars().filter(|c| c.is_alphanumeric()).collect::<String>())
        .collect();

        // if we used a for loop, we would have to created a mutable empty vector and push the cleaned word to it
        let word_count: usize = cleaned_words.len();
        format!("Word count: {}", word_count)
    }
}

impl TextAnalyzer for AverageWordLength {
    fn analyze(&self, text: &str) -> String {
        let cleaned_words: Vec<String> = text
        .split_whitespace()
        .map(|word| word.chars().filter(|c| c.is_alphanumeric()).collect::<String>())
        .collect();
        
        // .sum() is special - it doesn't need .collect() because it's a consuming adapter that directly produces a final value
        // .sum() takes an iterator and immediately adds up all the values, returning a single number
        // Compare this to .map() which returns a new iterator of transformed items, so you need .collect() to turn that iterator into
        // an actual collection
        let total_characters: usize = cleaned_words
        .iter() // We dont use .into_iter() here since it consumes cleaned_words then we are calling .len() on it
        // if cleaned_words is consumed, then .len() won't work
        .map(|w| w.len())
        .sum();

        let average = (total_characters as f64 / cleaned_words.len() as f64).floor() as usize;
        format!("Average word length: {}", average)
    }
}

fn main() {

    let input: &str = "The quick brown fox jumps over the lazy dog. The fox is quick.";

    // dyn TextAnalyzer means "any type that implements the TextAnalyzer trait."
    // Box<dyn TextAnalyzer> is a trait object - I don't care what type this is as long as it implements TextAnalyzer
    // Box allocates memory on the heap and gives you a pointer to it
    // Trait objects need to be behind a pointer like Box because the compiler doesn't know the size of the concrete type at compile time
    // Box lets Rust say: "I will figure out what type this actually is at runtime."
    // This is a form of dynamic dispatch - the specific function or method being called is determined at runtime rather than compile time
    let analyzers: Vec<Box<dyn TextAnalyzer>> = vec![
        // Box::new() wraps each concrete type in a Box so it becomes a Box<dyn TextAnalyzer>
        Box::new(CharacterCounter),
        Box::new(WordCounter),
        Box::new(AverageWordLength),
        // Trait objects always need to be behind some kind of pointer
        // Box is the most common, other options include Rc and Arc
    ];
    
    for method in analyzers {
        // Rust will automatically dereference the Box for us, so we don't need to do anything special
        let result = method.analyze(input);
        println!("{}", result)
    }

}
