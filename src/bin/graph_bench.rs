use cavegraph::cave_graph::graph::MapGraph;
use std::collections::HashSet;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug)]
enum Topology {
    Chain,
    Tree,
    Sparse,
    Medium,
    Dense,
}

impl Topology {
    fn as_str(self) -> &'static str {
        match self {
            Topology::Chain => "chain",
            Topology::Tree => "tree",
            Topology::Sparse => "sparse",
            Topology::Medium => "medium",
            Topology::Dense => "dense",
        }
    }

    fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "chain" => Ok(Topology::Chain),
            "tree" => Ok(Topology::Tree),
            "sparse" => Ok(Topology::Sparse),
            "medium" => Ok(Topology::Medium),
            "dense" => Ok(Topology::Dense),
            _ => Err(format!("Unknown topology: {}", value)),
        }
    }
}

#[derive(Clone, Copy)]
struct Case {
    topology: Topology,
    vertices: usize,
    edges: usize,
}

struct Config {
    matrix: bool,
    topology: Option<Topology>,
    vertices: Option<usize>,
    edges: Option<usize>,
    seed: u64,
    repeats: usize,
    csv: Option<PathBuf>,
    no_midpoints: Option<bool>,
}

impl Config {
    fn parse() -> Result<Self, String> {
        let mut matrix = false;
        let mut topology: Option<Topology> = None;
        let mut vertices: Option<usize> = None;
        let mut edges: Option<usize> = None;
        let mut seed: u64 = 1;
        let mut repeats: usize = 5;
        let mut csv: Option<PathBuf> = None;
        let mut no_midpoints: Option<bool> = None;

        let args: Vec<String> = env::args().collect();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--matrix" => {
                    matrix = true;
                    i += 1;
                }
                "--topology" => {
                    if i + 1 >= args.len() {
                        return Err("--topology requires a value".to_string());
                    }
                    topology = Some(Topology::from_str(&args[i + 1])?);
                    i += 2;
                }
                "--vertices" => {
                    if i + 1 >= args.len() {
                        return Err("--vertices requires a value".to_string());
                    }
                    vertices = Some(
                        args[i + 1]
                            .parse::<usize>()
                            .map_err(|_| "Invalid --vertices value".to_string())?,
                    );
                    i += 2;
                }
                "--edges" => {
                    if i + 1 >= args.len() {
                        return Err("--edges requires a value".to_string());
                    }
                    edges = Some(
                        args[i + 1]
                            .parse::<usize>()
                            .map_err(|_| "Invalid --edges value".to_string())?,
                    );
                    i += 2;
                }
                "--seed" => {
                    if i + 1 >= args.len() {
                        return Err("--seed requires a value".to_string());
                    }
                    seed = args[i + 1]
                        .parse::<u64>()
                        .map_err(|_| "Invalid --seed value".to_string())?;
                    i += 2;
                }
                "--repeats" => {
                    if i + 1 >= args.len() {
                        return Err("--repeats requires a value".to_string());
                    }
                    repeats = args[i + 1]
                        .parse::<usize>()
                        .map_err(|_| "Invalid --repeats value".to_string())?;
                    if repeats == 0 {
                        return Err("--repeats must be greater than 0".to_string());
                    }
                    i += 2;
                }
                "--csv" => {
                    if i + 1 >= args.len() {
                        return Err("--csv requires a value".to_string());
                    }
                    csv = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                }
                "--no-midpoints" => {
                    no_midpoints = Some(true);
                    i += 1;
                }
                "--with-midpoints" => {
                    no_midpoints = Some(false);
                    i += 1;
                }
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                other => {
                    return Err(format!("Unknown option: {}", other));
                }
            }
        }

        if !matrix && (topology.is_none() || vertices.is_none()) {
            return Err(
                "Single-case mode requires --topology and --vertices (or use --matrix)".to_string(),
            );
        }

        Ok(Self {
            matrix,
            topology,
            vertices,
            edges,
            seed,
            repeats,
            csv,
            no_midpoints,
        })
    }
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9e3779b97f4a7c15,
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn next_usize(&mut self, upper: usize) -> usize {
        if upper <= 1 {
            return 0;
        }
        (self.next_u64() as usize) % upper
    }
}

