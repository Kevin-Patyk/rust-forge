// This will allow dead code across the entire project file
#![allow(dead_code)]
use std::fmt;

// enums, short for enumeration, let you define a type that can be one of several different possible values, called a variant
// Think of it like giving a name to set of choices
// Enums are meant to represent choices clearly - when a value can be one of a fixed set of choices
// To group different related possibilities together
// When each choice may need to store different data
// Take advantage of powerful pattern matching
enum Measurement {
    Grams(f64),
    Milliliters(f64),
    Cups(f64),
    Tablespoons(f64)
}

struct Ingredient {
    name: String,
    amount: Measurement,
}

struct Recipe {
    title: String,
    servings: u32,
    ingredients: Vec<Ingredient>, // Vector of Ingredient structs 
    instructions: Vec<String>, 
}

#[derive(PartialEq)]
enum RecipeCategory {
    Breakfast,
    Lunch,
    Dinner,
    Dessert,
    Snack
}

struct CategorizedRecipe {
    recipe: Recipe,
    category: RecipeCategory,
    prep_time_minutes: u32,
}

// fmt::Formatter is a buffer/writer that Rust uses to build formatted strings
// When you implement Display or Debug, Rust gives you this formatter to write to
// Think of it like a blank canvas or string builder - its where you write your formatted output, Rust manages it internally, you write it piece by piece using write!() or writeln!()
// the &mut is important because we are mutating the formatter - each write!() adds more text to its internal buffer
// Think of Formatter like a StringBuilder -> you keep appending to it and, at the end, you get the complete string
// So when you write write!(f, ...) you are appending formatted text to the formatter's internal buffer, it accumulates all the writes, and Rust extracts the final string when you're done

// Here, we are implementing the Display trait for the Measurement enum
// Implementing the Display trait allows you to define HOW your custom type should be formatted
// Display works with the {} placeholder in format strings like println!("{}", something) and format!()
// Without Display, you'd get a compiler error because it doesn't implement the Display trait
impl fmt::Display for Measurement {
    // This is the required signature
    // We are borrowing the value we are formatting (&self)
    // f is the formatter you write to 
    // fmt::Result returns either Ok or an error
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Pattern matching on the enum variants to handle each case
        match self {
            // self is a Measurement enum - it could be any of the variants
            // This checks if self is the Grams variant and extracts/destructures the f64 value and binds it to the variable amount
            // Then we can use amount in the match arm - it is the actual number store inside the enum
            Measurement::Grams(amount) => write!(f, "{}g", amount),
            // The write!() macro writes formatted text to the formatter
            // The first argument is always f - the formatter
            // Then a format string like println!
            // Then any values to interpolate
            // It automatically returns fmt::Result, so you don't need to write return or Ok(())
            Measurement::Milliliters(amount) => write!(f, "{}ml", amount),
            // The macro implicitly returns - each write!() call returns fmt::Result and since its
            // the last expression in each match arm, it gets returned
            Measurement::Cups(amount) => write!(f, "{} cups", amount),
            // The write!() macro is similar to format!() or println!(), but instead of creating a String
            // or printing to console, it writes to the formatter f 
            Measurement::Tablespoons(amount) => write!(f, "{} tbsp", amount),
            // match expressions have implicit returns, just like most things in Rust
            // In Rust, almost everything is an expression that returns a value

            // General rule in rust:
            // last expression in a block = implicit return (no semicolon)
            // Statement with a semicolon = returns nothing ()
            // Works for functions, match arms, if/else blocks, loops with break values
            // The semicolon is powerful in Rust - it is the difference between Expression (returns a value) versus Statement (returns nothing)
        }
    }
}

