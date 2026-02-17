use std::collections::HashMap;
use std::rc::Rc;

#[derive(PartialEq, Eq, Hash)]
struct Vertex {
    name : String,
}

struct Edge {
    p0 : Rc<Vertex>,
    p1 : Rc<Vertex>,
    distance: f32,
}

struct MapGraph {
    vertices: HashMap<String, Rc<Vertex>>,
    edges: HashMap<(String, String), Rc<Edge>>,
}

impl MapGraph {
    fn new() -> MapGraph {
        MapGraph {vertices: HashMap::new(), edges: HashMap::new()}
    }

    fn insert_vertices(&mut self, verts: &[&str]) {
        for i in 0..verts.len() {
            println!("{}", verts[i]);

            let v: Rc<Vertex> = Vertex {
                name: String::from(verts[i]),
            }.into();

            self.vertices.insert(v.name.clone(), v);
        }
    }

    fn insert_edges(&mut self, edges: &[(&str, &str, f32)]) {
        for i in 0..edges.len() {
            let (p0, p1, d) = edges[i];
            println!("{} {}", p0, p1);

            let v0 = self.vertices.get(p0).
                expect("Couldn't find the first vertex");
            let v1 = self.vertices.get(p1).
                expect("Couldn't find the second vertex");

            let e = Edge {p0: v0.clone(), p1: v1.clone(), distance: d};
            self.edges.insert((p0.to_string(), p1.to_string()), e.into());
        }
    }

    fn find_edges(self, name: String) -> Vec<Rc<Edge>> {
        let mut v: Vec<Rc<Edge>> = Vec::new();

        for (_, e) in self.edges.iter() {
            if e.p0.name == name || e.p1.name == name {
                v.push(e.clone());
            }
        }

        v
    }
}

fn main() {
    let input_verts: [&str; 3] = ["A", "B", "C"];
    let input_edges: [(&str, &str, f32); 3] = [
        ("A", "B", 3.0), ("A", "C", 1.0), ("C", "B", 1.5)
    ];
    let mut graph = MapGraph::new();

    graph.insert_vertices(&input_verts);
    graph.insert_edges(&input_edges);

    let list = graph.find_edges("C".to_string());
    for e in list.iter() {
        println!("Lengths are {}", e.distance);
    }
}
