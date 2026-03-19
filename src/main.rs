use shortest_path::cave_graph::cave;
use shortest_path::cave_graph::cave::Cave;
use shortest_path::cave_graph::graph::MapGraph;

fn main() {
    let input_verts: [&str; 6] = ["A", "B", "C", "D", "E", "F"];
    let input_edges: [(&str, &str, f32); 8] = [
        ("A", "B", 3.0), ("B", "D", 15.0), ("A", "C", 1.0), ("C", "B", 1.5),
        ("A", "E", 0.5), ("E", "B", 7.0), ("F", "E", 1.0), ("F", "B", 0.3),
    ];
    let mut graph = MapGraph::new();

    graph.insert_vertices(&input_verts);
    graph.insert_edges(&input_edges);

    let distance = graph.shortest_path(&"A".to_string(), &"B".to_string());
    println!("Shortest distance is {}", distance);

    /* Read in a Therion-based cave */
    let mut cave = Cave::new();
    cave::therion_reader::read_therion(
        &mut cave, &"".to_string(), &"data/Deep_Lake/deep_lake.th".to_string());
        //&mut cave, &"".to_string(), &"HMaze.th".to_string());
    //cave.print();

    let graph = MapGraph::cave_graph(&cave);
    let distance = graph.shortest_path(
        &"a3@a".to_string(), &"b26@b".to_string());
    println!("Shortest distance is {}", distance);

    let (start, end, distance) = graph.diameter();
    println!("Graph diameter is {} between stations {} and {}",
             distance, start, end);

    /* Read in a Walls-based cave */
    let mut cave = Cave::new();
    cave::walls_reader::read_walls(
        &mut cave, &"data/Walls/".to_string(), &"MCSVY.wpj".to_string());
    cave.print();
}