fn print_usage() {
    println!(
        "Usage:\n  cargo run --bin graph_bench -- --matrix [--repeats N] [--csv FILE]\n  cargo run --bin graph_bench -- --topology chain|tree|sparse|medium|dense --vertices N [--edges N] [--seed N] [--repeats N] [--csv FILE] [--no-midpoints|--with-midpoints]"
    );
}

fn matrix_cases() -> Vec<Case> {
    let tiers = [100_usize, 300, 700, 1200, 2000];
    let mut cases = Vec::new();

    for v in tiers {
        cases.push(Case {
            topology: Topology::Chain,
            vertices: v,
            edges: v.saturating_sub(1),
        });
        cases.push(Case {
            topology: Topology::Tree,
            vertices: v,
            edges: v.saturating_sub(1),
        });
        cases.push(Case {
            topology: Topology::Sparse,
            vertices: v,
            edges: v.saturating_mul(12) / 10,
        });
        cases.push(Case {
            topology: Topology::Medium,
            vertices: v,
            edges: v.saturating_mul(14) / 10,
        });

        if v >= 700 {
            cases.push(Case {
                topology: Topology::Dense,
                vertices: v,
                edges: v * 2,
            });
        }
    }

    cases
}

fn add_edge(
    a: usize,
    b: usize,
    weight: f32,
    seen: &mut HashSet<(usize, usize)>,
    edges: &mut Vec<(usize, usize, f32)>,
) -> bool {
    if a == b {
        return false;
    }
    let (x, y) = if a < b { (a, b) } else { (b, a) };
    if seen.insert((x, y)) {
        edges.push((x, y, weight));
        true
    } else {
        false
    }
}

fn max_undirected_edges(vertices: usize) -> usize {
    vertices.saturating_mul(vertices.saturating_sub(1)) / 2
}

fn build_case(case: Case, seed: u64) -> MapGraph {
    let mut graph = MapGraph::new();

    let names: Vec<String> = (0..case.vertices).map(|i| format!("v{}", i)).collect();
    let name_refs: Vec<&str> = names.iter().map(String::as_str).collect();
    graph.insert_vertices(&name_refs);

    let mut edges: Vec<(usize, usize, f32)> = Vec::new();
    let mut seen: HashSet<(usize, usize)> = HashSet::new();

    match case.topology {
        Topology::Chain => {
            for i in 0..case.vertices.saturating_sub(1) {
                add_edge(i, i + 1, 1.0, &mut seen, &mut edges);
            }
        }
        Topology::Tree => {
            for i in 1..case.vertices {
                let parent = (i - 1) / 2;
                add_edge(i, parent, 1.0, &mut seen, &mut edges);
            }
        }
        Topology::Sparse | Topology::Medium | Topology::Dense => {
            for i in 0..case.vertices.saturating_sub(1) {
                add_edge(i, i + 1, 1.0, &mut seen, &mut edges);
            }

            let target = case
                .edges
                .min(max_undirected_edges(case.vertices))
                .max(case.vertices.saturating_sub(1));
            let mut rng = Lcg::new(seed);
            let mut tries = 0_usize;
            let max_tries = target.saturating_mul(20).max(1000);
            while edges.len() < target && tries < max_tries {
                let a = rng.next_usize(case.vertices);
                let b = rng.next_usize(case.vertices);
                let w = (1 + (rng.next_u64() % 10)) as f32;
                add_edge(a, b, w, &mut seen, &mut edges);
                tries += 1;
            }
        }
    }

    let edge_names: Vec<(String, String, f32)> = edges
        .iter()
        .map(|(a, b, w)| (names[*a].clone(), names[*b].clone(), *w))
        .collect();
    let edge_refs: Vec<(&str, &str, f32)> = edge_names
        .iter()
        .map(|(a, b, w)| (a.as_str(), b.as_str(), *w))
        .collect();

    graph.insert_edges(&edge_refs);
    graph
}

fn percentile_index(len: usize, p: f64) -> usize {
    let rank = (p * (len as f64)).ceil() as usize;
    rank.saturating_sub(1).min(len.saturating_sub(1))
}

