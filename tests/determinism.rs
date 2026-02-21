use rulebound::*;
use rand::SeedableRng;

#[test]
fn same_seed_same_solution_100_times() {
    let reference = solve_with_seed(42);
    for _ in 0..100 {
        let result = solve_with_seed(42);
        assert_eq!(result.solution.values, reference.solution.values);
    }
}

fn solve_with_seed(seed: u64) -> SolveResult {
    let mut solver = Solver::<1>::new(7, 3, SolverConfig::default());
    let edges = [(0,1),(0,2),(1,2),(1,3),(2,3),(2,4),(2,5),(3,4),(4,5)];
    for (a, b) in edges { solver.add_constraint(NotEqual::new(a, b)); }
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    solver.solve(&mut rng).unwrap()
}
