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
