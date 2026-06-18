use cavegraph::cave_graph::cave::{Cave, therion_reader, walls_reader};

#[test]
fn test_therion_parser_reads_valid_file() {
    let mut cave = Cave::new();
    let dir = "";
    let file = "data/HMaze.th";

    therion_reader::read_therion(&mut cave, &dir.to_string(), &file.to_string());

    assert!(!cave.books.is_empty(), "Cave should have at least one book");
}

#[test]
fn test_therion_parser_creates_shots() {
    let mut cave = Cave::new();
    let dir = "";
    let file = "data/HMaze.th";

    therion_reader::read_therion(&mut cave, &dir.to_string(), &file.to_string());

    let mut shot_count = 0;
    for book in cave.books.iter() {
        shot_count += book.shots.len();
    }
    assert!(shot_count > 0, "Cave should have shots");
}

#[test]
fn test_therion_parser_ignores_th2_files() {
    let mut cave = Cave::new();
    let dir = "";
    let file = "data/nonexistent.th2";

    // This should return without processing since it's a .th2 file
    therion_reader::read_therion(&mut cave, &dir.to_string(), &file.to_string());

    // Cave should be empty since .th2 files are skipped
    assert!(cave.books.is_empty(), "Cave should not process .th2 files");
}

#[test]
fn test_walls_parser_reads_valid_project() {
    let mut cave = Cave::new();
    let dir = "data/Walls/";
    let file = "MCSVY.wpj";

    walls_reader::read_walls(&mut cave, &dir.to_string(), &file.to_string());

    assert!(!cave.books.is_empty(), "Cave should have books from Walls project");
}

#[test]
fn test_walls_parser_creates_shots() {
    let mut cave = Cave::new();
    let dir = "data/Walls/";
    let file = "MCSVY.wpj";

    walls_reader::read_walls(&mut cave, &dir.to_string(), &file.to_string());

    let mut shot_count = 0;
    for book in cave.books.iter() {
        shot_count += book.shots.len();
    }
    assert!(shot_count > 0, "Cave should have shots from Walls project");
}

#[test]
fn test_therion_parser_extracts_survey_name() {
    let mut cave = Cave::new();
    let dir = "";
    let file = "data/HMaze.th";

    therion_reader::read_therion(&mut cave, &dir.to_string(), &file.to_string());

    assert!(!cave.books.is_empty(), "Cave should have survey book");
    let book = cave.books.iter().next().unwrap();
    assert_eq!(book.title, "HMaze", "Survey name should be HMaze");
}
