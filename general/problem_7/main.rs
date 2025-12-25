use std::collections::HashMap;

enum WordStats {
    Frequent(String, u32),
    Rare(String, u32),
}

fn main() {
    let input: &str = "The quick brown fox jumps over the lazy dog. The fox is quick.";

    let mut word_count: HashMap<String, u32> = HashMap::new();

    for word in input.split_whitespace() {
        let cleaned_word = word.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();

        *word_count.entry(cleaned_word).or_insert(0) += 1;
    }

    let mut words_vec: Vec<(String, u32)> = word_count.into_iter().collect();

    // .sort_by() is a method that sorts a vector using a custom comparison function
    // |a, b| is a closure 
    // by putting b first, you're sorting in reverse order (descending)
    // if a came first, it would sort in ascending (lowest counts first)
    // The whole thing says: "Sort this vector by comparing the counts (second element) of each tuple with the largest counts first"
    words_vec.sort_by(|a ,b| b.1.cmp(&a.1));

    let mut frequent_vec: Vec<WordStats> = Vec::new();
    let mut rare_vec: Vec<WordStats> = Vec::new();

    for (word, count) in words_vec {
        match count {
            // This is called a match arm with a guard condition
            // n is the pattern - it captures the value being matched (in this case count)
            // if n >= 3 is the guard - it's an additional condition that must be true for this arm to run
            // So together it means: "If the value matches the pattern n (which everything does) AND if n >= 3 is true, then run this arm
            // If n is less than 3, this arm won't match and Rust will move on to check the next arm
            // The pattern "n" matches on anything and then the guard adds the extra condition that must be true for that arm to execute
            n if n >= 3 => {
                frequent_vec.push(WordStats::Frequent(word, count))
            }
            // This is a wildcard pattern
            // It means "match anything" or "everything else"
            // In a match statement, _ is a catch-all arm that will match any value that wasn't caught by the previous arms
            _ => {
                rare_vec.push(WordStats::Rare(word, count))
            }
        }
    }

    println!("[FREQUENT_WORDS]");
    for item in frequent_vec {
        match item {
            WordStats::Frequent(word, count) => println!("{}: {}", word, count),
            // The curly brackets here are an empty code block
            // They mean "do nothing" or "run no code"
            _ => {},
        }
    }

    println!("[RARE_WORDS]");
    for item in rare_vec {
        match item {
            WordStats::Rare(word, count) => println!("{}: {}", word, count),
            _ => {},
        }
    }
}
