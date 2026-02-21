use criterion::{criterion_group, criterion_main, Criterion};
use rulebound::*;
use rand::SeedableRng;

fn sudoku_solve(c: &mut Criterion) {
    c.bench_function("sudoku_solve_easy", |b| {
        b.iter(|| {
            let mut solver = setup_sudoku();
            let givens: [(usize, usize); 17] = [
                (0, 4), (3, 7), (7, 1),
                (10, 6), (14, 3),
                (19, 8), (24, 5),
                (27, 0), (31, 6),
                (36, 2), (40, 7),
                (49, 3), (53, 8),
                (58, 1), (62, 4),
                (72, 5), (80, 0),
            ];
            for (idx, val) in givens {
                let _ = solver.assign(idx, val);
            }
            let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
            solver.solve(&mut rng)
        })
    });
}

fn map_coloring(c: &mut Criterion) {
    c.bench_function("map_coloring_australia", |b| {
        b.iter(|| {
            let mut solver = Solver::<1>::new(7, 3, SolverConfig::default());
            let edges = [(0,1),(0,2),(1,2),(1,3),(2,3),(2,4),(2,5),(3,4),(4,5)];
            for (a, b) in edges {
                solver.add_constraint(NotEqual::new(a, b));
            }
            let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
            solver.solve(&mut rng)
        })
    });
}

fn all_different_10(c: &mut Criterion) {
    c.bench_function("all_different_10_vars", |b| {
        b.iter(|| {
            let mut solver = Solver::<1>::new(10, 10, SolverConfig::default());
            solver.add_constraint(AllDifferent::new(&(0..10).collect::<Vec<_>>()));
            let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
            solver.solve(&mut rng)
        })
    });
}

fn incremental_propagation(c: &mut Criterion) {
    c.bench_function("incremental_5_constraints", |b| {
        b.iter(|| {
            let mut solver = Solver::<1>::new(6, 6, SolverConfig::default());
            solver.add_constraint(NotEqual::new(0, 1));
            solver.propagate().unwrap();
            solver.add_constraint(NotEqual::new(1, 2));
            solver.propagate_incremental().unwrap();
            solver.add_constraint(NotEqual::new(2, 3));
            solver.propagate_incremental().unwrap();
            solver.add_constraint(NotEqual::new(3, 4));
            solver.propagate_incremental().unwrap();
            solver.add_constraint(NotEqual::new(4, 5));
            solver.propagate_incremental().unwrap();
        })
    });
}

fn setup_sudoku() -> Solver<1> {
    let mut solver = Solver::<1>::new(81, 9, SolverConfig::default());
    for row in 0..9 {
        let vars: Vec<usize> = (0..9).map(|col| row * 9 + col).collect();
        solver.add_constraint(AllDifferent::new(&vars));
    }
    for col in 0..9 {
        let vars: Vec<usize> = (0..9).map(|row| row * 9 + col).collect();
        solver.add_constraint(AllDifferent::new(&vars));
    }
    for box_row in 0..3 {
        for box_col in 0..3 {
            let mut vars = Vec::new();
            for r in 0..3 {
                for c in 0..3 {
                    vars.push((box_row * 3 + r) * 9 + (box_col * 3 + c));
                }
            }
            solver.add_constraint(AllDifferent::new(&vars));
        }
    }
    solver
}

criterion_group!(benches, sudoku_solve, map_coloring, all_different_10, incremental_propagation);
criterion_main!(benches);
