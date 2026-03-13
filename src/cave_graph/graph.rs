use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;
use super::cave::Cave;

struct Vertex {
    name: String
}

impl Vertex {
    pub fn new(title: String) -> Vertex {
        Vertex {name: title}
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Vertex {}

struct VertexTracker {
    vertex: Rc<Vertex>,
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
        self.vertex == other.vertex
    }
}

impl Eq for VertexTracker {}

struct Edge {
    p0 : Rc<Vertex>,
    p1 : Rc<Vertex>,
    distance: f32,
}

/*
 * When passed a vertex which the edge is connected to, return the other
 * vertex the edge is connected to
 */
impl Edge {
    fn other_vert(&self, vertex: &Rc<Vertex>) -> Rc<Vertex> {
        if self.p0.name == vertex.name {
            self.p1.clone()
        } else {
            self.p0.clone()
        }
    }
}

pub struct MapGraph {
    vertices: HashMap<String, Rc<Vertex>>,
    edges: HashMap<(String, String), Rc<Edge>>,
}

impl MapGraph {
    pub fn new() -> MapGraph {
        MapGraph {vertices: HashMap::new(), edges: HashMap::new()}
    }

    /*
     * Add the vertex if the name is not already in the set of vertices.
     * Return the vertex.
     */
    fn insert_vertex(&mut self, name: &String) -> Rc<Vertex> {
        let v = match self.vertices.get(name) {
            Some(v) => v.clone(),
            None => {
                let v: Rc<Vertex> = Rc::new(Vertex::new(name.clone()));
                self.vertices.insert(name.clone(), v.clone());
                v
            }
        };

        v
    }

    /*
     * Take an array of vertex names and add them as vertex objects
     */
    pub fn insert_vertices(&mut self, verts: &[&str]) {
        for i in 0..verts.len() {
            //println!("{}", verts[i]);

            self.insert_vertex(&String::from(verts[i]));
        }
    }

    /*
     * Take an array of edge descriptions, turn them into edge objects that
     * reference the named vertex objects, and add them to the graph
     */
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

    /*
     * Return a vector of all the edges that connect to the named vertex
     */
    fn find_edges(&self, name: &String) -> Vec<Rc<Edge>> {
        let mut v: Vec<Rc<Edge>> = Vec::new();

        for (_, e) in self.edges.iter() {
            if e.p0.name == *name || e.p1.name == *name {
                v.push(e.clone());
            }
        }

        v
    }

    /*
     * Take a cave object and turn it into a graph we can analyze
     */
    pub fn cave_graph(cave: &Cave) -> MapGraph {
        let mut graph = MapGraph::new();

        /*
         * For each equality, create a hashmap entry for each of the station
         * names to the same vertex object
         */
        for eq in cave.equalities.iter() {
            let v0 = eq.v0();
            let v1 = eq.v1();
            match graph.vertices.get(&v0) {
                Some(v) => {
                    graph.vertices.insert(v1, v.clone());
                },
                None => {
                    let v: Rc<Vertex> = Rc::new(
                        Vertex::new(v0.clone()));
                    graph.vertices.insert(v0, v.clone());
                    graph.vertices.insert(v1, v);
                }
            }
        }

        /* For each book for each shot, create an edge */
        for book in cave.books.iter() {
            for s in book.shots.iter() {
                let ((s0, s1), shot) = s;
                let stat0 = format!("{}@{}", s0.name, book.title);
                let v0 = graph.insert_vertex(&stat0);
                let stat1 = format!("{}@{}", s1.name, book.title);
                let v1 = graph.insert_vertex(&stat1);

                let e = Edge {
                    p0: v0.clone(),
                    p1: v1.clone(),
                    distance: shot.length
                };
                graph.edges.insert(
                    (v0.name.clone(), v1.name.clone()),
                    e.into());
            }
        }

        graph
    }

    /*
     * Return the shortest distance between the two named vertices
     */
    pub fn shortest_path(&self, start: &String, finish: &String) -> f64 {
        let start_v = self.vertices.get(start).
            expect("Couldn't find the starting vertex");
        let end_v = self.vertices.get(finish).
            expect("Couldn't find the ending vertex");

        let mut unvisited: Vec<Rc<RefCell<VertexTracker>>> = Vec::new();
        let mut vt_lookup: HashMap<String, Rc<RefCell<VertexTracker>>> = HashMap::new();
        for (_, v) in self.vertices.iter() {
            let vt: Rc<RefCell<VertexTracker>> = RefCell::new(VertexTracker {
                distance: 999999999999.9,
                vertex: v.clone()
            }).into();
            if vt.borrow().vertex == *start_v {
                vt.borrow_mut().distance = 0.0;
            }
            unvisited.push(vt.clone());
            let name: String = vt.borrow().vertex.name.clone();
            vt_lookup.insert(name, vt);
        }

        unvisited.sort();
        for _i in 0..unvisited.len() {
            let vt = unvisited.remove(0);
            if vt.borrow().vertex == *end_v {break;}
            let edges = self.find_edges(&vt.borrow().vertex.name);
            for e in edges.iter() {
                let v_other = e.other_vert(&vt.borrow().vertex);
                let vt_other = vt_lookup.get(&v_other.name).
                    expect("Couldn't find the matching VT");
                let new_dist = vt.borrow().distance + f64::from(e.distance);
                if vt_other.borrow().distance > new_dist {
                    vt_other.borrow_mut().distance = new_dist;
                }
            }
            unvisited.sort();
        }

        let end_vt = vt_lookup.get(&end_v.name).
            expect("Couldn't find the matching ending VT");

        end_vt.borrow().distance
    }

    /*
     * Return the two vertices that are the furthest apart from each other,
     * and the distance between them
     */
    pub fn diameter(&self) -> (String, String, f64) {
        let mut longest_distance: f64 = 0.0;
        let mut longest_start = String::new();
        let mut longest_end = String::new();

        for (start_name, _) in self.vertices.iter() {
            for (end_name, _) in self.vertices.iter() {
                if start_name == end_name {continue;}
                let distance = self.shortest_path(&start_name, &end_name);
                if distance > longest_distance {
                    longest_distance = distance;
                    longest_start = start_name.clone();
                    longest_end = end_name.clone();
                }
            }
        }

        (longest_start, longest_end, longest_distance)
    }
}
