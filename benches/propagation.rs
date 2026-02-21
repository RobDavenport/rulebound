use criterion::{criterion_group, criterion_main, Criterion};
use rulebound::*;
use rand::SeedableRng;

fn sudoku_solve(c: &mut Criterion) {
    // Easy puzzle (~30 givens)
    c.bench_function("sudoku_solve_easy", |b| {
        b.iter(|| {
            let mut solver = setup_sudoku();
            let givens: [(usize, usize); 30] = [
                (0, 4), (1, 2), (3, 7), (5, 0), (7, 1), (8, 5),
                (10, 6), (12, 3), (14, 3), (16, 8),
                (19, 8), (21, 0), (24, 5), (25, 2),
                (27, 0), (31, 6), (34, 7),
                (36, 2), (40, 7), (44, 1),
                (49, 3), (53, 8), (46, 5),
                (58, 1), (62, 4), (63, 6),
                (72, 5), (74, 3), (76, 8), (80, 0),
            ];
            for (idx, val) in givens {
                let _ = solver.assign(idx, val);
            }
            let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
            solver.solve(&mut rng)
        })
    });

    // Medium puzzle (~25 givens)
    c.bench_function("sudoku_solve_medium", |b| {
        b.iter(|| {
            let mut solver = setup_sudoku();
            let givens: [(usize, usize); 25] = [
                (0, 4), (3, 7), (7, 1),
                (10, 6), (14, 3),
                (19, 8), (24, 5),
                (27, 0), (31, 6),
                (36, 2), (40, 7),
                (49, 3), (53, 8),
                (58, 1), (62, 4),
                (72, 5), (80, 0),
                (2, 8), (15, 1), (22, 6),
                (33, 4), (45, 7), (56, 2),
                (68, 3), (77, 5),
            ];
            for (idx, val) in givens {
                let _ = solver.assign(idx, val);
            }
            let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
            solver.solve(&mut rng)
        })
    });

    // Hard puzzle (~17 givens, minimum for unique solution)
    c.bench_function("sudoku_solve_hard", |b| {
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
    c.bench_function("map_coloring_20_nodes_4_colors", |b| {
        b.iter(|| {
            // 20-node random graph with 4 colors
            let mut solver = Solver::<1>::new(20, 4, SolverConfig::default());
            // Generate a connected random graph with ~40 edges (density ~0.2 for 20 nodes)
            let edges: [(usize, usize); 40] = [
                (0,1),(0,2),(0,5),(1,3),(1,6),(2,3),(2,7),(3,4),(3,8),
                (4,5),(4,9),(5,6),(5,10),(6,7),(6,11),(7,8),(7,12),
                (8,9),(8,13),(9,10),(9,14),(10,11),(10,15),(11,12),
                (11,16),(12,13),(12,17),(13,14),(13,18),(14,15),(14,19),
                (15,16),(16,17),(17,18),(18,19),(19,0),(0,10),(5,15),
                (1,11),(6,16),
            ];
            for (a, b) in edges {
                solver.add_constraint(NotEqual::new(a, b));
            }
            let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
            solver.solve(&mut rng)
        })
    });
}

fn all_different_50(c: &mut Criterion) {
    c.bench_function("all_different_50_vars", |b| {
        b.iter(|| {
            let mut solver = Solver::<1>::new(50, 50, SolverConfig::default());
            solver.add_constraint(AllDifferent::new(&(0..50).collect::<Vec<_>>()));
            let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
            solver.solve(&mut rng)
        })
    });
}

fn incremental_propagation(c: &mut Criterion) {
    c.bench_function("incremental_10_constraints", |b| {
        b.iter(|| {
            let mut solver = Solver::<1>::new(11, 11, SolverConfig::default());
            solver.add_constraint(NotEqual::new(0, 1));
            solver.propagate().unwrap();
            for i in 1..10 {
                solver.add_constraint(NotEqual::new(i, i + 1));
                solver.propagate_incremental().unwrap();
            }
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

criterion_group!(benches, sudoku_solve, map_coloring, all_different_50, incremental_propagation);
criterion_main!(benches);