impl fmt::Display for Recipe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!() writes text WITHOUT a new line (think of it like print!())
        // writeln!() writes text WITH a newline (adds \n at the end) - think of it like println!()
        writeln!(f, "Recipe: {}", self.title)?;
        // We need the ? operator for handling the Result type
        // writeln!() returns fmt::Result - fmt::Result is actually Result<(), fmt::Error>
        // It can be either Ok(()) or Err(fmt::Error)
        // We did not use the ? operator in the match statement since it is an implicit return
        // We need ? here since there are multiple writeln!() calls, each one can potentially fail, and we need to check each one and propagate errors
        // if we dont use ?, the Result is just ignored
        // At the end, we explicitly return Ok(())
        // With one write, you can return it directly. With multiple writes, you need to check each one
        writeln!(f, "Serving: {}", self.servings)?;
        // The semicolon here is about discarding the value and moving to the next statement
        // We extract the unit type (), which is nothing/no meaningful value, discard it, and move on to the next line of code
        // Without the semicolon, it would try to return () here
        // ? is propagating errors and ; is discarding the unit type ()
        // The semicolon means: I've handled this line, throw away the result, keep going
        writeln!(f, "\nIngredients:")?;
        for ingredient in &self.ingredients {
            writeln!(f, " - {}: {}", ingredient.name, ingredient.amount)?;
        }

        writeln!(f, "\nInstructions:")?;

        // enumerate gives you both the index and the value in a loop
        // i = index (0, 1, 2, ...)
        // instruction = the actual instruction
        // enumerate is super useful when you need numbered lists or to know the position while iterating
        for (i, instruction) in self.instructions.iter().enumerate() {
            // We are doing i + 1 since the index will start at 0 - we want it to be human readable
            writeln!(f, " {}. {}", i + 1, instruction)?;
        }

        Ok(())
    }
}

// Understanding the From trait pattern
    // impl From<SourceType> for TargetType {
    //     fn from(value: SourceType) -> Self {
    //         // Convert value to Self
    //     }
    // }

// The From trait in rust is a convenient way to say "Here's how to convert one type into another type"
// Use From when when you want a clean, defined conversion, converting should always succed, and you want easy into() support
// TryFrom: conversion might fail -> returns Result
    // impl From<Measurement> for Measurement {
    //     fn from(measurement: Measurement) -> Self {
    //         match measurement {
    //             // Convert cups to millileters
    //             Measurement::Cups(amount) => Measurement::Milliliters(amount * 240.0),
    //             // Convert tablespoons to milliliters
    //             Measurement::Tablespoons(amount) => Measurement::Milliliters(amount * 15.0),
    //             Measurement::Grams(amount) => Measurement::Grams(amount),
    //             Measurement::Milliliters(amount) => Measurement::Milliliters(amount),
    //         }
    //     }
    // }
// We won't be using From here like this since the From trait implementation is typically used for converting between
// different types, not normalizing variants of the same enum

struct NormalizedMeasurement(Measurement); // tuple struct

// Here, we are defining the From trait where we convert from Measurement to NormalizedMeasurement
impl From<Measurement> for NormalizedMeasurement {
    fn from(measurement: Measurement) -> Self {
        // We return Self since it refers to the type we are implementing for
        // We are creating an instance of NormalizedMeasurement 
        // It's like a factory function - raw Measurement -> NormalizedMeasurement instance containing converted measurement
        let normalized = match measurement {
            // We are converting from Cups to Millileters
            // If this is the Cups variant, extract the amount
            // amount * 240.0 converts cups to milliliters
            // Then we create a new millileters variant with the converted value
            Measurement::Cups(amount) => Measurement::Milliliters(amount * 240.0),
            Measurement::Tablespoons(amount) => Measurement::Milliliters(amount * 15.0),
            Measurement::Grams(amount) => Measurement::Grams(amount),
            Measurement::Milliliters(amount) => Measurement::Milliliters(amount),
        };
        NormalizedMeasurement(normalized)

        // This can be then used like:
        // let cups = Measurement::Cups(2.0)
        
        // We then convert using .into() - Rust infers the target type from the type annotation
        // let normalized: NormalizedMeasurement = cups.into()
        // This will match on Measurement::Cups(2.0) and convert it to Measurement::Milliliters and wrap it in NormalizedMeasurement
        // We are converting Measurement INTO NormalizedMeasurement

        // When you implement From<A> for B, Rust gives you:
        // B::from(a)- B FROM A - explicitly state the target type which is B
        // a.into() - A INTO B - target type inferred from context (type hint)
        // .into() is more popular since its concise and ergonomic
    }
}

struct RecipeBook {
    recipes: Vec<CategorizedRecipe>,
}

impl RecipeBook {
    fn new() -> Self {
        Self {
            recipes: Vec::new()
        }
    }

    fn add_recipe(&mut self, recipe: CategorizedRecipe) {
        self.recipes.push(recipe);
    }

