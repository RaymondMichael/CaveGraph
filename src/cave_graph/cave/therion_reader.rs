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
 * Parse a shot in a text file and add the shot into the book
 */
fn shot_parse(book: &mut Book, tokens: Vec<&str>) {
    let station1 = Rc::new(Station::new(String::from(tokens[0])));
    if !book.stations.contains(&station1) {
        book.stations.insert(Rc::clone(&station1));
    }

    if tokens[1] == "." || tokens[1] == "-" {
        return;
    }

    let station2 = Rc::new(Station::new(String::from(tokens[1])));
    if !book.stations.contains(&station2) {
        book.stations.insert(Rc::clone(&station2));
    }

    let len: f32 = tokens[2].parse::<f32>().unwrap();
    let az:  f32 = tokens[3].parse::<f32>().unwrap();
    let inc: f32 = tokens[4].parse::<f32>().unwrap();
    let shot: Shot = Shot::new(len, az, inc);
    book.shots.insert((station1, station2), shot);
}

/*
 * Read a station equality statement from a Therion file
 */
fn parse_equality(tokens: Vec<&str>) -> Equality {
    let st_string = tokens[1].to_string();
    let parts: Vec<&str> = st_string.split('@').collect();
    let s0 = parts[0];
    let b0 = parts[1];
    let st_string = tokens[2].to_string();
    let parts: Vec<&str> = st_string.split('@').collect();
    let s1 = parts[0];
    let b1 = parts[1];
    Equality {
        station0: s0.to_string(),
        station1: s1.to_string(),
        book0: b0.to_string(),
        book1: b1.to_string()
    }
}

/*
 * Take the name of a text file of survey data and return a Book of the data
 */
fn book_parse(dir_name: &String, fname: &String) -> Option<Book> {
    let lines = match read(dir_name, fname) {
        Ok(lines) => lines,
        Err(_e) => return None
    };

    let mut book = Book::new();
    let mut in_map: bool = false;

    for line in lines.map_while(Result::ok) {
        let trimmed = line.trim();

        if in_map {
            if trimmed.starts_with("endmap") {
                in_map = false;
            }
            continue;
        }

        if trimmed.starts_with('#') ||
            trimmed.starts_with("calibrate") ||
            trimmed.starts_with("centerline") ||
            trimmed.starts_with("cs") ||
            trimmed.starts_with("date") ||
            trimmed.starts_with("encoding") ||
            trimmed.starts_with("endcenterline") ||
            trimmed.starts_with("endsurvey") ||
            trimmed.starts_with("extend") ||
            trimmed.starts_with("flags") ||
            trimmed.starts_with("fix") ||
            trimmed.starts_with("join") ||
            trimmed.starts_with("station") ||
            trimmed.starts_with("team") ||
            trimmed.starts_with("units") ||
            trimmed.len() == 0 {
                continue;
            }

        let tokens:Vec<&str> = trimmed.split_whitespace().collect();

        if tokens[0] == "data" {
            if tokens[1] == "dimensions" {
                break;
            }
        } else if tokens[0] == "equate" {
            let equality: Equality = parse_equality(tokens);
            book.equalities.push(equality);
        } else if tokens[0] == "input" {
            book.sub_books.push(String::from(tokens[1]));
        } else if tokens[0] == "survey" {
            book.title = String::from(tokens[1]);
            book.prefix = book.title.clone();
        } else if tokens[0] == "map" {
            in_map = true;
            continue;
        } else {
            shot_parse(&mut book, tokens);
        }
    }

    Some(book)
}

pub fn read_therion(cave: &mut Cave, dir_name: &String, cave_name: &String) {
    if cave_name.contains(".th2") {
        return;
    }

    let mut cave_file_name: String = cave_name.to_string();
    if !cave_name.ends_with(".th") {
        cave_file_name = cave_file_name + ".th";
    }

    let res = book_parse(dir_name, &cave_file_name);

    match res {
        Some(book) => {
            /* Add the directory of the file to the current directory */
            let (prefix, _suffix) = match cave_name.rsplit_once('/') {
                Some((prefix, _suffix)) => (prefix, _suffix),
                None => ("", cave_name.as_str())
            };
            let ndir_name: String = dir_name.to_owned() +
                &prefix.to_string() + "/";

            for sub_book in book.sub_books.iter() {
                read_therion(cave, &ndir_name, sub_book);
            }
            /* It would be better to move, but this works for now */
            for sub_equality in book.equalities.iter() {
                cave.equalities.push(sub_equality.clone());
            }

            cave.books.insert(book);
        }
        None => {
            println!("Failed to parse book {}", cave_file_name);
        }
    }
}
