# CaveGraph

CaveGraph is a small Rust command-line tool for loading cave survey data, turning it into a graph, and measuring distances through the surveyed network.

The binary name is `cavegraph`.

## What it does

- Reads Therion survey files (`.th`)
- Reads Walls project files (`.wpj`)
- Builds a graph where stations are vertices and shots are weighted edges
- Computes the shortest path between two stations
- Computes the graph diameter, optionally restricted to endpoint stations
- Prints the parsed cave data for inspection

## Supported input

### Therion

Point the tool at a `.th` file. The parser follows nested `input` directives and ignores `.th2` files.

Example:

```bash
cargo run -- data/HMaze.th --diameter
```

### Walls

Point the tool at a `.wpj` file. The parser reads the project file, discovers the referenced survey books, and loads the corresponding `.SRV` files.

Example:

```bash
cargo run -- data/Walls/MCSVY.wpj --diameter
```

## Build

```bash
cargo build
```

For an optimized binary:

```bash
cargo build --release
```

## Usage

```text
cavegraph <file.th|file.wpj> [--diameter] [--no-midpoints] [--path station1 station2] [--print]
```

If you run the program without any reporting flag, it parses the input and exits without printing anything.

## Options

- `--diameter`: Compute the longest shortest path in the graph
- `--no-midpoints`: When used with `--diameter`, only consider endpoint stations (vertices with degree 1)
- `--path station1 station2`: Compute the shortest path distance between two named stations
- `--print`: Print the parsed cave structure, including books, stations, and shots

## Station names

Station names must match the graph vertex names used internally:

- Therion stations are typically addressed as `station@survey`
- Walls stations are typically addressed as `station@prefix`

Examples from the bundled sample data:

- `M9@HMaze`
- `M16@HMaze`
- `XB10@156`

## Examples

Compute the diameter of the bundled Therion sample:

```bash
cargo run -- data/HMaze.th --diameter
```

Observed output:

```text
Graph diameter is 134.19999980926514 between stations M9@HMaze and M16@HMaze
```

Compute the shortest path between two known stations in that same sample:

```bash
cargo run -- data/HMaze.th --path M9@HMaze M16@HMaze
```

Observed output:

```text
Shortest distance is 134.19999980926514
```

Compute the diameter of the bundled Walls sample:

```bash
cargo run -- data/Walls/MCSVY.wpj --diameter
```

Observed output:

```text
Graph diameter is 236.4500026702881 between stations XB10@156 and XB13@156
```

Print the parsed cave structure:

```bash
cargo run -- data/HMaze.th --print
```

## Project layout

- `src/main.rs`: CLI argument parsing and program entry point
- `src/cave_graph/cave/therion_reader.rs`: Therion parser
- `src/cave_graph/cave/walls_reader.rs`: Walls parser
- `src/cave_graph/graph.rs`: Graph construction and path algorithms
- `data/`: Sample Therion and Walls survey data

## Current limitations

- The CLI does not expose a `--help` flag; invalid usage prints a usage string and exits with an error
- Unknown or missing stations in `--path` return a user-facing error and exit with status 1
- Distances are printed as raw floating-point values
- Diameter calculation uses repeated shortest-path searches and may be slow on large surveys

## License

See `LICENSE`.