    fn get_by_category(&self, category: RecipeCategory) -> Vec<&CategorizedRecipe> {
        self.recipes.iter().filter(|recipe| recipe.category == category).collect()
    }

    fn get_quick_recipes(&self, max_minutes: u32) -> Vec<&CategorizedRecipe> {
        self.recipes.iter().filter(|recipe| recipe.prep_time_minutes < max_minutes).collect()
    }

    // We are not using &mut self here since we are not modifying the original RecipeBook
    // We are, instead, making an entire new Recipe
    fn scale_recipe(&self, title: &str, factor: f64) -> Option<Recipe> {
        // The ? operator here is for an early return on None
        // When .find() returns:
        // .find() returns Option<&CategorizedRecipe>
        // Either Some(&recipe) if found
        // Or None if not found
        // The ? operator is short-hand for a match statement
        let recipe = self.recipes.iter().find(|cat_recipe| cat_recipe.recipe.title == title)?;
        // Since RecipeBook holds a vector of CategorizedRecipe, we need to go into the CategorizedRecipe which holds a Recipe struct, which holds the title
        // Thus, we need to drill down into CategorizedRecipe then Recipe

        // Here, we are going to drill down into the CategorizedRecipe then Recipe struct which holds an vector of Ingredients
        let scaled_ingredients: Vec<Ingredient> = recipe.recipe.ingredients
            .iter()
            .map(|ingredient| {
                // Create new Ingredient struct with the scaled amount
                Ingredient {
                    // We are going to clone the ingredient name since it is a String
                    name: ingredient.name.clone(),
                    // Here, the amount will depend on which enum variant it is, so we are using a match statement
                    amount: match ingredient.amount {
                        Measurement::Grams(amt) => Measurement::Grams(amt * factor),
                        Measurement::Milliliters(amt) => Measurement::Milliliters(amt * factor),
                        Measurement::Cups(amt) => Measurement::Cups(amt * factor),
                        Measurement::Tablespoons(amt) => Measurement::Tablespoons(amt * factor),
                    }
                }
            })
            // Then we will collect it into a new vector of Ingredient structs
            .collect();

        // Now, we will create and return a new recipe - the original is untouched
        // We will wrap it in Some() to match the specified return annotation
        Some(Recipe {
            title: recipe.recipe.title.clone(),
            servings: (recipe.recipe.servings as f64 * factor) as u32,
            ingredients: scaled_ingredients,
            instructions: recipe.recipe.instructions.clone(),
        })

    }
}

// Here we have lifetime annotation a
// This means that the RecipeFilter struct can only live as long as the
// CategorizedRecipe struct it references
// The lifetime a ties the RecipeFilter to the original data in the RecipeBook
// preventing the filter from outliving the data it points to
// In practice, this means as long as the RecipeBook exists, RecipeFilter can exist
// This is because RecipeBook owns the CategorizedRecipes
struct RecipeFilter<'a> {
    filtered: Vec<&'a CategorizedRecipe>,
}

impl<'a> RecipeFilter<'a> {
    fn new(recipes: &'a Vec<CategorizedRecipe>) -> Self {
        // Here, we are converting a reference to a vector of categorized recipes to
        // A vector containing references to individual categorized recipes
        // This allows you to filter, manipulate, and chain operations on individual recipe references
        let filtered: Vec<&'a CategorizedRecipe> = recipes.iter().collect();

        Self {
            filtered
        }
    }

    fn by_category(self, category: RecipeCategory) -> Self {
        let filtered: Vec<&'a CategorizedRecipe> = self.filtered.into_iter().filter(|recipe| recipe.category == category).collect();

        Self {
            filtered
        }
    }

    fn max_prep_time(self, max_prep_time: u32) -> Self {
        let filtered: Vec<&'a CategorizedRecipe> = self.filtered.into_iter().filter(|recipe| recipe.prep_time_minutes <= max_prep_time).collect();

        Self {
            filtered
        }
    }

    fn min_servings(self, servings: u32) -> Self{
        let filtered: Vec<&'a CategorizedRecipe> = self.filtered.into_iter().filter(|recipe| recipe.recipe.servings >= servings).collect();

        Self {
            filtered
        }
    }

    fn collect(self) -> Vec<&'a CategorizedRecipe> {
        self.filtered
    }

}

fn main() {
    println!("Hello, world!");
}
