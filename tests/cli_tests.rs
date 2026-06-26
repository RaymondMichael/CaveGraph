use std::process::Command;
use std::str;

fn run_cavegraph(args: &[&str]) -> (i32, String, String) {
    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    cmd.arg("--bin");
    cmd.arg("cavegraph");
    cmd.arg("--");
    for arg in args {
        cmd.arg(arg);
    }

    let output = cmd.output().expect("Failed to execute command");
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (exit_code, stdout, stderr)
}

#[test]
fn test_diameter_flag_outputs_result() {
    let (code, stdout, _stderr) = run_cavegraph(&["data/HMaze.th", "--diameter"]);

    assert_eq!(code, 0, "Should exit successfully");
    assert!(stdout.contains("Graph diameter is"), "Should output diameter result");
    assert!(stdout.contains("stations"), "Should name the stations");
}

#[test]
fn test_path_flag_with_valid_stations() {
    let (code, stdout, _stderr) =
        run_cavegraph(&["data/HMaze.th", "--path", "M9@HMaze", "M16@HMaze"]);

    assert_eq!(code, 0, "Should exit successfully");
    assert!(
        stdout.contains("Shortest distance is"),
        "Should output path distance"
    );
}

#[test]
fn test_path_flag_can_show_count_and_vertices() {
    let (code, stdout, _stderr) = run_cavegraph(&[
        "data/HMaze.th",
        "--path",
        "M9@HMaze",
        "M16@HMaze",
        "--show-vertex-count",
        "--show-path",
    ]);

    assert_eq!(code, 0, "Should exit successfully");
    assert!(stdout.contains("Shortest distance is"), "Should output path distance");
    assert!(stdout.contains("Vertices on path:"), "Should output inclusive vertex count");
    assert!(stdout.contains("Path:"), "Should output ordered path");
    assert!(stdout.contains("M9@HMaze"), "Should include starting vertex in the path output");
    assert!(stdout.contains("M16@HMaze"), "Should include ending vertex in the path output");
}

#[test]
fn test_path_flag_with_unknown_start_station() {
    let (code, _stdout, stderr) =
        run_cavegraph(&["data/HMaze.th", "--path", "UNKNOWN@HMaze", "M16@HMaze"]);

    assert_ne!(code, 0, "Should exit with error");
    assert!(
        stderr.contains("Unknown starting station"),
        "Should report unknown starting station"
    );
}

#[test]
fn test_path_flag_with_unknown_end_station() {
    let (code, _stdout, stderr) =
        run_cavegraph(&["data/HMaze.th", "--path", "M9@HMaze", "UNKNOWN@HMaze"]);

    assert_ne!(code, 0, "Should exit with error");
    assert!(
        stderr.contains("Unknown ending station"),
        "Should report unknown ending station"
    );
}

#[test]
fn test_no_midpoints_flag_with_diameter() {
    let (code, stdout, _stderr) = run_cavegraph(&["data/HMaze.th", "--diameter", "--no-midpoints"]);

    assert_eq!(code, 0, "Should exit successfully");
    assert!(stdout.contains("Graph diameter is"), "Should output diameter result");
}

#[test]
fn test_diameter_flag_can_show_count_and_vertices() {
    let (code, stdout, _stderr) = run_cavegraph(&[
        "data/HMaze.th",
        "--diameter",
        "--show-vertex-count",
        "--show-path",
    ]);

    assert_eq!(code, 0, "Should exit successfully");
    assert!(stdout.contains("Graph diameter is"), "Should output diameter result");
    assert!(stdout.contains("Vertices on path:"), "Should output inclusive vertex count");
    assert!(stdout.contains("Path:"), "Should output ordered path");
    assert!(stdout.contains("M9@HMaze"), "Should include one diameter endpoint");
    assert!(stdout.contains("M16@HMaze"), "Should include the other diameter endpoint");
}

#[test]
fn test_print_flag_outputs_cave_structure() {
    let (code, stdout, _stderr) = run_cavegraph(&["data/HMaze.th", "--print"]);

    assert_eq!(code, 0, "Should exit successfully");
    assert!(stdout.contains("Title is"), "Should print book title");
    assert!(stdout.contains("Station is"), "Should print stations");
}

#[test]
fn test_missing_file_returns_error() {
    let (code, _stdout, stderr) = run_cavegraph(&["data/nonexistent_file.th"]);

    assert_ne!(code, 0, "Should exit with error");
    assert!(
        stderr.contains("File not found") || stderr.contains("Error"),
        "Should report file not found"
    );
}

#[test]
fn test_invalid_file_extension() {
    // Create a temporary file with invalid extension
    std::fs::write("test_file.invalid", "dummy content").ok();

    let (code, _stdout, stderr) = run_cavegraph(&["test_file.invalid"]);

    // Clean up
    std::fs::remove_file("test_file.invalid").ok();

    assert_ne!(code, 0, "Should exit with error");
    assert!(
        stderr.contains("Invalid file extension") || stderr.contains("Error"),
        "Should report invalid extension or error. Got: {}",
        stderr
    );
}

#[test]
fn test_no_arguments_shows_usage() {
    let (code, _stdout, stderr) = run_cavegraph(&[]);

    assert_ne!(code, 0, "Should exit with error");
    assert!(
        stderr.contains("Usage:"),
        "Should show usage on missing arguments"
    );
}

#[test]
fn test_help_flag_prints_help_message() {
    let (code, stdout, stderr) = run_cavegraph(&["--help"]);

    assert_eq!(code, 0, "Should exit successfully");
    assert!(
        !stderr.contains("Error:"),
        "Should not print an error for --help. stderr was: {}",
        stderr
    );
    assert!(stdout.contains("Usage: cavegraph"), "Should print usage header");
    assert!(stdout.contains("--diameter"), "Should list diameter option");
    assert!(stdout.contains("--path <s1> <s2>"), "Should list path option");
    assert!(stdout.contains("--print"), "Should list print option");
    assert!(stdout.contains("--no-midpoints"), "Should list no-midpoints option");
    assert!(
        stdout.contains("--show-vertex-count"),
        "Should list vertex count option"
    );
    assert!(stdout.contains("--show-path"), "Should list show-path option");
}

#[test]
fn test_short_help_flag_prints_help_message() {
    let (code, stdout, _stderr) = run_cavegraph(&["-h"]);

    assert_eq!(code, 0, "Should exit successfully");
    assert!(stdout.contains("Usage: cavegraph"), "Should print usage header");
}

#[test]
fn test_walls_project_diameter() {
    let (code, stdout, _stderr) = run_cavegraph(&["data/Walls/MCSVY.wpj", "--diameter"]);

    assert_eq!(code, 0, "Should exit successfully");
    assert!(stdout.contains("Graph diameter is"), "Should output diameter result");
}
