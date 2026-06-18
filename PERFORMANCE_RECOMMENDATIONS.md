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

- [ ] 1) Replace repeated `Vec::sort()` in shortest-path traversal with a min-priority queue (`BinaryHeap` pattern).
  - Why: Sorting the candidate list repeatedly is expensive.
  - Expected gain: Large reduction in shortest-path inner-loop cost.

- [ ] 2) Avoid duplicate pair work in `diameter()`.
  - Why: Current pair iteration effectively computes both (A, B) and (B, A).
  - Expected gain: Up to ~2x fewer shortest-path calls in diameter search.

- [ ] 3) Reuse shortest-path buffers across runs.
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
