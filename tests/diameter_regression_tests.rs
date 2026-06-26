use cavegraph::cave_graph::cave::{Cave, therion_reader, walls_reader};
use cavegraph::cave_graph::graph::MapGraph;

const DIAMETER_TOLERANCE: f64 = 0.2;

fn assert_diameter_within_tolerance(path: &str, observed: f64, expected: f64) {
    let delta = (observed - expected).abs();
    assert!(
        delta <= DIAMETER_TOLERANCE,
        "Diameter regression for {}: observed={} expected={} delta={} tolerance={}",
        path,
        observed,
        expected,
        delta,
        DIAMETER_TOLERANCE
    );
}

#[test]
fn test_diameter_regression_hmaze() {
    let mut cave = Cave::new();
    therion_reader::read_therion(&mut cave, &"".to_string(), &"data/HMaze.th".to_string());

    let graph = MapGraph::cave_graph(&cave).expect("Failed to build cave graph");
    let distance_with_midpoints = graph.diameter(false).distance;
    let distance_no_midpoints = graph.diameter(true).distance;

    assert_diameter_within_tolerance(
        "data/HMaze.th (with midpoints)",
        distance_with_midpoints,
        134.19999980926514,
    );
    assert_diameter_within_tolerance(
        "data/HMaze.th (no midpoints)",
        distance_no_midpoints,
        134.19999980926514,
    );
}

#[test]
fn test_diameter_regression_deep_lake() {
    let mut cave = Cave::new();
    therion_reader::read_therion(
        &mut cave,
        &"".to_string(),
        &"data/Deep_Lake/deep_lake.th".to_string(),
    );

    let graph = MapGraph::cave_graph(&cave).expect("Failed to build cave graph");
    let distance_with_midpoints = graph.diameter(false).distance;
    let distance_no_midpoints = graph.diameter(true).distance;

    assert_diameter_within_tolerance(
        "data/Deep_Lake/deep_lake.th (with midpoints)",
        distance_with_midpoints,
        1075.8000001907349,
    );
    assert_diameter_within_tolerance(
        "data/Deep_Lake/deep_lake.th (no midpoints)",
        distance_no_midpoints,
        1075.8000001907349,
    );
}

#[test]
fn test_diameter_regression_walls_mcsvy() {
    let mut cave = Cave::new();
    walls_reader::read_walls(
        &mut cave,
        &"data/Walls/".to_string(),
        &"MCSVY.wpj".to_string(),
    );

    let graph = MapGraph::cave_graph(&cave).expect("Failed to build cave graph");
    let distance_with_midpoints = graph.diameter(false).distance;
    let distance_no_midpoints = graph.diameter(true).distance;

    assert_diameter_within_tolerance(
        "data/Walls/MCSVY.wpj (with midpoints)",
        distance_with_midpoints,
        205.9000015258789,
    );
    assert_diameter_within_tolerance(
        "data/Walls/MCSVY.wpj (no midpoints)",
        distance_no_midpoints,
        205.9000015258789,
    );
}