fn summarize_ms(durations: &[Duration]) -> (u128, u128, u128, u128) {
    let mut values: Vec<u128> = durations.iter().map(Duration::as_millis).collect();
    values.sort_unstable();
    let min = values[0];
    let max = values[values.len() - 1];
    let median = values[values.len() / 2];
    let p95 = values[percentile_index(values.len(), 0.95)];
    (median, p95, min, max)
}

fn unix_timestamp() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => 0,
    }
}

fn append_csv_rows(
    path: &PathBuf,
    case: Case,
    seed: u64,
    no_midpoints: bool,
    durations: &[Duration],
    median_ms: u128,
    p95_ms: u128,
    min_ms: u128,
    max_ms: u128,
) -> Result<(), String> {
    let needs_header = match std::fs::metadata(path) {
        Ok(meta) => meta.len() == 0,
        Err(_) => true,
    };

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("Failed to open CSV file: {}", e))?;

    if needs_header {
        writeln!(
            file,
            "timestamp,topology,vertices,edges,seed,no_midpoints,repeat_index,duration_ms,median_ms,p95_ms,min_ms,max_ms"
        )
        .map_err(|e| format!("Failed to write CSV header: {}", e))?;
    }

    let ts = unix_timestamp();
    for (idx, d) in durations.iter().enumerate() {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            ts,
            case.topology.as_str(),
            case.vertices,
            case.edges,
            seed,
            no_midpoints,
            idx,
            d.as_millis(),
            median_ms,
            p95_ms,
            min_ms,
            max_ms
        )
        .map_err(|e| format!("Failed to write CSV row: {}", e))?;
    }

    Ok(())
}

fn run_case(case: Case, repeats: usize, seed: u64, no_midpoints: bool, csv: Option<&PathBuf>) {
    let graph = build_case(case, seed);

    let mut durations = Vec::with_capacity(repeats);
    let mut last_result = (String::new(), String::new(), 0.0_f64);
    for _ in 0..repeats {
        let start = Instant::now();
        last_result = graph.diameter(no_midpoints);
        durations.push(start.elapsed());
    }

    let (median_ms, p95_ms, min_ms, max_ms) = summarize_ms(&durations);

    println!(
        "topology={} V={} E={} seed={} no_midpoints={} repeats={} median_ms={} p95_ms={} min_ms={} max_ms={} diameter={} {} -> {}",
        case.topology.as_str(),
        case.vertices,
        case.edges,
        seed,
        no_midpoints,
        repeats,
        median_ms,
        p95_ms,
        min_ms,
        max_ms,
        last_result.2,
        last_result.0,
        last_result.1
    );

    if let Some(path) = csv {
        if let Err(err) = append_csv_rows(
            path,
            case,
            seed,
            no_midpoints,
            &durations,
            median_ms,
            p95_ms,
            min_ms,
            max_ms,
        ) {
            eprintln!("{}", err);
        }
    }
}

fn mode_values(no_midpoints: Option<bool>) -> Vec<bool> {
    match no_midpoints {
        Some(value) => vec![value],
        None => vec![false, true],
    }
}

fn main() {
    let config = match Config::parse() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            print_usage();
            std::process::exit(1);
        }
    };

    if config.matrix {
        for case in matrix_cases() {
            for no_midpoints in mode_values(config.no_midpoints) {
                for seed in [1_u64, 2, 3] {
                    run_case(case, config.repeats, seed, no_midpoints, config.csv.as_ref());
                }
            }
        }
    } else {
        let topology = match config.topology {
            Some(t) => t,
            None => {
                eprintln!("Missing --topology");
                std::process::exit(1);
            }
        };
        let vertices = match config.vertices {
            Some(v) => v,
            None => {
                eprintln!("Missing --vertices");
                std::process::exit(1);
            }
        };

        let default_edges = match topology {
            Topology::Chain | Topology::Tree => vertices.saturating_sub(1),
            Topology::Sparse => vertices.saturating_mul(12) / 10,
            Topology::Medium => vertices.saturating_mul(14) / 10,
            Topology::Dense => vertices * 2,
        };

        let case = Case {
            topology,
            vertices,
            edges: config.edges.unwrap_or(default_edges),
        };

        for no_midpoints in mode_values(config.no_midpoints) {
            run_case(case, config.repeats, config.seed, no_midpoints, config.csv.as_ref());
        }
    }
}
