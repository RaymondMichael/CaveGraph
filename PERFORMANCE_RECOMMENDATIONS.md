# CaveGraph Performance Recommendations

This file tracks the optimization backlog for `diameter()` and related shortest-path work.

## Baseline observations (small-graph benchmark)

- Runtime grows sharply with vertex count even at modest edge counts.
- With midpoints enabled, representative medians were approximately:
  - Chain: V=100 -> 0.8s, V=200 -> 6.5s, V=300 -> 24s
  - Dense: V=100 -> 1.3s, V=200 -> 13.5s, V=300 -> 56s
- Control sweep with fixed edges (E=150) still showed strong growth:
  - V=100 -> 1.2s, V=200 -> 6.7s, V=300 -> 24.3s
- Current bottlenecks are algorithmic and inner-loop data-structure costs, not only density.

## Prioritized optimization list

- [x] 1) Replace repeated `Vec::sort()` in shortest-path traversal with a min-priority queue (`BinaryHeap` pattern).
  - Why: Sorting the candidate list repeatedly is expensive.
  - Expected gain: Large reduction in shortest-path inner-loop cost.

- [x] 2) Avoid duplicate pair work in `diameter()`.
  - Why: Current pair iteration effectively computes both (A, B) and (B, A).
  - Expected gain: Up to ~2x fewer shortest-path calls in diameter search.

- [x] 3) Reuse shortest-path buffers across runs.
  - Why: Rebuilding distance/visited structures each call adds allocation and hashmap overhead.
  - Expected gain: Lower per-call overhead during repeated diameter computations.

- [ ] 4) Move hot-path graph operations to integer vertex IDs.
  - Why: String/hash lookups and `Rc<RefCell<...>>` indirection are costly in tight loops.
  - Expected gain: Better cache locality and lower lookup/borrow overhead.

- [ ] 5) Cache endpoint list for `no_midpoints` mode.
  - Why: Endpoint filtering should be computed once and reused.
  - Expected gain: Faster diameter scans in midpoint-restricted mode.

## Secondary optimization candidates

- [ ] Add pruning bounds in diameter search to skip hopeless pairs.
- [ ] Parallelize independent start-node computations (e.g., Rayon) once core data structures are optimized.
- [ ] Add an optional approximate-diameter mode for fast exploratory runs.

## Validation workflow for each change

1. Implement exactly one optimization step.
2. Re-run the same `graph_bench` cases used for baseline comparisons.
3. Record before/after median and p95 timings.
4. Confirm exactness of diameter outputs (except in any explicit approximate mode).

## Working order

1. Priority queue shortest path.
2. Unique unordered pair iteration in diameter.
3. Buffer reuse.
4. Integer-ID graph representation.
5. Endpoint caching and optional parallelization.

## Results log template

Copy this block for each optimization pass.

### Optimization pass: <name>

- Date:
- Branch/commit:
- Recommendation item:
- Status: planned | in-progress | completed

#### Change summary

- Files changed:
- What changed:
- Any behavior changes:

#### Benchmark command set

```bash
# Baseline

# Post-change
```

#### Timing results

| Case | Baseline median (ms) | New median (ms) | Speedup (x) | Baseline p95 (ms) | New p95 (ms) |
|---|---:|---:|---:|---:|---:|
| chain V=100 |  |  |  |  |  |
| chain V=200 |  |  |  |  |  |
| chain V=300 |  |  |  |  |  |
| sparse V=100 E=150 |  |  |  |  |  |
| sparse V=200 E=150 |  |  |  |  |  |
| sparse V=300 E=150 |  |  |  |  |  |

#### Correctness checks

- Diameter value match vs baseline:
- Any regressions observed:

#### Decision

- Keep change? yes | no
- Follow-up actions:

## Results log

### Optimization pass: Priority-queue shortest path

- Date: 2026-06-18
- Branch/commit: working tree (pre-commit)
- Recommendation item: 1) Replace repeated `Vec::sort()` with a min-priority queue
- Status: completed

#### Change summary

- Files changed: `src/cave_graph/graph.rs`
- What changed: Replaced sort-based frontier management in shortest-path traversal with `BinaryHeap` min-priority queue behavior.
- Any behavior changes: Diameter/shortest-path outputs changed on some synthetic random topologies and now appear internally consistent with graph constraints.

#### Benchmark command set

