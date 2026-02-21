use rulebound::*;
use rand::SeedableRng;

#[test]
fn australia_map_3_colors() {
    // WA, NT, SA, Q, NSW, V, T — edges represent shared borders
    let mut solver = Solver::<1>::new(7, 3, SolverConfig::default());
    let edges = [(0,1),(0,2),(1,2),(1,3),(2,3),(2,4),(2,5),(3,4),(4,5)];
    for (a, b) in edges {
        solver.add_constraint(NotEqual::new(a, b));
    }
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    let result = solver.solve(&mut rng).unwrap();
    for (a, b) in edges {
        assert_ne!(result.solution.values[a], result.solution.values[b],
            "adjacent nodes {a} and {b} share color");
    }
}

#[test]
fn petersen_graph_3_colors() {
    // Petersen graph: 10 nodes, 15 edges, chromatic number = 3
    // Outer cycle: 0-1-2-3-4-0
    // Inner pentagram: 5-7-9-6-8-5
    // Spokes: 0-5, 1-6, 2-7, 3-8, 4-9
    let mut solver = Solver::<1>::new(10, 3, SolverConfig::default());
    let edges = [
        // Outer cycle
        (0,1),(1,2),(2,3),(3,4),(4,0),
        // Inner pentagram
        (5,7),(7,9),(9,6),(6,8),(8,5),
        // Spokes
        (0,5),(1,6),(2,7),(3,8),(4,9),
    ];
    for (a, b) in edges {
        solver.add_constraint(NotEqual::new(a, b));
    }
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    let result = solver.solve(&mut rng).unwrap();
    for (a, b) in edges {
        assert_ne!(result.solution.values[a], result.solution.values[b],
            "adjacent nodes {a} and {b} share color");
    }

    // Verify it's NOT 2-colorable (chromatic number is 3, not 2)
    let mut solver2 = Solver::<1>::new(10, 2, SolverConfig::default());
    for (a, b) in edges {
        solver2.add_constraint(NotEqual::new(a, b));
    }
    let mut rng2 = rand::rngs::SmallRng::seed_from_u64(42);
    assert!(solver2.solve(&mut rng2).is_err(), "Petersen graph with 2 colors should be unsolvable");
}

#[test]
fn complete_graph_k4_needs_4_colors() {
    // K4 with 3 colors should be unsolvable
    let mut solver = Solver::<1>::new(4, 3, SolverConfig::default());
    for i in 0..4 { for j in (i+1)..4 { solver.add_constraint(NotEqual::new(i, j)); } }
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    assert!(solver.solve(&mut rng).is_err(), "K4 with 3 colors is unsolvable");

    // K4 with 4 colors should be solvable
    let mut solver = Solver::<1>::new(4, 4, SolverConfig::default());
    for i in 0..4 { for j in (i+1)..4 { solver.add_constraint(NotEqual::new(i, j)); } }
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    let result = solver.solve(&mut rng).unwrap();
    for i in 0..4 { for j in (i+1)..4 {
        assert_ne!(result.solution.values[i], result.solution.values[j]);
    }}
}
