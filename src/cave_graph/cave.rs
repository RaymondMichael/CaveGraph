use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

pub mod therion_reader;

#[derive(PartialEq, Eq, Hash)]
struct Station {
    name: String
}

impl Station {
    pub fn new(name: String) -> Self {
        Self {
            name
        }
    }
}

struct Equality {
    book0: String,
    station0: String,
    book1: String,
    station1: String
}

impl Clone for Equality {
    fn clone(&self) -> Self {
        Self {
            book0: self.book0.to_string(),
            station0: self.station0.to_string(),
            book1: self.book1.to_string(),
            station1: self.station1.to_string()
        }
    }
}

struct Shot {
    length: f32,
    azimuth: f32,
    inclination: f32
}

impl Shot {
    pub fn new(length: f32, azimuth: f32, inclination: f32) -> Self {
        Self {
            length,
            azimuth,
            inclination
        }
    }
}

struct Book {
    title: String,
    stations: HashSet<Rc<Station>>,
    shots: HashMap<(Rc<Station>, Rc<Station>), Shot>,
    sub_books: Vec<String>,
    equalities: Vec<Equality>
}

impl Book {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            stations: HashSet::new(),
            shots: HashMap::new(),
            sub_books: Vec::new(),
            equalities: Vec::new()
        }
    }
}

impl PartialEq for Book {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
    }
}
impl Eq for Book {}


impl Hash for Book {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
    }
}

pub struct Cave {
    books: HashSet<Book>,
    equalities: Vec<Equality>
}

impl Cave {
    pub fn new() -> Self {
        Self {
            books: HashSet::new(),
            equalities: Vec::new()
        }
    }

    pub fn print(&self) {
        for book in self.books.iter() {
	     println!("Title is {}", book.title);

            for s in book.stations.iter() {
                println!("Station is {}", s.name);
            }

            for s in book.shots.iter() {
                let ((stat1, stat2), shot) = s;
                println!("Shot: {} {} {} {} {}",
                         stat1.name, stat2.name,
                         shot.length, shot.azimuth, shot.inclination);
            }
        }
    }
}
