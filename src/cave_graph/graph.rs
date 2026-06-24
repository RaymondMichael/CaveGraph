use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::rc::Rc;
//use std::time::SystemTime;
use super::cave::Cave;

struct Edge {
    other : usize,
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

// Integer-ID based vertex tracker for hot-path Dijkstra
struct VertexTracker {
    id: usize,
    distance: f64
}

impl Ord for VertexTracker {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering so BinaryHeap behaves as a min-priority queue.
        other.distance.total_cmp(&self.distance).
            then_with(|| other.id.cmp(&self.id))
    }
}

impl PartialOrd for VertexTracker {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for VertexTracker {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.distance == other.distance
    }
}

impl Eq for VertexTracker {}

pub struct MapGraph {
    //XXX duplicates the name
    vertices: Vec<Rc<RefCell<Vertex>>>,
    name_to_id: HashMap<String, usize>,
    id_to_name: Vec<String>,
}

impl MapGraph {
    const CANARY: f64 = 999999999999.9;

    pub fn new() -> MapGraph {
        MapGraph {
            vertices: Vec::new(),
            name_to_id: HashMap::new(),
            id_to_name: Vec::new(),
        }
    }

    /*
     * Add the vertex if the name is not already in the set of vertices.
     * Return the vertex.
     */
    fn insert_vertex(&mut self, name: &String) -> usize {
        let id = match self.name_to_id.get(name) {
            Some(id) => {
                *id
            },
            None => {
                // Assign a new ID for this vertex
                let id = self.id_to_name.len();
                self.name_to_id.insert(name.clone(), id);
                self.id_to_name.push(name.clone());

                let v: Rc<RefCell<Vertex>> = RefCell::new(Vertex::new(name.clone())).into();
                self.vertices.push(v);
                id
            }
        };

        id
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

            let id0 = self.name_to_id.get(p0).
                expect("Couldn't find the first vertex ID");
            let id1 = self.name_to_id.get(p1).
                expect("Couldn't find the second vertex ID");

            let v0 = &self.vertices[*id0];
            let v1 = &self.vertices[*id1];

            let e1 = Edge {
                other: *id1,
                distance: f64::from(d)
            };
            v0.borrow_mut().edges.push(e1);

            let e0 = Edge {
                other: *id0,
                distance: f64::from(d)
            };
            v1.borrow_mut().edges.push(e0);
        }
    }

    /*
     * In self.vertices we want to change all references to v1_obj to
     * reference v0_obj instead
     */
    fn move_equalities(&mut self, id0: usize, id1: usize) {
        let mut v: Vec<usize> = Vec::new();
        let v0_obj = self.vertices[id0].clone();
        let v1_obj = self.vertices[id1].clone();

        /* Collect all the mappings we need to move */
        for i in 0..self.vertices.len() {
            if self.vertices[i] == v1_obj {
                v.push(i);
            }
        }

        /* Insert them now that self.vertices is mutable */
        for id in v.iter() {
            self.vertices[*id] = v0_obj.clone();
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
        let m0 = self.name_to_id.get(&v0).copied();
        let m1 = self.name_to_id.get(&v1).copied();

        match m0 {
            Some(id0) => {
                match m1 {
                    Some(id1) => {
                        self.move_equalities(id0, id1);
                    },
                    None => {
                        let id1 = self.id_to_name.len();
                        self.name_to_id.insert(v1.clone(), id1);
                        self.id_to_name.push(v1.clone());
                        let va = self.vertices[id0].clone();
                        self.vertices.push(va);
                    }
                }
            },
            None => {
                match m1 {
                    Some(id1) => {
                        let id0 = self.id_to_name.len();
                        self.name_to_id.insert(v0.clone(), id0);
                        self.id_to_name.push(v0.clone());
                        let vb = self.vertices[id1].clone();
                        self.vertices.push(vb);
                    },
                    None => {
                        let v: Rc<RefCell<Vertex>> = RefCell::new(
                            Vertex::new(v0.clone())).into();
                        let id0 = self.id_to_name.len();
                        self.name_to_id.insert(v0.clone(), id0);
                        self.id_to_name.push(v0.clone());
                        self.vertices.push(v.clone());
                        let id1 = self.id_to_name.len();
                        self.name_to_id.insert(v1.clone(), id1);
                        self.id_to_name.push(v1.clone());
                        self.vertices.push(v);
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
                let id0 = graph.insert_vertex(&stat0);
                let v0 = graph.vertices[id0].clone();
                let stat1 = format!("{}@{}", s1.name, book.prefix);
                let id1 = graph.insert_vertex(&stat1);
                let v1 = &graph.vertices[id1];

                let e1 = Edge {
                    other: id1,
                    distance: f64::from(shot.length)
                };
                v0.borrow_mut().edges.push(e1);

                let e0 = Edge {
                    other: id0,
                    distance: f64::from(shot.length)
                };
                v1.borrow_mut().edges.push(e0);
            }
        }

        graph
    }

    /*
     * Return the shortest distance between the two vertices using integer IDs
     * in the hot path. Uses pre-allocated Vec for O(1) distance lookups.
     */
    fn shortest_path_between_ids(
        &self,
        start_id: usize,
        end_id: usize,
        distances: &mut Vec<f64>,
        frontier: &mut BinaryHeap<VertexTracker>,
    ) -> f64 {
        // Reset distances
        for distance in distances.iter_mut() {
            *distance = Self::CANARY;
        }
        distances[start_id] = 0.0;

        frontier.clear();
        frontier.push(VertexTracker {
            id: start_id,
            distance: 0.0,
        });

        while let Some(current) = frontier.pop() {
            let known = distances[current.id];

            // Skip stale entries that were superseded by a shorter path.
            if current.distance > known {
                continue;
            }

            if current.id == end_id {
                return current.distance;
            }

            let vertex = &self.vertices[current.id];

            for edge in vertex.borrow().edges.iter() {
                let other_id = edge.other;
                let new_distance = current.distance + edge.distance;
                let other_known = distances[other_id];

                if new_distance < other_known {
                    distances[other_id] = new_distance;
                    frontier.push(VertexTracker {
                        id: other_id,
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
        let start_id = match self.name_to_id.get(start) {
            Some(start_id) => start_id,
            None => return Err(format!("Unknown starting station: {}", start)),
        };
        let end_id = match self.name_to_id.get(finish) {
            Some(end_id) => end_id,
            None => return Err(format!("Unknown ending station: {}", finish)),
        };

        let mut distances: Vec<f64> = Vec::with_capacity(self.vertices.len());
        for _ in 0..self.vertices.len() {
            distances.push(Self::CANARY);
        }
        let mut frontier: BinaryHeap<VertexTracker> = BinaryHeap::new();

        Ok(self.shortest_path_between_ids(
            *start_id, *end_id, &mut distances, &mut frontier))
    }

    /*
     * Return the two vertices that are the furthest apart from each other,
     * and the distance between them
     */
    pub fn diameter(&self, no_midpoints: bool) -> (String, String, f64) {
        let mut longest_distance: f64 = 0.0;
        let mut longest_start = String::new();
        let mut longest_end = String::new();

        // Pre-allocate reusable buffers for ID-based Dijkstra
        let mut distances: Vec<f64> = vec![Self::CANARY; self.id_to_name.len()];
        let mut frontier: BinaryHeap<VertexTracker> = BinaryHeap::new();

        for start_id in 0..self.vertices.len() {
            let v0 = &self.vertices[start_id];
            if v0.borrow().edges.len() != 1 && no_midpoints {continue;}

            for end_id in (start_id + 1)..self.vertices.len() {
                let v1 = &self.vertices[end_id];
                if v1.borrow().edges.len() != 1 && no_midpoints {continue;}

                let distance = self.shortest_path_between_ids(
                    start_id,
                    end_id,
                    &mut distances,
                    &mut frontier,
                );
                if distance > longest_distance {
                    longest_distance = distance;
                    longest_start = self.id_to_name[start_id].clone();
                    longest_end = self.id_to_name[end_id].clone();
                } else if distance == Self::CANARY {
                    println!("{} and {} are disconnected", self.id_to_name[start_id], self.id_to_name[end_id]);
                }
            }
        }

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
