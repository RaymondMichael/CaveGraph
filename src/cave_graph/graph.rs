use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::rc::Rc;
//use std::time::SystemTime;
use super::cave::Cave;

struct Edge {
    other : Rc<RefCell<Vertex>>,
    distance: f64
}

struct Vertex {
    name: String,
    edges: Vec<Edge>
}

impl Vertex {
    pub fn new(title: String) -> Vertex {
        Vertex {
            name: title,
            edges: Vec::new()
        }
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Vertex {}

struct VertexTracker {
    name: String,
    distance: f64
}

impl Ord for VertexTracker {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering so BinaryHeap behaves as a min-priority queue.
        other
            .distance
            .total_cmp(&self.distance)
            .then_with(|| other.name.cmp(&self.name))
    }
}

impl PartialOrd for VertexTracker {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for VertexTracker {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.distance == other.distance
    }
}

impl Eq for VertexTracker {}

pub struct MapGraph {
    vertices: HashMap<String, Rc<RefCell<Vertex>>>,
}

impl MapGraph {
    const CANARY: f64 = 999999999999.9;

    pub fn new() -> MapGraph {
        MapGraph {vertices: HashMap::new()}
    }

    /*
     * Add the vertex if the name is not already in the set of vertices.
     * Return the vertex.
     */
    fn insert_vertex(&mut self, name: &String) -> Rc<RefCell<Vertex>> {
        let v = match self.vertices.get(name) {
            Some(v) => v.clone(),
            None => {
                let v: Rc<RefCell<Vertex>> = RefCell::new(Vertex::new(name.clone())).into();
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

            let v0 = self.vertices.get(p0).
                expect("Couldn't find the first vertex");
            let v1 = self.vertices.get(p1).
                expect("Couldn't find the second vertex");

            let e1 = Edge {
                other: v1.clone(),
                distance: f64::from(d)
            };
            v0.borrow_mut().edges.push(e1);

            let e0 = Edge {
                other: v0.clone(),
                distance: f64::from(d)
            };
            v1.borrow_mut().edges.push(e0);
        }
    }

    /*
     * In self.vertices we want to change all references to v1_obj to
     * reference v0_obj instead
     */
    fn move_equalities(&mut self, v0_obj: Rc<RefCell<Vertex>>, v1_obj: Rc<RefCell<Vertex>>) {
        let mut v: Vec<(String, Rc<RefCell<Vertex>>)> = Vec::new();

        /* Collect all the mappings we need to move */
        for (name, obj) in self.vertices.iter() {
            if *obj == v1_obj {
                v.push((name.clone(), v0_obj.clone()));
            }
        }

        /* Insert them now that self.vertices is mutable */
        for (name, obj) in v.iter() {
            self.vertices.insert(name.clone(), obj.clone());
        }
    }

    /*
     * This handles adding a single station equality when setting up a new
     * MapGraph. We check whether the two referenced stations are already in
     * the set of vertices already, and act accordingly. The most complicated
     * situation is when they both already exist, and so all the data about
     * one needs to be moved to the other.
     */
    fn insert_equality(&mut self, v0: String, v1: String) {
        let m0 = self.vertices.get(&v0);
        let m1 = self.vertices.get(&v1);

        match m0 {
            Some(va) => {
                match m1 {
                    Some(vb) => {
                        self.move_equalities(va.clone(), vb.clone());
                    },
                    None => {
                        self.vertices.insert(v1, va.clone());
                    }
                }
            },
            None => {
                match m1 {
                    Some(vb) => {
                        self.vertices.insert(v0, vb.clone());
                    },
                    None => {
                        let v: Rc<RefCell<Vertex>> = RefCell::new(
                            Vertex::new(v0.clone())).into();
                        self.vertices.insert(v0, v.clone());
                        self.vertices.insert(v1, v);
                    }
                }
            }
        }
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
            graph.insert_equality(v0, v1);
        }

        /* For each book for each shot, create an edge */
        for book in cave.books.iter() {
            for s in book.shots.iter() {
                let ((s0, s1), shot) = s;
                let stat0 = format!("{}@{}", s0.name, book.prefix);
                let v0 = graph.insert_vertex(&stat0);
                let stat1 = format!("{}@{}", s1.name, book.prefix);
                let v1 = graph.insert_vertex(&stat1);

                let e1 = Edge {
                    other: v1.clone(),
                    distance: f64::from(shot.length)
                };
                v0.borrow_mut().edges.push(e1);

                let e0 = Edge {
                    other: v0.clone(),
                    distance: f64::from(shot.length)
                };
                v1.borrow_mut().edges.push(e0);
            }
        }

        graph
    }

