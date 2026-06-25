use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
//use std::time::SystemTime;
use super::cave::Cave;

struct Edge {
    other : usize,
    distance: f64
}

struct Vertex {
    edges: Vec<Edge>
}

impl Vertex {
    pub fn new() -> Vertex {
        Vertex {
            edges: Vec::new()
        }
    }
}

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
    // ID-indexed vertex storage for hot-path traversal.
    vertices: Vec<Vertex>,
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

                self.vertices.push(Vertex::new());
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
     * Return mutable references to the two Vertexes with the passed IDs
     */
    fn vertex_pair_mut(vertices: &mut [Vertex], id0: usize, id1: usize) ->
        (&mut Vertex, &mut Vertex) {
        if id0 < id1 {
            let (left, right) = vertices.split_at_mut(id1);
            (&mut left[id0], &mut right[0])
        } else {
            let (left, right) = vertices.split_at_mut(id0);
            (&mut right[0], &mut left[id1])
        }
    }

    /*
     * Take an array of edge descriptions, turn them into edge objects that
     * reference the named vertex objects, and add them to the graph
     */
    pub fn insert_edges(&mut self, edges: &[(&str, &str, f32)]) {
        for i in 0..edges.len() {
            let (p0, p1, d) = edges[i];

            let id0 = *self.name_to_id.get(p0)
                .expect("Couldn't find the first vertex ID");
            let id1 = *self.name_to_id.get(p1)
                .expect("Couldn't find the second vertex ID");

            let e1 = Edge {
                other: id1,
                distance: f64::from(d)
            };

            let e0 = Edge {
                other: id0,
                distance: f64::from(d)
            };

            assert_ne!(id0, id1, "Self-loop edge is not allowed: {}", p0);

            let (v0, v1) = Self::vertex_pair_mut(&mut self.vertices, id0, id1);
            v0.edges.push(e1);
            v1.edges.push(e0);
        }
    }

    /*
     * In self.vertices we want to change all references to v1_obj to
     * reference v0_obj instead
     */
    fn move_equalities(&mut self, id0: usize, id1: usize) {
        if id0 == id1 {
            return;
        }

        for id in self.name_to_id.values_mut() {
            if *id == id1 {
                *id = id0;
            }
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
                        self.name_to_id.insert(v1, id0);
                    }
                }
            },
            None => {
                match m1 {
                    Some(id1) => {
                        self.name_to_id.insert(v0, id1);
                    },
                    None => {
                        let id0 = self.id_to_name.len();
                        self.name_to_id.insert(v0.clone(), id0);
                        self.name_to_id.insert(v1, id0);
                        self.id_to_name.push(v0.clone());
                        self.vertices.push(Vertex::new());
                    }
                }
            }
        }
    }

    /*
     * This frees the memory of vertex objects and names that have been
     * orphaned by the process of listing vertexes as equal
     */
    fn compact_vertices(&mut self) {
        if self.vertices.is_empty() {
            return;
        }

        let mut used: Vec<bool> = vec![false; self.vertices.len()];
        for &id in self.name_to_id.values() {
            used[id] = true;
        }

        let mut remap: Vec<usize> = vec![usize::MAX; self.vertices.len()];
        let mut compacted_vertices: Vec<Vertex> = Vec::new();
        let mut compacted_names: Vec<String> = Vec::new();

        for old_id in 0..self.vertices.len() {
            if used[old_id] {
                remap[old_id] = compacted_vertices.len();
                compacted_vertices.push(Vertex::new());
                compacted_names.push(self.id_to_name[old_id].clone());
            }
        }

        for id in self.name_to_id.values_mut() {
            *id = remap[*id];
        }

        self.vertices = compacted_vertices;
        self.id_to_name = compacted_names;
    }

    /*
     * Take a cave object and turn it into a graph we can analyze
     */
    pub fn cave_graph(cave: &Cave) -> Result<MapGraph, String> {
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
        graph.compact_vertices();

        /* For each book for each shot, create an edge */
        for book in cave.books.iter() {
            for s in book.shots.iter() {
                let ((s0, s1), shot) = s;
                let stat0 = format!("{}@{}", s0.name, book.prefix);
                let id0 = graph.insert_vertex(&stat0);
                let stat1 = format!("{}@{}", s1.name, book.prefix);
                let id1 = graph.insert_vertex(&stat1);

                if id0 == id1 {
                    return Err(format!(
                        "Self-loop edge detected while building graph: {}",
                        stat0
                    ));
                }

                let e1 = Edge {
                    other: id1,
                    distance: f64::from(shot.length)
                };

                let e0 = Edge {
                    other: id0,
                    distance: f64::from(shot.length)
                };

                let (v0, v1) = Self::vertex_pair_mut(&mut graph.vertices, id0, id1);
                v0.edges.push(e1);
                v1.edges.push(e0);
            }
        }

        Ok(graph)
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

            for edge in self.vertices[current.id].edges.iter() {
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

        let mut distances: Vec<f64> = vec![Self::CANARY; self.vertices.len()];
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
        let mut distances: Vec<f64> = vec![Self::CANARY; self.vertices.len()];
        let mut frontier: BinaryHeap<VertexTracker> = BinaryHeap::new();

        for start_id in 0..self.vertices.len() {
            if no_midpoints && self.vertices[start_id].edges.len() != 1 {
                continue;
            }

            for end_id in (start_id + 1)..self.vertices.len() {
                if no_midpoints && self.vertices[end_id].edges.len() != 1 {
                    continue;
                }

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

        let graph = MapGraph::cave_graph(&cave).expect("Failed to build cave graph");
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

        let graph = MapGraph::cave_graph(&cave).expect("Failed to build cave graph");
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

        let graph = MapGraph::cave_graph(&cave).expect("Failed to build cave graph");
        let result = graph.shortest_path(&"M1@HMaze".to_string(), &"M2@HMaze".to_string());
        assert!(result.is_ok(), "Valid stations should return Ok");
    }

    #[test]
    fn test_map_graph_creation_succeeds() {
        let mut cave = Cave::new();
        let dir = "".to_string();
        let file = "data/HMaze.th".to_string();
        super::super::cave::therion_reader::read_therion(&mut cave, &dir, &file);

        let graph = MapGraph::cave_graph(&cave).expect("Failed to build cave graph");
        assert!(!graph.vertices.is_empty(), "Graph should have vertices");
    }
}
