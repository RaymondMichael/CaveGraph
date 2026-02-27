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

    let distance = graph.shortest_path("A".to_string(), "B".to_string());
    println!("Shortest distance is {}", distance);
}
