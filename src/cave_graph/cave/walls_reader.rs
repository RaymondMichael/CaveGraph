use std::fs::File;
use std::io::{self, BufRead};
//use std::rc::Rc;
//use super::Book;
use super::Cave;
//use super::Equality;
//use super::Shot;
//use super::Station;

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

    let mut books: Vec<String> = Vec::new();

    for line in lines.map_while(Result::ok) {
        let trimmed = line.trim();

        if trimmed.starts_with(";") ||
            trimmed.starts_with(".BOOK") ||
            trimmed.starts_with(".ENDBOOK") ||
            trimmed.starts_with(".REF") ||
            trimmed.starts_with(".STATUS") ||
            trimmed.starts_with(".SURVEY") {
                continue;
            }

        let tokens:Vec<&str> = trimmed.split_whitespace().collect();
        books.push(tokens[1].to_string());
    }

    Ok(books)
}

/*
 * Read the data from a Walls survey file
 */
fn book_parse(cave: &mut Cave, dir_name: &String, book_name: &String)
              -> Result<(), io::Error> {
    let fname = book_name.to_string() + ".SRV";
    let lines = read(dir_name, &fname)?;

    for line in lines.map_while(Result::ok) {
        let trimmed = line.trim();

        if trimmed.is_empty() ||
            trimmed.starts_with(";") ||
            trimmed.starts_with("#") {
                continue;
            }

        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        let from_st = tokens[0];
        let to_st = tokens[1];
        let length = tokens[2];
        let az = tokens[3];
        let inclination = tokens[4];

        println!("Survey line: {} {} {} {} {}",
                 from_st, to_st, length, az, inclination);
    }

    Ok(())
}

/*
 * Read a Walls cave project into an internal Cave object
 */
pub fn read_walls(cave: &mut Cave, dir_name: &String, cave_name: &String) {
    let mut cave_file_name: String = cave_name.to_string();
    if !cave_name.ends_with(".wpj") {
        cave_file_name = cave_file_name + ".wpj";
    }

    let books = project_parse(dir_name, &cave_file_name).unwrap();

    for book in books.iter() {
        match book_parse(cave, dir_name, &book) {
            Ok(_) => (),
            Err(_) => panic!("Error parsing Walls survey file: {}", book)
        };
    }
}
