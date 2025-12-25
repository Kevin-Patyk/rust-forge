#![allow(dead_code)]
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

struct Song {
    title: String,
    artist: String,
    duration_secs: u32,
    play_count: u32,
}

struct Playlist {
    name: String,
    // Song - struct with song data
    // RefCell<Song> - allows mutation through shared references
    // Rc<RefCell<Song> - allows multiple ownership + mutation
    // Vec<...> - vector of these shared, mutable songs
    songs: Vec<Rc<RefCell<Song>>>,
    // Now, we can push the song to multiple playlists
    // And we can update the play_count of the song through these different playlists
    creator: String,
}

struct Library {
    songs: Vec<Rc<RefCell<Song>>>,
    playlists: Vec<Playlist>,
}

impl Song {
    fn new(title: String, artist: String, duration_secs: u32) -> Self {
        Self {
            title,
            artist,
            duration_secs,
            play_count: 0,
        }
    }

    fn play(&mut self) {
        self.play_count += 1;
    }

    fn total_minutes(&self) -> f64 {
        self.duration_secs as f64 / 60.0 
    }
}

impl Playlist {
    fn new(name: String, creator: String) -> Self {
        Self {
            name, 
            songs: Vec::new(),
            creator,
        }
    }

    fn add_song(&mut self, song: Rc<RefCell<Song>>) {
        self.songs.push(song);
    }

    fn total_duration(&self) -> u32 {
        // song is &Rc<RefCell<Song>>
        // .borrow() is Ref<Song> (smart pointer to song)
        // with .duration_secs, it is u32 (the actual field)
        // The Ref<Song> type is returned by .borrow(), acts like &Song (can access fields), automatically derefs to song, keeps track of borrow for runtime checking
        // Rc<RefCell<T>> -> .borrow() -> access fields
        self.songs.iter().map(|song| song.borrow().duration_secs).sum()
        // We can't access fields through &Rc<RefCell<Song>> because it is not a Song type -> it's like nested boxes 
        // Rc automatically derefs to RefCell<Song>
        // To get through RefCell, we need .borrow()
        // .borrow() gives you temporary read access to the data inside RefCell
    }

    fn most_played(&self) -> Option<String> {
        if self.songs.is_empty() {
            return None
        }

        // We start with initial values as as a tuple of (0, None)
        // This is tracking (max_play_count, Option<Title>)
        self.songs.iter().fold((0, None), |acc, song| {

            // Here, we are borrowing access to Song fields
            let borrowed = song.borrow();

            // if borrowed.play_count > acc.0 - acc.0 is the max play count so far
            if borrowed.play_count > acc.0 {
                // This song has more plays - update both
                (borrowed.play_count, Some(borrowed.title.clone()))
            } else {
                // If borrowed.play_count is less than the current accumulator, keep the current max
                acc
            }
        }).1 // Extract just the Option<String> (second element of the tuple)
        // Discard the count - just return the title

        // .fold() is an iterator adapter that reduces a sequence of items into a single accumulated value
        // It works by repeatedly applying a closure to an accumulator and each item in the iterator

        // Alternative:
            // fn most_played(&self) -> Option<String> {
            //     self.songs
            //         .iter()
            //         .map(|song| {
            //             let s = song.borrow();
            //             (s.play_count, s.title.clone())
            //         })
            //         .max_by_key(|(count, _)| *count)
            //         .map(|(_, title)| title)
            // }
    }

    fn play_all(&self) {
        // .for_each() is an iterator method that runs a closure on each item and consumes the iterator immediately
        // .map() is lazy, doesn't run until consumed 
        // .for_each() is eager, it runs immediately
        // use .for_each() for side effects, like mutations, print, write
        // use .map() for transformations
        // .for_each() can be thought of as "Do this action for every item, right now"
        self.songs.iter().for_each(|song | {
            song.borrow_mut().play_count += 1
        });
        // .borrow_mut() gives you temporary mutable access to the data inside RefCell, while .borrow() gives read-only access
        // .borrow_mut() returns RefMut<T>, while .borrow() returns Ref<T>
    }   
}

impl Library {
    fn new() -> Self {
        Self {
            songs: Vec::new(),
            playlists: Vec::new(),
        }
    }

    fn add_song(&mut self, song: Song) -> Rc<RefCell<Song>> {
        // We meed to wrap the song in Rc::new(RefCell::new()) before pushing it to the vector so it can match the type
        let wrapped = Rc::new(RefCell::new(song));

        // Push a clone of the Rc (cheap - just increments reference count)
        // Clone the Rc pointer, not the song data
        self.songs.push(Rc::clone(&wrapped));

        // Return the Rc
        wrapped
    }

