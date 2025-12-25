#![allow(dead_code)]

#[derive(Debug)]
// This is an enum with variants
// Each variant is like a mini-struct
// Pattern match to access variant data
// Useful for representing types that are one of several things
// Can have different fields per variant
enum Media {
    Audio { title: String, artist: String, duration_secs: u32},
    Video { title: String, resolution: String, duration_secs: u32},
    Podcast { title: String, episode: u32, duration_secs: u32 },
}

struct AudioFile {
    title: String,
    artist: String,
    duration_secs: u32,
}

struct VideoFile {
    title: String,
    resolution: String,
    duration_secs: u32,
}

struct PodcastFile {
    title: String,
    episode: u32,
    duration_secs: u32,
}

struct MediaInfo {
    title: String,
    duration_secs: u32,
    media_type: String,
}

// Here, we are defining how to convert from AudioFile to Media
// How to convert from a struct to an enum
// This will allow us to use .into() and ::from for conversions
impl From<AudioFile> for Media {
    fn from(file: AudioFile) -> Self {
        Media::Audio{ title: file.title, artist: file.artist, duration_secs: file.duration_secs}
    }
}

// Here, we are defining how to convert from VideoFile to Media
// How to convert from a struct to an enum
// This will allow us to use .into() and ::from for conversions
impl From<VideoFile> for Media {
    fn from(file: VideoFile) -> Self {
        Media::Video{ title: file.title, resolution: file.resolution, duration_secs: file.duration_secs }
    }
}

// Here, we are defining how to convert from PodcastFile to Media
// How to convert from a struct to an enum
// This will allow us to use .into() and ::from for conversions
impl From<PodcastFile> for Media {
    fn from(file: PodcastFile) -> Self {
        Media::Podcast{ title: file.title, episode: file.episode, duration_secs: file.duration_secs}
    }
}

// Note from before, but we can also implement the From trait for error types
// It allows one error type to be automatically converted into another, which is essential for building error enums,
// propagating errors with ?, or composing libraries

// Implementing From automatically gives you Into
// Used for infallible conversions (always succeeds)
impl From<Media> for MediaInfo {
    fn from(media: Media) -> Self {
        match media {
            // We are using the _ operator here since we do not care about the second field, so we are just discarding it
            // As a note, we have to put the field name first - it would not work with just _ without the leading field name
            Media::Audio{ title, artist: _, duration_secs } => MediaInfo { title, duration_secs, media_type: "Audio".to_string() },
            Media::Video{ title, resolution: _, duration_secs } => MediaInfo { title, duration_secs, media_type: "Video".to_string() },
            Media::Podcast{ title, episode: _, duration_secs } => MediaInfo { title, duration_secs, media_type: "Podcast".to_string() },
        }
    }
}

struct Playlist {
    name: String,
    items: Vec<Media>,
}

impl Playlist {
    fn new(name: String) -> Self {
        Self {
            name,
            items: Vec::new(),
        }
    }

    // This function accepts any type that converts into Media
    // This is a generic function with a trait bound
    // T is a generic type parameter
    // T: Into<Media> is a trait bound saying: "I must implement Into<Media>"
    fn add<T: Into<Media>>(&mut self, item: T) {
        // We have to call .into() here to convert T to Media since the field items requires Vec<Media>
        self.items.push(item.into());
    }

    fn total_duration(&self) -> u32 {
        // We need to use the match statement here since Media is an enum
        self.items.iter().map(|item| {
            match item {
                // The .. syntax inside a pattern is a struct pattern shorthand for "ignore the rest of the fields"
                // "I am only interested in duration_secs; ignore all other fields in this struct"
                Media::Audio { duration_secs, .. } => *duration_secs,
                // We need to dereference to get the actual value
                // Dereferencing gets the value that the pointer points to
                // In this case, .iter() returns a iterator of references (&Media)
                // When we match, item is &Media (reference)
                // This binds a REFERENCE to the duration_secs (&u32) field
                // Thus, to get the value, we need to dereference since sum expects u32 not &u32
                Media::Video { duration_secs, .. } => *duration_secs,
                Media::Podcast { duration_secs, .. } => *duration_secs,
                // When you have a reference to an enum or struct, all fields you extract through pattern matching are also references
                // If you could extract owned values from a reference, you would be taking ownership of something you only borrowed, which violates Rust's ownership rules
                // When you match &T, all extracted fields are &FieldType
            }
        }).sum()
    }

    fn get_titles(&self) -> Vec<String> {
        self.items.iter().map(|item| {
            match item {
                // We need to clone here since item is &Media (reference)
                // If we do not clone, then we would be moving ownership, which we do not want to do
                // .clone() does NOT dereference, but it works on reference because of how Clone is implemented
                // .clone() creates a new, owned value from a reference
                Media::Audio { title, .. } => title.clone(),
                Media::Video { title, .. } => title.clone(),
                Media::Podcast { title, .. } => title.clone(),
            }
        }).collect()
    }
}

