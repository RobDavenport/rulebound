//! Integration test: Sudoku solver
use rulebound::*;
use rand::SeedableRng;

fn setup_sudoku() -> Solver<1> {
    let mut solver = Solver::<1>::new(81, 9, SolverConfig::default());

    // Row constraints: AllDifferent for each row
    for row in 0..9 {
        let vars: Vec<usize> = (0..9).map(|col| row * 9 + col).collect();
        solver.add_constraint(AllDifferent::new(&vars));
    }
    // Column constraints
    for col in 0..9 {
        let vars: Vec<usize> = (0..9).map(|row| row * 9 + col).collect();
        solver.add_constraint(AllDifferent::new(&vars));
    }
    // 3x3 box constraints
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

fn verify_sudoku(solution: &Solution) {
    // Each row has all 9 values
    for row in 0..9 {
        let mut seen = [false; 9];
        for col in 0..9 {
            let v = solution.values[row * 9 + col];
            assert!(v < 9, "value out of range");
            assert!(!seen[v], "duplicate in row {row}");
            seen[v] = true;
        }
    }
    // Each column
    for col in 0..9 {
        let mut seen = [false; 9];
        for row in 0..9 {
            let v = solution.values[row * 9 + col];
            assert!(!seen[v], "duplicate in col {col}");
            seen[v] = true;
        }
    }
    // Each box
    for br in 0..3 {
        for bc in 0..3 {
            let mut seen = [false; 9];
            for r in 0..3 {
                for c in 0..3 {
                    let v = solution.values[(br * 3 + r) * 9 + (bc * 3 + c)];
                    assert!(!seen[v], "duplicate in box ({br},{bc})");
                    seen[v] = true;
                }
            }
        }
    }
}

#[test]
fn solve_easy_sudoku() {
    let mut solver = setup_sudoku();
    // Easy puzzle derived from a known valid Sudoku grid.
    // Values are 0-indexed (0..9 maps to 1..9 in standard notation).
    let givens: [(usize, usize); 30] = [
        (1, 2), (8, 1), (12, 0), (15, 2), (18, 0),
        (19, 8), (20, 7), (23, 1), (30, 6), (32, 0),
        (33, 3), (36, 3), (39, 7), (43, 8), (49, 1),
        (50, 3), (52, 4), (53, 5), (55, 5), (56, 0),
        (57, 4), (58, 2), (59, 6), (62, 3), (63, 1),
        (65, 6), (68, 8), (70, 2), (77, 5), (78, 0),
    ];
    for (idx, val) in givens {
        solver.assign(idx, val).unwrap();
    }
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    let result = solver.solve(&mut rng).unwrap();
    verify_sudoku(&result.solution);
}

#[test]
fn solve_empty_sudoku() {
    let mut solver = setup_sudoku();
    let mut rng = rand::rngs::SmallRng::seed_from_u64(123);
    let result = solver.solve(&mut rng).unwrap();
    verify_sudoku(&result.solution);
}
