use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

pub mod therion_reader;
pub mod walls_reader;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Station {
    pub name: String
}

impl Station {
    pub fn new(name: String) -> Self {
        Self {
            name
        }
    }
}

pub struct Equality {
    book0: String,
    station0: String,
    book1: String,
    station1: String
}

impl Equality {
    pub fn v0(&self) -> String {
        format!("{}@{}", self.station0, self.book0)
    }

    pub fn v1(&self) -> String {
        format!("{}@{}", self.station1, self.book1)
    }
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

pub struct Shot {
    pub length: f32,
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

pub struct Book {
    pub title: String,
    pub prefix: String,
    stations: HashSet<Rc<Station>>,
    pub shots: HashMap<(Rc<Station>, Rc<Station>), Shot>,
    sub_books: Vec<String>,
    equalities: Vec<Equality>
}

impl Book {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            prefix: String::new(),
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
    pub books: HashSet<Book>,
    pub equalities: Vec<Equality>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_station_creation() {
        let station = Station::new("A".to_string());
        assert_eq!(station.name, "A");
    }

    #[test]
    fn test_station_equality() {
        let s1 = Station::new("A".to_string());
        let s2 = Station::new("A".to_string());
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_shot_creation() {
        let shot = Shot::new(100.0, 45.0, 30.0);
        assert_eq!(shot.length, 100.0);
    }

    #[test]
    fn test_book_creation() {
        let book = Book::new();
        assert_eq!(book.title, "");
        assert_eq!(book.prefix, "");
    }

    #[test]
    fn test_equality_formatting() {
        let eq = Equality {
            station0: "A".to_string(),
            book0: "Main".to_string(),
            station1: "B".to_string(),
            book1: "Branch".to_string(),
        };
        assert_eq!(eq.v0(), "A@Main");
        assert_eq!(eq.v1(), "B@Branch");
    }

    #[test]
    fn test_cave_creation() {
        let cave = Cave::new();
        assert!(cave.books.is_empty());
        assert!(cave.equalities.is_empty());
    }
}