    fn add_playlist(&mut self, playlist: Playlist) {
        self.playlists.push(playlist)
    }

    fn find_song(&self, title: &str) -> Option<Rc<RefCell<Song>>> {
        // We iterate over &Rc<RefCell<Song>>
        // .find() the first matching song
        // then borrow and compare
        // Then we clone the Rc if found 
        // .find() returns Option<&Rc<RefCell<Song>>>
        // .cloned() returns Option<Rc<RefCell<Song>>> 
        // Cheap - just increments the reference counter
        // We need to clone the Rc so it is owned and can match the return annotation
        self.songs.iter().find(|songs| songs.borrow().title == title).cloned()
    }

    fn total_plays(&self) -> u32 {
        self.songs.iter().map(|song| song.borrow().play_count).sum()
    }

    fn most_popular_artist(&self) -> Option<String> {
        let mut artist_plays: HashMap<String, u32> = HashMap::new();

        for song in &self.songs {
            let borrowed = song.borrow();
            // .entry() and .or_insert() are part of the entry API for HashMap
            // They let you insert or modify a value for a given key in one step without having to check if the key exists
            // .entry() returns an enum representing if that key does or not exist
            // If it exists, you get the existing key, if it does not, it make a new key
            // .or_insert() - if the key already exists, returns a mutable reference to the existing value
            // If the key does not exist, it inserts the default value and returns a mutable reference to it
            // "Look up this artist in the map. If they aren't there yet, insert them a play count of 0. Then add this song's play count to their total."
            *artist_plays.entry(borrowed.artist.clone()).or_insert(0) += borrowed.play_count
        }

        // .into_iter() consumes the HashMap
        // Each item is a tuple (artist_name, total_plays)
        // .max_by_key() finds the item with the maximum value based on a key function - returns Option<Item>
        artist_plays
            .into_iter()
            // Destructure the tuple (artist, plays)
            // _ = ignore the artist name
            // plays = bind the u32 value
            // .max_by_key() finds the item with maximum key value -> returns Option
            // For each tuple, extract the second element (plays) and compare
            // Find the max key and return the WHOLE tuple
            // Returns Option<String, u32> in this example
            .max_by_key(|(_, plays)| *plays)
            // We need *plays since without it, it is u32 (reference from destructuring)
            // max_by_key needs an Ord type to compare
            // *total_plays deferences to u32 which implements Ord
            .map(|(artist, _)| artist)
            // This is .map() on an Option not an Iterator
            // It transforms the value inside from tuple to just artist (String) without unwrapping
    }

    // max_by_key() is an iterator adaptor that lets you find the maximum item in an iterator based on a derived key
    // You give it a closure that maps each item -> a key 
    // It returns the item whose key is the greatest
    // The key must implement Ord and the comparison is done with standard ordering
    // iterator.max_by_key(|item| key)
    // key - refers to a function that extracts or computes a value from each element that will be used for comparison
    // Think of it like sorting or comparing items based on a specific property rather than the items themselves
    // The key function transforms each element into something comparable
    // It returns the item with the largest key
}

