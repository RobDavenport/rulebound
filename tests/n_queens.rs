use rulebound::*;
use rand::SeedableRng;

fn solve_n_queens(n: usize) -> SolveResult {
    // N variables (one per row), domain {0..N-1} (column positions)
    let mut solver = Solver::<1>::new(n, n, SolverConfig::default());

    // No two queens in same column
    solver.add_constraint(AllDifferent::new(&(0..n).collect::<Vec<_>>()));

    // Diagonal constraints: |col_i - col_j| != |i - j|
    // Use Table constraint for each pair
    for i in 0..n {
        for j in (i+1)..n {
            let diff = j - i;
            let mut allowed = Vec::new();
            for ci in 0..n {
                for cj in 0..n {
                    if ci != cj
                        && (ci as isize - cj as isize).unsigned_abs() != diff
                    {
                        allowed.push(vec![ci, cj]);
                    }
                }
            }
            solver.add_constraint(Table::new(&[i, j], allowed));
        }
    }

    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    solver.solve(&mut rng).unwrap()
}

#[test]
fn four_queens() {
    let result = solve_n_queens(4);
    let cols = &result.solution.values;
    assert_eq!(cols.len(), 4);
    // Verify no conflicts
    for i in 0..4 {
        for j in (i+1)..4 {
            assert_ne!(cols[i], cols[j], "same column");
            assert_ne!((cols[i] as isize - cols[j] as isize).unsigned_abs(), j - i, "diagonal");
        }
    }
}

#[test]
fn eight_queens() {
    let result = solve_n_queens(8);
    let cols = &result.solution.values;
    for i in 0..8 {
        for j in (i+1)..8 {
            assert_ne!(cols[i], cols[j], "same column");
            assert_ne!((cols[i] as isize - cols[j] as isize).unsigned_abs(), j - i, "diagonal");
        }
    }
}
