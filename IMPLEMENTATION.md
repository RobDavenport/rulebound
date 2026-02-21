# rulebound Implementation Plan

## What This Is

A lightweight, `no_std` constraint propagation solver (AC-3) for real-time game logic. All module files are scaffolded with trait signatures and `todo!()` stubs. Your job: fill in the implementations, write tests, build the WASM demo.

## Hard Rules

- `#![no_std]` with `extern crate alloc` — no std dependency in core library
- `rand_core` is the ONLY dependency
- All tests: `cargo test --target x86_64-pc-windows-msvc`
- WASM check: `cargo build --target wasm32-unknown-unknown --release`
- Deterministic: same seed must produce same results

## Reference Implementation

The sibling library `wavfc` (at `../wavfc/`) uses the same patterns. Key files to study:
- `../wavfc/src/propagator.rs` — AC-3/AC-4 propagation loop (worklist-based)
- `../wavfc/src/solver.rs` — observe-collapse-propagate loop with backtracking
- `../wavfc/src/bitset.rs` — BitSet implementation (rulebound's is already adapted from this)
- `../wavfc/src/constraint.rs` — GlobalConstraint pattern
- `../wavfc/demo-wasm/src/lib.rs` — WASM FFI bindings pattern
- `../wavfc/demo-wasm/www/main.js` — Demo UI pattern (898 lines, ES6 module)

## Implementation Steps

### Phase 1: Core Constraint Engine

**Step 1: Implement NotEqual constraint** (`src/constraint.rs`)
- If variable A is a singleton {v}, remove v from B's domain (and vice versa)
- Return `Err(Contradiction)` if either domain becomes empty
- Write test: two variables, assign one, check other's domain shrinks

**Step 2: Implement AllDifferent constraint** (`src/constraint.rs`)
- For each singleton variable in scope, remove its value from all other variables' domains
- This is the "naive" AllDifferent (not full arc consistency — that's fine for games)
- Write test: 3 variables with domain {1,2,3}, assign first to 1, check others lose 1

**Step 3: Implement LessThan constraint** (`src/constraint.rs`)
- Remove values from A's domain that are >= max(B's domain)
- Remove values from B's domain that are <= min(A's domain)
- Write test: A in {1..5}, B in {1..5}, after propagation A loses 5 and B loses 1

**Step 4: Implement AC-3 propagation engine** (`src/propagator.rs`)
- Worklist algorithm: queue all constraints initially
- Pop constraint, call propagate(), if any domain changed, re-queue all constraints involving those variables
- Loop until worklist empty (fixpoint) or contradiction
- Key: track which variables changed to avoid redundant work
- Write test: chain of NotEqual constraints, assign first variable, check propagation cascades

**Step 5: Implement Solver.assign()** (`src/solver.rs`)
- Set variable's domain to singleton {value}
- Run propagation
- Return error if contradiction

**Step 6: Implement Solver.propagate()** (`src/solver.rs`)
- Delegate to Propagator
- Return whether all variables are solved

**Step 7: Implement backtracking search** (`src/solver.rs`)
- Solver.solve(): propagate first, then if unsolved:
  - Select unresolved variable (using Heuristic — MinDomain picks smallest domain)
  - Try each value in its domain:
    - Snapshot domains
    - Assign value, propagate
    - If contradiction, restore snapshot, try next value
    - If solved, return Solution
  - If all values fail, backtrack (or error based on BacktrackStrategy)
- Write test: Sudoku solver (see Phase 2)

**Step 8: Implement remaining constraints**
- `ExactlyN`: count variables that must/can/cannot take the value, prune accordingly
- `Table`: for each variable in scope, remove values that don't appear in any allowed tuple consistent with current domains
- `Implication`: if A is singleton {val_a}, intersect B's domain with allowed_b

**Step 9: Implement incremental propagation** (`src/solver.rs`, `src/propagator.rs`)
- `propagate_incremental()`: only queue constraints involving recently changed variables
- Useful for game-time use where constraints are added one at a time

### Phase 2: Tests

Write these test files in `tests/` (integration tests):

