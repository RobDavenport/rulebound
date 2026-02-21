# rulebound

A lightweight, `no_std` constraint propagation solver for real-time game logic and procedural verification.

**[Live Demo](https://robdavenport.github.io/rulebound/)**

## Features

- **Game-time constraint solving**: Incremental AC-3 propagation for per-frame use
- **Built-in constraints**: AllDifferent, NotEqual, LessThan, ExactlyN, Table, Implication
- **Configurable search**: Propagation + optional backtracking with multiple strategies
- **Observable**: Monitor solving progress via the `Observer` trait
- **`no_std` compatible**: Works in embedded and WASM environments
- **Deterministic**: Same seed produces same results

## Quick Start

```rust
use rulebound::*;

let config = SolverConfig::new();
let mut solver = Solver::<2>::new(9, 9, config);

// Add constraints
solver.add_constraint(AllDifferent::new(&[0, 1, 2]));

// Fix known values
solver.assign(0, 5).unwrap();

// Solve
let result = solver.solve(&mut rng).unwrap();
```

## Use Cases

- Verify WFC/procedural generation output is solvable
- Place items/enemies with spatial rules
- NPC scheduling (Stardew Valley-style daily routines)
- Puzzle generation + verification (Sudoku, lock-and-key)
- Crafting/economy balance validation

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
