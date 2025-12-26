use std::collections::HashMap;

fn main() {
    let input: &str = "The quick brown fox jumps over the lazy dog. The fox is quick.";

    let mut word_count: HashMap<String, u32> = HashMap::new();

    for word in input.split_whitespace() {
        let cleaned_word = word.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();

        // Here you do need the dereference because you're trying to add 1 to the number
        // The += operator doesn't automatically work on a reference so you have to dereference it
        // The .push() method is specifically designed to work on mutable references so you don't need to dereference
        *word_count.entry(cleaned_word).or_insert(0) += 1;
    }

    let word_vec: Vec<(String, u32)> = word_count.into_iter().collect();

    let mut word_count_groups: HashMap<String, Vec<(String, u32)>> = HashMap::new();

    for (word, count) in word_vec {
        if word.len() >=1 && word.len() <= 3 {
            // .entry() looks up the key in the HashMap. It returns an Entry object that represents the key already exists or it doesnt
            // .or_insert() says: If the key doesn't exist, insert a new empty vector and give me a mutable reference to it
            // If the key already exists, just give me a mutable reference to the existing vector
            // .push() - now you take that mutable reference to the vector and push the tuple to it
            word_count_groups.entry("Short".to_string()).or_insert(Vec::new()).push((word, count))
            // The first time you run this code with "Short", there is no "Short" key yet, so .or_insert creates a new empty Vec::new()
            // and immediately returns a reference to it, so you push into that new vector
            // The second time you run with "Short", the key already exists with items in it, so .or_insert() does not create a new vector
            // and just returns a reference to the existing one. Then you push another item to it. 
        } else if word.len() >=4 && word.len() <= 6 {
            word_count_groups.entry("Medium".to_string()).or_insert(Vec::new()).push((word, count))
        } else {
            // We do not need to dereference in this case since .push() already works on the mutable reference
            // When you do .or_insert(Vec::new()) it returns a mutable reference to the vector
            // A mutable reference means you have permission to modify that vector
            // So when you call .push() on that mutable reference, it automatically knows to work with it
            word_count_groups.entry("Long".to_string()).or_insert(Vec::new()).push((word, count))
        }
    }

    let groups_vec: Vec<String> = vec!["Short".to_string(), "Medium".to_string(), "Long".to_string()];

    // Instead of having groups_vec, I can just loop through an array of string slices: 
    // for group in &["Short", "Medium", "Long"]
    // Then dereference .get(*group)
    for group in groups_vec {
        // .get(&group) returns an Option, which means it could be either:
        // Some(words) - the key exists and here's the vector
        // None - the key doesn't exist in the HashMap
        // if let is a safe way to handle this - "If the result is Some(words), then extract the vector into a variable called words and run this code block"
        // "If it's none, just skip the block."
        // We need this because you can't loop through words that don't exist, that would crash the program. if let protects you by checking first
        // "If I can extract words from this Option, then do something with them. Otherwise, do nothing."
        if let Some(words) = word_count_groups.get(&group) {
            println!("[{}]", group);
            for (word, count) in words {
                println!("{}: {}", word, count)
            }
        }

        // Using if let is shorter than using a match statement to handle both the Some() and None cases
        // if let is cleaner and simpler way when you only care about the Some() case.
    }
}