// AsRef is a trait that allows cheap reference conversion from one type to another
// "I can give you a reference &T to something inside me, cheaply"
// A lightweight, zero-cost way to convert a type to a reference of another type
// It can make functions more flexible - they can accept multiple types: 
fn print_text<S: AsRef<str>>(text: S) {
    println!("{}", text.as_ref());
}
// It is about flexible, cheap reference conversion

// I am implementing the AsRef<str> trait for the Media type
// Media can be converted into a reference to str (&str)
// This is read only
impl AsRef<str> for Media {
    // AsRef is the trait name
    // <str> is the generic parameter - what type of reference we return
    // We are giving Media the ability to convert to &str
    // Now Media can be used anywhere that accepts AsRef<str>
    fn as_ref(&self) -> &str {
        // This method is how to get the &TargetType from &self
        // You can then call .as_ref() on Media to get &str
        match self {
            Media::Audio { title, .. } => &title,
            Media::Video { title, .. } => &title,
            Media::Podcast { title, .. } => &title,
        }
    }
    // Now we can pass Media titles to functions expecting &str
    // If a function accepts &str and your type implements AsRef<str>, you can convert your type to a &str by calling .as_ref() and use it in the function
}
// The generic parameter in AsRef<T> determines what .as_ref() returns
// The .as_ref() method always returns a reference - that is the whole point of the AsRef trait
// AsRef = "As a Reference" - always borrows, never owns

// This can modify, as opposed to AsRef
// It is just like AsRef but for mutable access
// Media can be converted into a mutable String
// This has write access
impl AsMut<String> for Media {
    // AsRef is the trait name
    // <String> is the generic parameter - what type of mutable reference we return
    // We are giving Media the ability to convert to a mutable String
    fn as_mut(&mut self) -> &mut String {
        // This method is how to get &mut TargetType from &mut self
        match self {
            Media::Audio { title, .. } => title,
            Media::Video { title, .. } => title,
            Media::Podcast { title, .. } => title,
        }
    }
}
// You can now use Media in any function that requires <S: AsMut<String>>
// It would be read as: This function accepts any type S, as long as S implements AsMut<String>, meaning S can be converted into a &mut String

// AsRef<T> = "How this type can be converted into a reference &T" -> give me a read only view
// AsMut<T> = "How this type can be converted into a mutable reference &mut T" -> give me a a writable view
// Both enable functions to accept multiple types generically
// Very cheap - just borrowing, no allocation

// This is a newtype struct
// Single, unnamed field
// Wraps media in a new type
struct MediaWrapper(Media);

// How this type can be converted into a reference &T
// How MediaWrapper can be converted to a reference to it's inner type
impl AsRef<Media> for MediaWrapper {
    // Allows MediaWrapper to be converted to &Media
    fn as_ref(&self) -> &Media {
        &self.0 // self.0 accesses the first and only field and returns a reference
    }
}

// Converts MediaWrapper INTO Media (unwraps it)
// Takes ownership of wrapper
// Returns the inner Media
// Works with:
    // let wrapper = MediaWrapper(Media::Audio { ... });
    // let media: Media = wrapper.into();
    // let media: Media = Media::from(wrapper);
impl From<MediaWrapper> for Media {
    fn from(wrapper: MediaWrapper) -> Self {
        wrapper.0
    }
}

fn main() {
    // This function accepts any type S, as long as S implements AsRef<str>, meaning it can be converted into a &str
    fn print_title<S: AsRef<str>>(title: S) {
        println!("Title: {}", title.as_ref());
    }

    // Usage example for AsRef<str>
    let audio = Media::Audio {
        title: "My Song".to_string(),
        artist: "Artist".to_string(),
        duration_secs: 180,
    };

    // Now you can pass Media directly!
    print_title(&audio);  // Works! Converts Media to &str
    print_title("Direct string");  // Also works!
    print_title(String::from("Owned"));  // Also works!

    // 1. &audio is &Media
    // 2. Compiler sees print_title needs AsRef<str>
    // 3. Media implements AsRef<str> 
    // 4. Calls audio.as_ref() -> returns &str (the title)
    // 5. Function receives "My Song" as &str

    // We can use AsMut in the following way:
    let mut audio = Media::Audio {
        title: "Old Title".to_string(),
        artist: "Artist".to_string(),
        duration_secs: 180,
    };

    // Get a mutable reference to the title field
    let title_ref: &mut String = audio.as_mut();

    // Modify the title mutable reference
    // title_ref is NOT a copy or a new string
    // title_ref POINTS to the title field inside audio
    // When you modify title_ref, you are modifying the original (audio.title)
    // Mutable references let you modify the original data in place
    title_ref.push_str(" - Extended");  
    // The title inside of audio is now "Old Title - Extended"

    // Or we can modify it directly
    audio.as_mut().push_str(" Mix");

    // Now, we can see the modified title
    println!("{:?}", audio); // The title will now be: "Old Title - Extended Mix"

    // 
}
 