use std::collections::HashMap;

enum FrequencyCategory {
    Rare,
    Uncommon,
    Common,
}

fn main() {
    let input: &str = "The quick brown fox jumps over the lazy dog. The fox is quick.";

    let mut word_count: HashMap<String, u32> = HashMap::new();

    for word in input.split_whitespace() {
        
        let cleaned = word.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();
    
        *word_count.entry(cleaned).or_insert(0) += 1;
    }

    // A tuple in Rust is a collection of values of different types grouped together into a single value
    // The key difference from a Vec is that a tuple has a fixed length and can hold different types
    // Whereas a Vec has a variable length but all elements must be the same type
    // Tuples are useful when you want to group related pieces of different data together
    let mut vec: Vec<(String, u32)> = word_count.into_iter().collect();

    vec.sort_by(|a , b| b.1.cmp(&a.1));

    let mut total_vec: Vec<(String, u32, FrequencyCategory)> = Vec::new();

    for (word, count) in vec {
        if count == 1 {
            total_vec.push((word, count, FrequencyCategory::Rare));
        } else if count >=2 && count <=3 {
            total_vec.push((word, count, FrequencyCategory::Uncommon));
        } else {
            total_vec.push((word, count, FrequencyCategory::Common));
        }
    }

    // We cannot print category directly since the enum doesn't derive Debug or Display
    // So we will be hardcoding it
    for (word, count, category) in total_vec {
        match category {
            FrequencyCategory::Rare => println!("[RARE] {}: {}", word, count),
            FrequencyCategory::Uncommon => println!("[UNCOMMON] {}: {}", word, count),
            FrequencyCategory::Common => println!("[COMMON] {}: {}", word, count),
        }
    }
}