**tests/sudoku.rs** — The killer demo test
```rust
// Create 81 variables (9x9), each with domain {1..9}
// Add AllDifferent for each row (9 constraints)
// Add AllDifferent for each column (9 constraints)
// Add AllDifferent for each 3x3 box (9 constraints)
// Pre-assign known values from a real puzzle
// Call solver.solve() and verify the solution is valid
```

**tests/map_coloring.rs**
```rust
// Create N variables (graph nodes), each with domain {0..num_colors}
// Add NotEqual for each edge
// Solve and verify no adjacent nodes share a color
// Test with: Australia map (7 nodes), Petersen graph, complete graph K4
```

**tests/n_queens.rs**
```rust
// N variables (one per row), domain {0..N-1} (column positions)
// AllDifferent on all variables (no two queens in same column)
// Custom diagonal constraints (|col_i - col_j| != |i - j|)
// Solve for N=4, N=8
```

**tests/determinism.rs**
```rust
// Same problem + same seed → same solution, 100 times
```

**tests/incremental.rs**
```rust
// Start with partial constraints, propagate
// Add more constraints, propagate_incremental
// Verify correctness matches solving from scratch
```

### Phase 3: Benchmarks

**benches/propagation.rs** — Replace placeholder with:
- Sudoku solve (easy, medium, hard puzzles)
- Map coloring (20-node graph, 4 colors)
- Large AllDifferent (50 variables, domain 50)
- Incremental: add 10 constraints one at a time

### Phase 4: WASM Demo

**demo-wasm/src/lib.rs** — Replace stubs with:
- `solve_sudoku(puzzle_json: &str) -> String` — Takes 81 values (0=unknown), returns solution JSON
- `StepSolver::new(puzzle_json: &str)` — Initialize with puzzle
- `StepSolver::step() -> String` — Returns JSON event:
  - `{"type":"propagated","variable":42,"domain":[1,3,7]}` — domain shrunk
  - `{"type":"assigned","variable":42,"value":3}` — variable solved
  - `{"type":"backtrack","depth":2}` — backtracking occurred
  - `{"type":"solved","values":[...]}` — done
  - `{"type":"contradiction"}` — unsolvable

**demo-wasm/www/main.js** — Replace placeholder with:
- Full Sudoku mode: 9x9 canvas grid
  - Click cell to cycle through values (0=empty, 1-9)
  - "Solve" button: instant solve, show result
  - "Step" button: one propagation step, animate domain changes
  - "Auto Play" button: animate solving with speed slider
  - Color coding: given values (white), solved values (cyan), domain hints (small gray numbers)
- Map Coloring mode:
  - Pre-built graph (Australia map) drawn on canvas
  - Nodes as circles, edges as lines
  - "Solve" colors the nodes
  - Step-through shows constraint propagation
- Stats panel: variables solved, domains pruned, backtracks, time

**demo-wasm/www/index.html** — Already has the layout, may need minor tweaks for new controls

### Phase 5: Verify Everything

```bash
# Core library
cargo test --target x86_64-pc-windows-msvc
cargo build --target wasm32-unknown-unknown --release
cargo bench

# WASM demo
wasm-pack build demo-wasm --target web --release
# Manually test in browser: open demo-wasm/www/index.html via local server
```

Commit and push. GitHub Actions will deploy to Pages automatically.

## AC-3 Algorithm Reference

```
function AC-3(constraints, domains):
    worklist = all constraints
    while worklist is not empty:
        constraint = worklist.pop()
        for each variable V in constraint.scope():
            old_size = domains[V].count()
            constraint.propagate(domains)  // may prune domains
            if domains[V].is_empty():
                return CONTRADICTION
            if domains[V].count() < old_size:
                // V's domain changed — re-queue all constraints involving V
                for each constraint C involving V (except current):
                    worklist.push(C)
    return OK
```

## Backtracking Search Reference

```
function solve(domains, constraints, rng):
    propagate(domains, constraints)
    if all domains are singletons: return SOLVED
    if any domain is empty: return CONTRADICTION

    V = select_variable(domains, heuristic)  // e.g., smallest domain
    for value in domains[V] (shuffled by rng):
        snapshot = clone(domains)
        domains[V] = {value}
        result = solve(domains, constraints, rng)  // recurse
        if result == SOLVED: return SOLVED
        domains = snapshot  // restore
    return CONTRADICTION
```
