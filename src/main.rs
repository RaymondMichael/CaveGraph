use shortest_path::cave_graph::cave;
use shortest_path::cave_graph::cave::Cave;
use shortest_path::cave_graph::graph::MapGraph;
use std::env;
use std::path::Path;
use std::process;

struct Config {
    file_path: String,
    calculate_diameter: bool,
    shortest_path_stations: Option<(String, String)>,
    print_cave: bool,
}

fn parse_arguments() -> Result<Config, String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err("Usage: shortest_path <file.th|file.wpj> [--diameter] [--path station1 station2] [--print]".to_string());
    }

    let file_path = args[1].clone();

    // Validate file exists
    if !Path::new(&file_path).exists() {
        return Err(format!("Error: File not found: {}", file_path));
    }

    // Validate file extension
    let extension = Path::new(&file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    if extension != "th" && extension != "wpj" {
        return Err(format!("Error: Invalid file extension. Expected .th or .wpj, got .{}", extension));
    }

    let mut calculate_diameter = false;
    let mut shortest_path_stations: Option<(String, String)> = None;
    let mut print_cave = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--diameter" => {
                calculate_diameter = true;
                i += 1;
            }
            "--path" => {
                if i + 2 >= args.len() {
                    return Err("Error: --path requires exactly two station names".to_string());
                }
                shortest_path_stations = Some((args[i + 1].clone(), args[i + 2].clone()));
                i += 3;
            }
            "--print" => {
                print_cave = true;
                i += 1;
            }
            _ => {
                return Err(format!("Error: Unknown option: {}", args[i]));
            }
        }
    }

    Ok(Config {
        file_path,
        calculate_diameter,
        shortest_path_stations,
        print_cave,
    })
}

fn main() {
    let config = match parse_arguments() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    // Determine file type and read cave
    let mut cave = Cave::new();
    let extension = Path::new(&config.file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension {
        "th" => {
            cave::therion_reader::read_therion(
                &mut cave, &"".to_string(), &config.file_path);
        }
        "wpj" => {
            let mut directory = Path::new(&config.file_path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string());
            if !directory.ends_with('/') {
                directory.push('/');
            }
            let filename = Path::new(&config.file_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| config.file_path.clone());

            cave::walls_reader::read_walls(&mut cave, &directory, &filename);
        }
        _ => {
            eprintln!("Error: Unsupported file type: {}", extension);
            process::exit(1);
        }
    }

    // Create graph from cave
    let graph = MapGraph::cave_graph(&cave);

    // Print cave details if requested
    if config.print_cave {
        cave.print();
    }

    // Calculate and output shortest path if requested
    if let Some((station1, station2)) = config.shortest_path_stations {
        let distance = graph.shortest_path(&station1, &station2);
        println!("Shortest distance is {}", distance);
    }

    // Calculate and output diameter if requested
    if config.calculate_diameter {
        let (start, end, distance) = graph.diameter();
        println!("Graph diameter is {} between stations {} and {}", distance, start, end);
    }
}
