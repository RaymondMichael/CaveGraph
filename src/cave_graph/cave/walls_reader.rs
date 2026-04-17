use std::fs::File;
use std::io::{self, BufRead};
use std::rc::Rc;
use super::Book;
use super::Cave;
use super::Equality;
use super::Shot;
use super::Station;

/*
 * Read in a full text file with the passed name and return something that
 * can be parsed line by line
 */
fn read(dname: &String, fname: &String) ->
    io::Result<io::Lines<io::BufReader<File>>> {
    let filename = format!("{}{}", dname, fname);
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/*
 * Read the project file to identify the names of the files containing the
 * actual survey data
 */
fn project_parse(dir_name: &String, fname: &String)
                 -> Result <Vec<String>, io::Error> {
    let lines = read(dir_name, fname)?;
    let mut next_is_book: bool = false;

    let mut books: Vec<String> = Vec::new();

    for line in lines.map_while(Result::ok) {
        let trimmed = line.trim();

        if next_is_book {
            if !trimmed.starts_with(".NAME") {
                panic!("Expected .NAME following .SURVEY in {}", fname);
            }

            /* Book names may have spaces in them */
            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            let index = trimmed.find(tokens[1]).unwrap();
            let name = trimmed[index..].to_string();

            books.push(name);
            next_is_book = false;
            continue;
        }

        if trimmed.starts_with(".SURVEY") {
            next_is_book = true;
            continue;
        }

        if trimmed.starts_with(";") ||
            trimmed.starts_with(".BOOK") ||
            trimmed.starts_with(".ENDBOOK") ||
            trimmed.starts_with(".NAME") ||
            trimmed.starts_with(".REF") ||
            trimmed.starts_with(".STATUS") {
                continue;
            }

        let token: Vec<&str> = trimmed.split_whitespace().collect();
        panic!("Unknown Walls project file token {}", token[0]);
    }

    Ok(books)
}

/*
 * This looks for various meta data about the book
 */
fn book_meta_parse(book: &mut Book, tokens: &Vec<&str>) {
    if tokens[0].to_lowercase() == "#PREFIX".to_lowercase() {
        book.prefix = tokens[1].to_string();
    }
}

/*
 * This takes a station name which may be a regular station, or a station in
 * a different book. It returns a tuple of the station's book and the station
 * as a string.
 */
fn parse_station(book_prefix: &String, station_name: &str) -> (String, String) {
    if station_name.contains(':') {
        let parts: Vec<&str> = station_name.split(':').collect();
        (parts[0].to_string(), station_name.to_string())
    } else {
        (book_prefix.clone(), station_name.to_string())
    }
}

/*
 * If the passed string contains both a front sight and a back sight, then
 * return just the front sight
 */
fn front_az(token: &str) -> f32 {
    let mut value: f32;

    /* If no front sight, use the back sight */
    if token.starts_with("/") {
        let parts: Vec<&str> = token.split("/").collect();
        let tmp = parts[1];
        /* If no back sight, then it's 0.0 */
        if tmp == "--" {
            value = 0.0;
        } else {
            value = tmp.parse::<f32>().unwrap();
            if value >= 180.0 {
                value -= 180.0;
            } else {
                value += 180.0;
            }
        }
    } else if token.contains("/") { /* Front and back sight */
        let parts: Vec<&str> = token.split("/").collect();
        let mut tmp = parts[0];
        if tmp == "--" { /* If no front sight, use the back sight */
            tmp = parts[1];
            if tmp == "--" { /* No back sight either */
                value = 0.0;
            } else {
                value = tmp.parse::<f32>().unwrap();
                if value >= 180.0 {
                    value -= 180.0;
                } else {
                    value += 180.0;
                }
            }
        } else { /* Use the front sight */
            value = tmp.parse::<f32>().unwrap();
        }
    } else { /* Only the front sight */
        if token == "--" {
            value = 0.0;
        } else {
            value = token.parse::<f32>().unwrap();
        }
    }

    value
}

/*
 * Parse an inclination string. Handle various formats that might have
 * backsights, or only have backsights, or be "--"
 */
fn front_inc(token: &str) -> f32 {
    let mut value: f32;

    /* If no front sight, use the back sight */
    if token.starts_with("/") {
        let parts: Vec<&str> = token.split("/").collect();
        let tmp = parts[1];
        /* If no back sight, then it's 0.0 */
        if tmp == "--" {
            value = 0.0;
        } else {
            value = tmp.parse::<f32>().unwrap();
            value = -value;
        }
    } else if token.contains("/") { /* Front and back sight */
        let parts: Vec<&str> = token.split("/").collect();
        let mut tmp = parts[0];
        if tmp == "--" { /* If no front sight, use the back sight */
            tmp = parts[1];
            if tmp == "--" { /* No back sight either */
                value = 0.0;
            } else {
                value = tmp.parse::<f32>().unwrap();
                value = -value;
            }
        } else { /* Use the front sight */
            value = tmp.parse::<f32>().unwrap();
        }
    } else { /* Only the front sight */
        if token == "--" {
            value = 0.0;
        } else {
            value = token.parse::<f32>().unwrap();
        }
    }

    value
}

/*
 * Parse a length string
 */
fn parse_length(token: &str) -> f32 {
    if token.contains("i") {
        let parts: Vec<&str> = token.split("i").collect();
        let feet = parts[0].parse::<f32>().unwrap();
        let inches = parts[1].parse::<f32>().unwrap();

        feet + (inches / 12.0)
    } else {
        token.parse::<f32>().unwrap()
    }
}

/*
 * Create an equality so we can figure out which two books are involved
 */
fn insert_equality(
    book: &mut Book, other_book: String, station: String) {
    let parts: Vec<&str> = station.split(':').collect();
    assert!(other_book == parts[0]);

    let eq = Equality {
        station0: parts[1].to_string(),
        book0: other_book,
        station1: station,
        book1: book.prefix.clone()
    };

    book.equalities.push(eq);
}



/*
 * Read the data from a Walls survey file
 */
fn book_parse(dir_name: &String, book_name: &String)
              -> Result<Book, io::Error> {
    let fname = book_name.to_string() + ".SRV";
    let lines = read(dir_name, &fname)?;
    let mut book = Book::new();

    book.title = book_name.clone();
    book.prefix = book.title.clone(); // Hopefully we see #prefix

    for line in lines.map_while(Result::ok) {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with(";") {
            continue;
        }

        let tokens: Vec<&str> = trimmed.split_whitespace().collect();

        if trimmed.starts_with("#") {
            book_meta_parse(&mut book, &tokens);
            continue;
        }

        /* This line only has LRUDs for a station */
        if tokens[1].starts_with("*") || tokens[1].starts_with("<") {
            continue;
        }

        /* This covers an equivalent reverse shot to get the other LRUDs */
        if tokens[2].starts_with("*") || tokens[2].starts_with("<") {
            continue;
        }

        /* Returns (prefix, (prefix:station / station)) */
        let (bk0, st0) = parse_station(&book.prefix, &tokens[0]);
        let station0 = Rc::new(Station::new(st0.clone()));
        let (bk1, st1) = parse_station(&book.prefix, &tokens[1]);
        let station1 = Rc::new(Station::new(st1.clone()));

        let length = parse_length(tokens[2]);
        let az = front_az(tokens[3]);
        let inclination = front_inc(tokens[4]);

        if !book.stations.contains(&station0) {
            book.stations.insert(station0.clone());
        }
        if !book.stations.contains(&station1) {
            book.stations.insert(station1.clone());
        }

        /* Sets st0@bk0 == bk0:st0@book.prefix */
        if bk0 != book.prefix {
            insert_equality(&mut book, bk0, st0);
        }
        if bk1 != book.prefix {
            insert_equality(&mut book, bk1, st1);
        }

        let shot: Shot = Shot::new(length, az, inclination);
        book.shots.insert((station0, station1), shot);
    }

    Ok(book)
}

/*
 * Read a Walls cave project into an internal Cave object
 */
pub fn read_walls(cave: &mut Cave, dir_name: &String, cave_name: &String) {
    let mut cave_file_name: String = cave_name.to_string();
    if !cave_name.ends_with(".wpj") {
        cave_file_name = cave_file_name + ".wpj";
    }

    let book_names = project_parse(dir_name, &cave_file_name).unwrap();

    for book_name in book_names.iter() {
        match book_parse(dir_name, &book_name) {
            Ok(book) => {
                /* Better to move, but this works for now */
                for sub_equality in book.equalities.iter() {
                    cave.equalities.push(sub_equality.clone());
                }
                cave.books.insert(book);
            },
            Err(_) => panic!("Error parsing Walls survey file: {}", book_name)
        };
    }
}
