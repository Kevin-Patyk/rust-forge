use std::collections::HashMap;

fn main() {
    let input: &str = "The quick brown fox jumps over the lazy dog. The fox is quick.";

    let mut word_count: HashMap<String, u32> = HashMap::new();

    // .split_whitespace() is a string method that splits a string into an iterator of substrings, using anyway whitespace characters
    for word in input.split_whitespace() {
        // This converts the string into an iterator of individual characters
        // Then it filters out anything that is not alphanumeric
        // It then gathers all those filtered characters back into a single string
        // Then it converts that string to lowercase
        let cleaned = word.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();

        // This looks up a key in the hashmap
        // If the key does not exist, it creates a key and gives it a value of 0
        // If the key exists, it returns a reference to the existing value so it can be used or modified
        // Remember: HashMaps in Rust enforce unique keys. Each key can only appear once in a HashMap
        *word_count.entry(cleaned).or_insert(0) += 1;
        // .or_insert() always give back a mutable reference to the value
        // In order to be able to work with the actual value, it needs to be dereferenced 
    }

    // We use .into_iter() here because we want to consume the HashMap and get ownership of the items
    // We want to move the key-value pairs out of the HashMap and into the Vec
    // We don't need the HashMap anymore after this point, so it makes sense to consume it
    let mut vec: Vec<(String, u32)> = word_count.into_iter().collect();

    // b.1 is the second element of tuple b
    // a.1 is the second element of tuple a
    // .cmp() compares them (returns ordering)
    // b.1 comes before a.1 - this is reverse sort order
    // Normally a.cmp(b) sorts ascending (smallest first), but b.cmp(a) sorts descending (largest first)
    vec.sort_by(|a, b| b.1.cmp(&a.1));

    for (word, count) in vec {
        println!("{}: {}", word, count)
    }
}