```bash
# Baseline (pre-change)
cd /Users/mraymond/usr/dev/hpe/CaveGraph && {
  for mode in --with-midpoints --no-midpoints; do
    for v in 100 200 300; do
      for t in chain tree sparse medium dense; do
        cargo run --quiet --bin graph_bench -- --topology "$t" --vertices "$v" --repeats 3 "$mode";
      done;
    done;
  done;
}

cd /Users/mraymond/usr/dev/hpe/CaveGraph && {
  for v in 100 200 300; do
    cargo run --quiet --bin graph_bench -- --topology sparse --vertices "$v" --edges 150 --repeats 2 --with-midpoints;
  done;
}

# Post-change
cd /Users/mraymond/usr/dev/hpe/CaveGraph && {
  for mode in --with-midpoints --no-midpoints; do
    for v in 100 200 300; do
      for t in chain tree sparse medium dense; do
        cargo run --quiet --bin graph_bench -- --topology "$t" --vertices "$v" --repeats 3 "$mode";
      done;
    done;
  done;
}

cd /Users/mraymond/usr/dev/hpe/CaveGraph && {
  for v in 100 200 300; do
    cargo run --quiet --bin graph_bench -- --topology sparse --vertices "$v" --edges 150 --repeats 2 --with-midpoints;
  done;
}
```

#### Timing results

| Case | Baseline median (ms) | New median (ms) | Speedup (x) | Baseline p95 (ms) | New p95 (ms) |
|---|---:|---:|---:|---:|---:|
| chain V=100 | 794 | 759 | 1.05 | 825 | 781 |
| chain V=200 | 6453 | 6144 | 1.05 | 6576 | 6165 |
| chain V=300 | 23929 | 21150 | 1.13 | 24202 | 21153 |
| sparse V=100 E=150 | 1247 | 1146 | 1.09 | 1247 | 1146 |
| sparse V=200 E=150 | 6703 | 6437 | 1.04 | 6703 | 6437 |
| sparse V=300 E=150 | 24328 | 21423 | 1.14 | 24328 | 21423 |

Additional representative with-midpoints medians:

- Sparse defaults: V=100 987 -> 933 (1.06x), V=200 9208 -> 7868 (1.17x), V=300 37894 -> 27585 (1.37x)
- Medium defaults: V=100 1112 -> 1047 (1.06x), V=200 10638 -> 8806 (1.21x), V=300 43969 -> 30373 (1.45x)
- Dense defaults: V=100 1340 -> 1242 (1.08x), V=200 13550 -> 10604 (1.28x), V=300 56158 -> 36918 (1.52x)

#### Correctness checks

- Diameter value match vs baseline: no, values differ on random sparse/medium/dense synthetic graphs.
- Any regressions observed: tree topology became slower in some runs; `--no-midpoints` tree also regressed.
- Notes: baseline had impossible diameter outputs in some dense random cases (greater than obvious graph upper bounds), suggesting prior frontier logic could produce non-shortest paths.

#### Decision

- Keep change? yes
- Follow-up actions:
  - Investigate tree regressions with a focused micro-benchmark.
  - Proceed to recommendation item 2 (unique unordered pair iteration in `diameter()`).

### Optimization pass: Unique unordered pair iteration in diameter

- Date: 2026-06-18
- Branch/commit: working tree (pre-commit)
- Recommendation item: 2) Avoid duplicate pair work in `diameter()`
- Status: completed

#### Change summary

- Files changed: `src/cave_graph/graph.rs`
- What changed: Updated `diameter()` to iterate only unique unordered candidate pairs (`i < j`) rather than both `(A, B)` and `(B, A)`.
- Any behavior changes: No expected behavior change in diameter values; execution path count reduced.

#### Benchmark command set

```bash
# Baseline (pre-change)
cd /Users/mraymond/usr/dev/hpe/CaveGraph && {
  for mode in --with-midpoints --no-midpoints; do
    for v in 100 200 300; do
      for t in chain tree sparse medium dense; do
        cargo run --quiet --bin graph_bench -- --topology "$t" --vertices "$v" --repeats 3 "$mode";
      done;
    done;
  done;
}

cd /Users/mraymond/usr/dev/hpe/CaveGraph && {
  for v in 100 200 300; do
    cargo run --quiet --bin graph_bench -- --topology sparse --vertices "$v" --edges 150 --repeats 2 --with-midpoints;
  done;
}

# Post-change
cd /Users/mraymond/usr/dev/hpe/CaveGraph && {
  for mode in --with-midpoints --no-midpoints; do
    for v in 100 200 300; do
      for t in chain tree sparse medium dense; do
        cargo run --quiet --bin graph_bench -- --topology "$t" --vertices "$v" --repeats 3 "$mode";
      done;
    done;
  done;
}

cd /Users/mraymond/usr/dev/hpe/CaveGraph && {
  for v in 100 200 300; do
    cargo run --quiet --bin graph_bench -- --topology sparse --vertices "$v" --edges 150 --repeats 2 --with-midpoints;
  done;
}
```

#### Timing results

