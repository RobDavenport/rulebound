use rulebound::*;
use rand::SeedableRng;

#[test]
fn incremental_matches_full_solve() {
    // Build incrementally
    let mut solver = Solver::<1>::new(4, 4, SolverConfig::default());
    solver.add_constraint(NotEqual::new(0, 1));
    solver.propagate().unwrap();
    solver.add_constraint(NotEqual::new(1, 2));
    solver.propagate_incremental().unwrap();
    solver.add_constraint(NotEqual::new(2, 3));
    solver.propagate_incremental().unwrap();
    solver.add_constraint(AllDifferent::new(&[0, 1, 2, 3]));
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    let inc_result = solver.solve(&mut rng).unwrap();

    // Build from scratch
    let mut solver2 = Solver::<1>::new(4, 4, SolverConfig::default());
    solver2.add_constraint(NotEqual::new(0, 1));
    solver2.add_constraint(NotEqual::new(1, 2));
    solver2.add_constraint(NotEqual::new(2, 3));
    solver2.add_constraint(AllDifferent::new(&[0, 1, 2, 3]));
    let mut rng2 = rand::rngs::SmallRng::seed_from_u64(42);
    let full_result = solver2.solve(&mut rng2).unwrap();

    // Same solution (deterministic)
    assert_eq!(inc_result.solution.values, full_result.solution.values);
}
