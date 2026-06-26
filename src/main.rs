use cavegraph::cave_graph::cave;
use cavegraph::cave_graph::cave::Cave;
use cavegraph::cave_graph::graph::MapGraph;
use std::env;
use std::path::Path;
use std::process;

const HELP_MESSAGE: &str = "Usage: cavegraph <file.th|file.wpj> [options]\n\nOptions:\n  --help, -h            Show this help message\n  --diameter            Calculate graph diameter\n  --path <s1> <s2>      Calculate shortest path between two stations\n  --print               Print parsed cave structure\n  --stats               Print total edge length and vertex count\n  --no-midpoints        Ignore midpoint stations in diameter calculation\n  --show-vertex-count   Show number of vertices on reported path\n  --show-path           Print full station path";

struct Config {
    file_path: String,
    calculate_diameter: bool,
    shortest_path_stations: Option<(String, String)>,
    print_cave: bool,
    show_stats: bool,
    no_midpoints: bool,
    show_vertex_count: bool,
    show_path: bool,
}

fn parse_arguments() -> Result<Option<Config>, String> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 && (args[1] == "--help" || args[1] == "-h") {
        return Ok(None);
    }

    if args.len() < 2 {
        return Err(HELP_MESSAGE.to_string());
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
    let mut show_stats = false;
    let mut no_midpoints = false;
    let mut show_vertex_count = false;
    let mut show_path = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                return Ok(None);
            }
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
            "--stats" => {
                show_stats = true;
                i += 1;
            }
            "--no-midpoints" => {
                no_midpoints = true;
                i += 1;
            }
            "--show-vertex-count" => {
                show_vertex_count = true;
                i += 1;
            }
            "--show-path" => {
                show_path = true;
                i += 1;
            }
            _ => {
                return Err(format!("Error: Unknown option: {}", args[i]));
            }
        }
    }

    Ok(Some(Config {
        file_path,
        calculate_diameter,
        shortest_path_stations,
        print_cave,
        show_stats,
        no_midpoints,
        show_vertex_count,
        show_path,
    }))
}

fn print_path_details(path_result: &cavegraph::cave_graph::graph::PathResult, show_vertex_count: bool, show_path: bool) {
    if show_vertex_count {
        println!("Vertices on path: {}", path_result.path.len());
    }

    if show_path {
        println!("Path: {}", path_result.path.join(" -> "));
    }
}

fn main() {
    let config = match parse_arguments() {
        Ok(Some(cfg)) => cfg,
        Ok(None) => {
            println!("{}", HELP_MESSAGE);
            return;
        }
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
    let graph = match MapGraph::cave_graph(&cave) {
        Ok(graph) => graph,
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    };

    // Print cave details if requested
    if config.print_cave {
        cave.print();
    }

    if config.show_stats {
        println!("Total edge length: {}", graph.total_edge_length());
        println!("Total vertices: {}", graph.vertex_count());
    }

    // Calculate and output shortest path if requested
    if let Some((station1, station2)) = config.shortest_path_stations {
        let path_result = match graph.shortest_path(&station1, &station2) {
            Ok(path_result) => path_result,
            Err(err) => {
                eprintln!("Error: {}", err);
                process::exit(1);
            }
        };
        println!("Shortest distance is {}", path_result.distance);
        print_path_details(&path_result, config.show_vertex_count, config.show_path);
    }

    // Calculate and output diameter if requested
    if config.calculate_diameter {
        let path_result = graph.diameter(config.no_midpoints);
        println!(
            "Graph diameter is {} between stations {} and {}",
            path_result.distance,
            path_result.start,
            path_result.end
        );
        print_path_details(&path_result, config.show_vertex_count, config.show_path);
    }
}