| Case | Baseline median (ms) | New median (ms) | Speedup (x) | Baseline p95 (ms) | New p95 (ms) |
|---|---:|---:|---:|---:|---:|
| chain V=100 | 758 | 408 | 1.86 | 783 | 418 |
| chain V=200 | 6114 | 3197 | 1.91 | 6203 | 3245 |
| chain V=300 | 21105 | 10741 | 1.96 | 21178 | 10793 |
| sparse V=100 E=150 | 1162 | 586 | 1.98 | 1162 | 586 |
| sparse V=200 E=150 | 6644 | 3201 | 2.08 | 6644 | 3201 |
| sparse V=300 E=150 | 21318 | 10750 | 1.98 | 21318 | 10750 |

Additional representative with-midpoints medians:

- Tree defaults: V=100 919 -> 495 (1.86x), V=200 7503 -> 3920 (1.91x), V=300 26395 -> 13192 (2.00x)
- Medium defaults: V=100 1038 -> 557 (1.86x), V=200 8843 -> 4503 (1.96x), V=300 30469 -> 15346 (1.99x)
- Dense defaults: V=100 1264 -> 653 (1.94x), V=200 10679 -> 5377 (1.99x), V=300 37032 -> 18584 (1.99x)

#### Correctness checks

- Diameter value match vs baseline: yes for deterministic cases; random topology endpoints may vary in name order while distance remains consistent.
- Any regressions observed: none in sampled benchmarks.

#### Decision

- Keep change? yes
- Follow-up actions:
  - Proceed to recommendation item 3 (reuse shortest-path buffers).

### Optimization pass: Reuse shortest-path buffers across diameter calls

- Date: 2026-06-18
- Branch/commit: bd350ae
- Recommendation item: 3) Reuse shortest-path buffers across runs
- Status: completed

#### Change summary

- Files changed: `src/cave_graph/graph.rs`
- What changed: Extracted core Dijkstra algorithm to `shortest_path_between_vertices_with_buffers()` accepting pre-allocated `HashMap<String, f64>` and `BinaryHeap<VertexTracker>`. Modified `diameter()` to create these buffers once and reuse them across all vertex-pair iterations with reset logic (values_mut for HashMap, clear for BinaryHeap).
- Any behavior changes: No behavior changes; internal allocation patterns differ.

#### Benchmark command set

```bash
# Baseline (post-opt#2)
cd /Users/mraymond/usr/dev/hpe/CaveGraph && {
  for mode in --with-midpoints --no-midpoints; do
    for v in 100 200 300; do
      for t in chain tree sparse medium dense; do
        cargo run --quiet --bin graph_bench -- --topology "$t" --vertices "$v" --repeats 3 "$mode";
      done;
    done;
  done;
}

cd /Users/mraymond/usr/dev/hpe/CaveGraph && {
  for v in 100 200 300; do
    cargo run --quiet --bin graph_bench -- --topology sparse --vertices "$v" --edges 150 --repeats 2 --with-midpoints;
  done;
}

# Post-change
cd /Users/mraymond/usr/dev/hpe/CaveGraph && {
  for mode in --with-midpoints --no-midpoints; do
    for v in 100 200 300; do
      for t in chain tree sparse medium dense; do
        cargo run --quiet --bin graph_bench -- --topology "$t" --vertices "$v" --repeats 3 "$mode";
      done;
    done;
  done;
}

cd /Users/mraymond/usr/dev/hpe/CaveGraph && {
  for v in 100 200 300; do
    cargo run --quiet --bin graph_bench -- --topology sparse --vertices "$v" --edges 150 --repeats 2 --with-midpoints;
  done;
}
```

#### Timing results

| Case | Baseline median (ms) | New median (ms) | Speedup (x) | Baseline p95 (ms) | New p95 (ms) |
|---|---:|---:|---:|---:|---:|
| chain V=100 | 408 | 283 | 1.44 | 418 | 308 |
| chain V=200 | 3197 | 2255 | 1.42 | 3245 | 2267 |
| chain V=300 | 10741 | 7705 | 1.39 | 10793 | 7766 |
| sparse V=100 E=150 | 586 | 462 | 1.27 | 586 | 462 |
| sparse V=200 E=150 | 3201 | 2363 | 1.35 | 3201 | 2363 |
| sparse V=300 E=150 | 10750 | 7912 | 1.36 | 10750 | 7912 |

Additional representative with-midpoints medians:

- Tree defaults: V=100 495 -> 361 (1.37x), V=200 3920 -> 2931 (1.34x), V=300 13192 -> 10185 (1.29x)
- Medium defaults: V=100 557 -> 427 (1.30x), V=200 4503 -> 3647 (1.24x), V=300 15346 -> 12422 (1.24x)
- Dense defaults: V=100 653 -> 542 (1.20x), V=200 5377 -> 4389 (1.23x), V=300 18584 -> 15496 (1.20x)

#### Correctness checks

- Diameter value match vs baseline: yes, all tested cases produce identical diameter values.
- Any regressions observed: none; consistent improvement across all topologies.

#### Decision

- Keep change? yes
- Follow-up actions:
  - Proceed to recommendation item 4 (integer-ID graph representation).