fn main() {
    // Imagine you have a song that appears in 3 different playlists
    // When someone plays it, you want to the play count to update everywhere
    // Normal Rust ownership - doesn't work:
        // let song = Song::new("Bohemian Rhapsody".into(), "Queen".into(), 354);

    // This would not work:
        // playlist1.add_song(song); - Song moved here
        // playlist2.add_song(song); - Error! Song was already moved

    // We need multiple ownership - multiple playlists "own" the same song
    // Mutability - we need to update play_count even through shared references

    // Rc = Reference counted pointer
    // Allows multiple owners of the same data
    // Keeps track of how many owners exist
    // Data is dropped when the last owner is dropped

        // let song = Rc::new(Song::new("Bohemian Rhapsody".into(), "Queen".into(), 354)); // Wrap the song in Rc
    
    // Now we can clone the Rc (cheap - just increments counter)
        // let song_ref1 = Rc::clone(&song); - Reference count: 2
        // let song_ref2 = Rc::clone(&song); - Reference count: 3
        // let song_ref3 = Rc::clone(&song); - Reference count: 4

    // All points to the SAME song in memory
    // Rc::strong_count() returns how many strong references exist go to a given Rc<T> value
        // println!("References: {}", Rc::strong_count(&song));  // Prints: 4
    
    // Key points about Rc:
    // .clone() is cheap - it just increments the counter, doesn't copy data
    // All clones point to the same data
    // Data is freed when all the Rcs are dropped
    // Read only by default - you get &T, not &mut T

    // The problem with just Rc:
        // let song = Rc::new(Song { playcount_0, ... })
    
    // Can't mutate through Rc!
        // song.play_count += 1; // Error! Can't mutate through shared reference
    
    // We wrap in RefCell for interior mutability
        // let song = Rc::new(RefCell::new(Song { play_count: 0, ... }));
    
    // Borrow immutably
        // let borrowed = song.borrow(); - Returns Ref<Song?
    
    // Borrow mutably
        // let mut borrowed = song.borrow_mut();

    // RefCell:
    // Moves borrow checking from compile-time to runtime
    // .borrow() gets immutable reference Ref<T>
    // .borrow_mut() gets mutable reference RefMut<T>
    // Panics if you violate borrow rules at runtime
    // Same borrow rules as Rust but checked at runtime

    // When you combine Rc and RefCell:
    // You can create a song with shared ownership AND mutability
    let song = Rc::new(RefCell::new(Song {
        title: "Bohemian Rhapsody".into(),
        artist: "Queen".into(),
        duration_secs: 354,
        play_count: 0,
    }));

    // Add to multiple playlists (clone the Rc, not the song!)
    let song_in_playlist1 = Rc::clone(&song);
    let _song_in_playlist2 = Rc::clone(&song);
    let _song_in_playlist3 = Rc::clone(&song);

    // Play the song through any reference - updates everywhere!
    song_in_playlist1.borrow_mut().play_count += 1;

    // Use Rc<T> when:
    // Multiple parts need to read the same data
    // Single-threaded only
    // Example: Shared configuration, read-only cache

    // Use RefCell<T> when:
    // Need to mutate through a shared reference
    // Single owner, but need interior mutability
    // Example: Mock objects in tests
    
    // Use Rc<RefCell<T>> when:
    // Multiple owners need to read AND write the same data
    // Single threaded only

    // The pattern Rc<RefCell<T>> is shared ownership + mutability
    // It is a very common pattern in Rust in single-threaded Rust

    let mut library = Library::new();

    let song = Song::new(
        "Imagine".to_string(),
        "John Lennon".to_string(),
        183,
    );

    let song_ref = library.add_song(song);

    // Should print 2: Library has one, song_ref has one
    println!("After adding to library - Rc count: {}", Rc::strong_count(&song_ref));
    
    // Create multiple playlists 
    let mut playlist1 = Playlist::new("Classics".to_string(), "Alice".to_string());
    let mut playlist2 = Playlist::new("Favorites".to_string(), "Bob".to_string());
    let mut playlist3 = Playlist::new("Chill".to_string(), "Charlie".to_string());

    // Add the SAME song to all playlists (clone the Rc, not the song!)
    playlist1.add_song(Rc::clone(&song_ref));
    println!("After adding to playlist1 - Rc count: {}", Rc::strong_count(&song_ref));
    // Should print: 3
    
    playlist2.add_song(Rc::clone(&song_ref));
    println!("After adding to playlist2 - Rc count: {}", Rc::strong_count(&song_ref));
    // Should print: 4
    
    playlist3.add_song(Rc::clone(&song_ref));
    println!("After adding to playlist3 - Rc count: {}", Rc::strong_count(&song_ref));
    // Should print: 5

    println!("\n=== Initial Play Counts ===");
    println!("From song_ref: {}", song_ref.borrow().play_count);
    println!("From playlist1: {}", playlist1.songs[0].borrow().play_count);
    println!("From playlist2: {}", playlist2.songs[0].borrow().play_count);

    // Play song from playlist1
    println!("\n=== Playing song from playlist1 ===");
    playlist1.songs[0].borrow_mut().play_count += 1;
    
    // Check play count from ALL references - should all be 1!
    println!("From song_ref: {}", song_ref.borrow().play_count);
    println!("From playlist1: {}", playlist1.songs[0].borrow().play_count);
    println!("From playlist2: {}", playlist2.songs[0].borrow().play_count);
    println!("From playlist3: {}", playlist3.songs[0].borrow().play_count);
    println!("From library: {}", library.songs[0].borrow().play_count);

    // Play all songs in playlist2 (which has just our one song)
    println!("\n=== Using play_all() on playlist2 ===");
    playlist2.play_all();
    
    // Check again - now should be 2!
    println!("From song_ref: {}", song_ref.borrow().play_count);
    println!("From playlist1: {}", playlist1.songs[0].borrow().play_count);
}