    /*
     * Return the shortest distance between the two vertices
     */
    fn shortest_path_between_vertices(
        &self,
        start_v: &Rc<RefCell<Vertex>>,
        end_v: &Rc<RefCell<Vertex>>,
    ) -> f64 {
        let start_name = start_v.borrow().name.clone();
        let end_name = end_v.borrow().name.clone();

        let mut distances: HashMap<String, f64> = HashMap::with_capacity(self.vertices.len());
        for name in self.vertices.keys() {
            distances.insert(name.clone(), Self::CANARY);
        }
        distances.insert(start_name.clone(), 0.0);

        let mut frontier: BinaryHeap<VertexTracker> = BinaryHeap::new();
        frontier.push(VertexTracker {
            name: start_name,
            distance: 0.0,
        });

        while let Some(current) = frontier.pop() {
            let known = *distances
                .get(&current.name)
                .expect("Missing distance for queued vertex");

            // Skip stale entries that were superseded by a shorter path.
            if current.distance > known {
                continue;
            }

            if current.name == end_name {
                return current.distance;
            }

            let vertex = self
                .vertices
                .get(&current.name)
                .expect("Couldn't find vertex by queued name");

            for edge in vertex.borrow().edges.iter() {
                let other_name = edge.other.borrow().name.clone();
                let new_distance = current.distance + edge.distance;
                let other_known = *distances
                    .get(&other_name)
                    .expect("Missing distance for adjacent vertex");

                if new_distance < other_known {
                    distances.insert(other_name.clone(), new_distance);
                    frontier.push(VertexTracker {
                        name: other_name,
                        distance: new_distance,
                    });
                }
            }
        }

        Self::CANARY
    }

    /*
     * Return the shortest distance between the two named vertices
     */
    pub fn shortest_path(&self, start: &String, finish: &String) -> Result<f64, String> {
        let start_v = match self.vertices.get(start) {
            Some(vertex) => vertex,
            None => return Err(format!("Unknown starting station: {}", start)),
        };
        let end_v = match self.vertices.get(finish) {
            Some(vertex) => vertex,
            None => return Err(format!("Unknown ending station: {}", finish)),
        };

        Ok(self.shortest_path_between_vertices(start_v, end_v))
    }

    /*
     * Return the two vertices that are the furthest apart from each other,
     * and the distance between them
     */
    pub fn diameter(&self, no_midpoints: bool) -> (String, String, f64) {
        let mut longest_distance: f64 = 0.0;
        let mut longest_start = String::new();
        let mut longest_end = String::new();
        //let mut counter0 = 0;

        for (start_name, v0) in self.vertices.iter() {
            //let mut counter1 = 0;

            /* Optionally skip non-endpoint vertices in diameter search */
            if no_midpoints && v0.borrow().edges.len() != 1 {continue;}

            for (end_name, v1) in self.vertices.iter() {
                if no_midpoints && v1.borrow().edges.len() != 1 {continue;}
                //let begin = SystemTime::now();
                if start_name == end_name {continue;}
                let distance = self.shortest_path_between_vertices(v0, v1);
                if distance > longest_distance {
                    longest_distance = distance;
                    longest_start = start_name.clone();
                    longest_end = end_name.clone();
                } else if distance == Self::CANARY {
                    println!("{} and {} are disconnected",
                             start_name, end_name);
                }
                //println!("Done with {}/{}", counter0, counter1);
                //counter1 += 1;
                //let end = SystemTime::now();
                //let diff0 = end.duration_since(begin).unwrap();
                //println!("Loop took {:?}", diff0);
            }

            //counter0 += 1;
        }

        //let end = SystemTime::now();
        //let time_diff = end.duration_since(begin).unwrap();
        //println!("Diameter took {:?}", time_diff);

        (longest_start, longest_end, longest_distance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::cave::Cave;

    #[test]
    fn test_shortest_path_unknown_start() {
        let mut cave = Cave::new();
        let dir = "".to_string();
        let file = "data/HMaze.th".to_string();
        super::super::cave::therion_reader::read_therion(&mut cave, &dir, &file);

        let graph = MapGraph::cave_graph(&cave);
        let result = graph.shortest_path(&"UNKNOWN@Test".to_string(), &"M1@HMaze".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown starting station"));
    }

    #[test]
    fn test_shortest_path_unknown_end() {
        let mut cave = Cave::new();
        let dir = "".to_string();
        let file = "data/HMaze.th".to_string();
        super::super::cave::therion_reader::read_therion(&mut cave, &dir, &file);

        let graph = MapGraph::cave_graph(&cave);
        let result = graph.shortest_path(&"M1@HMaze".to_string(), &"UNKNOWN@Test".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown ending station"));
    }

    #[test]
    fn test_shortest_path_returns_ok_for_valid_stations() {
        let mut cave = Cave::new();
        let dir = "".to_string();
        let file = "data/HMaze.th".to_string();
        super::super::cave::therion_reader::read_therion(&mut cave, &dir, &file);

        let graph = MapGraph::cave_graph(&cave);
        let result = graph.shortest_path(&"M1@HMaze".to_string(), &"M2@HMaze".to_string());
        assert!(result.is_ok(), "Valid stations should return Ok");
    }

    #[test]
    fn test_map_graph_creation_succeeds() {
        let mut cave = Cave::new();
        let dir = "".to_string();
        let file = "data/HMaze.th".to_string();
        super::super::cave::therion_reader::read_therion(&mut cave, &dir, &file);

        let graph = MapGraph::cave_graph(&cave);
        assert!(!graph.vertices.is_empty(), "Graph should have vertices");
    }
}
