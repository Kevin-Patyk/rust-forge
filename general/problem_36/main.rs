// Rust practice problem: Clone on Write (`Cow`)

// Background

// `std::borrow::Cow` (Clone on Write) is a smart pointer that can hold either borrowed or owned data:

    // enum Cow<'a, B: ?Sized + ToOwned> {
    //     Borrowed(&'a B),
    //     Owned(<B as ToOwned>::Owned),
    // }

// When to use `Cow`: You have a function that sometimes needs to transform/allocate data and sometimes can just pass it through unchanged.
// Instead of always cloning (wasteful) or always borrowing (can't return modified data), `Cow` lets you do both through a single return type.

// The motivating pattern (we have seen code like this):

    // fn maybe_transform(s: &Series) -> Series {
    //     if needs_transform(s) {
    //         Ok(s.cast(&new_dtype)?)   // new allocation
    //     } else {
    //         Ok(s.clone())             // unnecessary clone!
    //     }
    // }

// That `s.clone()` in the else branch is wasteful - the data didn't change, so why copy it? `Cow` fixes this.

// The problem:
// You are building a data pipeline that normalizes column values before analysis.
// Some values need transformation, some don't. 
// You want to avoid cloning when the data is already in the right shape.

// Here is a function that either transforms data or passes it through unchanged:

// `&str` is a string slice - a reference to a sequence of UTF-8 text
// It does not allocate memory -> it just points to existing text somewhere else
// // Using `&str` instead of `String` makes a function more flexible
// `&str` works with string literals, slices of a `String`, slices of another `&str`, and borrowed `String`
fn normalize_field(value: &str) -> String {
    let trimmed = value.trim();
    // || is the logical OR operator, used in boolean expressions
    // It returns `true` if either side is true
    // Uses short-circuit evaluation, meaning if the first condition is true, the rest doesn't run
    if trimmed != value || value.chars().any(|c| c.is_uppercase()) {
        trimmed.to_lowercase()  // needs work — allocate
    } else {
        value.to_string()       // already clean — but we clone anyway
    }
}
// The `value.to_string()` in the else branch is wasteful. 
// The data is fine as is, but we are forced to allocate a new `String` because our return type demands an owned values.
// This is the exact same pattern as the Polars code where the else branch had `Ok(s.clone())`.

use std::borrow::Cow;

// Step 1: Change the return type to `Cow`
// `Cow<'_, str>` means "I am returning something that acts like a string - it might be a borrowed `&str` or an owned `String`, and the caller doesn't care which."
// The `'_` lifetime just means "same lifetime as the input." The compiler infers that the borrowed variant can't outlive `value`.
// For the lifetime, this means, if the result is borrowed, it cannot outlive the data it references.
fn normalize_field_cow(value: &str) -> Cow<'_, str> {
    let trimmed = value.trim();
    if trimmed != value || value.chars().any(|c| c.is_uppercase()) {
        // Step 3: Return `Cow::Owned` on the transform path
        Cow::Owned(trimmed.to_lowercase())
        // We were already allocating here, so nothing changes - we just wrap it in `Cow::Owned` so the types match.
    } else {
        // Step 2: Return `Cow::Borrowed` on the clean path:
        Cow::Borrowed(value) // zero cost - just hands back the reference
        // This is the whole point. Instead of `value.to_string()`, which allocates a new `String` on the heap, we just wrap the existing `&str` 
        // in `Cow::Borrowed`. No allocation, no cost.
    }
}

// Step 4: Using it - `Cow` implements `Deref`
// Here's the magic.
// `Cow<'_, str>` implements `Deref<Target = str>`, so you can pass it anywhere a `&str` is expected without doing anything special.
// As a note `Deref` is a trait that lets the compiler automatically convert one type into a reference to another type
// So, if you pass a `String` to a function that takes `&str`, the compiler sees the type mismatch, finds the `Deref` implementation,
// and inserts the conversion automatically.
// This is called deref coercion:
    // fn greet(name: &str) {
    //     println!("hello {name}");
    // }

    // let s = String::from("alice");
    // greet(&s);  // compiler converts &String -> &str via Deref

fn print_length(s: &str) {
    println!("{} is {} bytes", s, s.len());
}

fn main() {
    // Step 4: Using it - `Cow` implements `Deref`
    let clean = normalize_field("alice");       // Borrowed — no allocation
    let dirty = normalize_field("  Bob  ");     // Owned — allocated

    // Both work identically here. The caller doesn't know or care:
    print_length(&clean);
    print_length(&dirty);

    // Step 5: If you ever need to own it, you can
    // Sometimes downstream code needs a `String`. `Cow` has `.into_owned()` for that:
    let result = normalize_field("alice");

    // If it was already Owned, this is free (just unwraps it).
    // If it was borrowed, this clones - but only now, when you actually need it.
    // This is the "clone on write" part of the name - the clone is deferred until you actually need ownership and skipped entirely if you never do.
}

// Anytime you see this shape in your code:

    // if needs_change {
    //     Ok(transform(x))
    // } else {
    //     Ok(x.clone())       // <-- unnecessary allocation
    // }

// That's a `Cow`. The clone in the else branch is the smell.
// Replace the return type with `Cow`, used `Borrowed` for the pass through path, `Owned` for the transform path, and you've eliminated allocations for every input path that was clean.

// When we use `Cow`, our function will then return either `Cow::Borrowed` or `Cow::Owned` and the caller gets a single `Cow` type back
// If the downstream code just reads/borrows, you do nothing.
    // * `Deref` handles it and the `Cow` is transparent - you code doesn't know or care that a `Cow` is involved.
// If your downstream code needs ownership, you call `.into_owned()` at that point.
    // * And even then, `into_owned()` is smart - if the `Cow` happened to be `Owned` already, it just unwraps it with no clone.
    // * It only clones if it was borrowed.

// So the cost of cloning is pushed to the latest possible moment and only happens if both conditions are true:
    // * The data was borrowed AND someone downstream actually needs to own it.
