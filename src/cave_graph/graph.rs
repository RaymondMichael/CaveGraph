use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

struct Vertex {
    name : String,
    distance: f64,
}

impl Ord for Vertex {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.distance < other.distance {
            Ordering::Less
        } else if self.distance > other.distance {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for Vertex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Vertex {}
/*
    struct VertexTracker {
        vertex: <Rc<Vertex>>,
        distance: f64
    }

    impl Ord for VertexTracker {
        fn cmp(&self, other: &Self) -> Ordering {
            if self.distance < other.distance {
                Ordering::Less
            } else if self.distance > other.distance {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }
    }

    impl PartialOrd for VertexTracker {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl PartialEq for VertexTracker {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
        }
    }

    impl Eq for VertexTracker {}
*/
struct Edge {
    p0 : Rc<RefCell<Vertex>>,
    p1 : Rc<RefCell<Vertex>>,
    distance: f32,
}

impl Edge {
    fn other_vert(&self, vertex: &Rc<RefCell<Vertex>>) -> Rc<RefCell<Vertex>> {
        if self.p0.borrow().name == vertex.borrow().name {
            self.p1.clone()
        } else {
            self.p0.clone()
        }
    }
}

pub struct MapGraph {
    vertices: HashMap<String, Rc<RefCell<Vertex>>>,
    edges: HashMap<(String, String), Rc<Edge>>,
}

impl MapGraph {
    pub fn new() -> MapGraph {
        MapGraph {vertices: HashMap::new(), edges: HashMap::new()}
    }

    pub fn insert_vertices(&mut self, verts: &[&str]) {
        for i in 0..verts.len() {
            //println!("{}", verts[i]);

            let v: Rc<RefCell<Vertex>> = RefCell::new(Vertex {
                name: String::from(verts[i]),
                distance: 99999999999.9
            }).into();
            let name = v.borrow_mut().name.clone();

            self.vertices.insert(name, v);
        }
    }

    pub fn insert_edges(&mut self, edges: &[(&str, &str, f32)]) {
        for i in 0..edges.len() {
            let (p0, p1, d) = edges[i];
            //println!("{} {}", p0, p1);

            let v0 = self.vertices.get(p0).
                expect("Couldn't find the first vertex");
            let v1 = self.vertices.get(p1).
                expect("Couldn't find the second vertex");

            let e = Edge {p0: v0.clone(), p1: v1.clone(), distance: d};
            self.edges.insert((p0.to_string(), p1.to_string()), e.into());
        }
    }

    fn find_edges(&self, name: &String) -> Vec<Rc<Edge>> {
        let mut v: Vec<Rc<Edge>> = Vec::new();

        for (_, e) in self.edges.iter() {
            if e.p0.borrow().name == *name || e.p1.borrow().name == *name {
                v.push(e.clone());
            }
        }

        v
    }

    pub fn shortest_path(&mut self, start: &String, finish: &String) -> f64 {
        let start_v = self.vertices.get(start).
            expect("Couldn't find the starting vertex");
        let end_v = self.vertices.get(finish).
            expect("Couldn't find the ending vertex");

        let mut unvisited: Vec<Rc<RefCell<Vertex>>> = Vec::new();
        for (_, v) in self.vertices.iter() {
            v.borrow_mut().distance = 999999999999.9;
            unvisited.push(v.clone());
        }
        start_v.borrow_mut().distance = 0.0;

        unvisited.sort();
        for _i in 0..unvisited.len() {
            let v = unvisited.remove(0);
            if v == *end_v {break;}
            let edges = self.find_edges(&v.borrow().name);
            for e in edges.iter() {
                let v_other = e.other_vert(&v);
                let new_dist = v.borrow().distance + f64::from(e.distance);
                if v_other.borrow().distance > new_dist {
                    v_other.borrow_mut().distance = new_dist;
                }
            }
            unvisited.sort();
        }
        
        end_v.borrow().distance
    }
}
