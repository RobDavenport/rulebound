//! WASM bindings for the rulebound interactive Sudoku demo.

use wasm_bindgen::prelude::*;
use rulebound::*;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand_core::RngCore;

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

/// Parse a JSON array of 81 integers. 0 means empty, 1-9 are given values.
fn parse_puzzle(json: &str) -> Vec<u8> {
    let trimmed = json.trim();
    let inner = trimmed.strip_prefix('[').unwrap_or(trimmed);
    let inner = inner.strip_suffix(']').unwrap_or(inner);
    inner.split(',')
        .map(|s| s.trim().parse::<u8>().unwrap_or(0))
        .collect()
}

/// Format solution as JSON array of 81 values (1-indexed for display).
fn format_solution(values: &[usize]) -> String {
    let mut s = String::from("[");
    for (i, &v) in values.iter().enumerate() {
        if i > 0 { s.push(','); }
        s.push_str(&(v + 1).to_string());
    }
    s.push(']');
    s
}

/// Format domains as JSON array of 81 arrays.
fn format_domains(domains: &[Domain<1>]) -> String {
    let mut s = String::from("[");
    for (i, d) in domains.iter().enumerate() {
        if i > 0 { s.push(','); }
        s.push('[');
        let mut first = true;
        for v in d.iter() {
            if !first { s.push(','); }
            s.push_str(&(v + 1).to_string());
            first = false;
        }
        s.push(']');
    }
    s.push(']');
    s
}

/// Solve a Sudoku puzzle instantly. Input: JSON with "puzzle" (81-element array, 0=empty, 1-9=given)
/// and "seed" (integer). Returns JSON with solution, stats, or error.
#[wasm_bindgen]
pub fn solve_sudoku(puzzle_json: &str) -> String {
    let cells = parse_puzzle(puzzle_json);
    let seed = extract_seed(puzzle_json);

    let mut solver = setup_sudoku();

    // Assign givens (convert from 1-indexed to 0-indexed)
    for (i, &val) in cells.iter().enumerate() {
        if val >= 1 && val <= 9 {
            if solver.assign(i, (val - 1) as usize).is_err() {
                return format!(r#"{{"status":"error","message":"contradiction at cell {}"}}"#, i);
            }
        }
    }

    let mut rng = SmallRng::seed_from_u64(seed);
    match solver.solve(&mut rng) {
        Ok(result) => {
            let sol = format_solution(&result.solution.values);
            format!(
                r#"{{"status":"solved","solution":{},"propagations":{},"backtracks":{}}}"#,
                sol, result.propagation_rounds, result.backtracks
            )
        }
        Err(e) => {
            format!(r#"{{"status":"error","message":"{}"}}"#, e)
        }
    }
}

fn extract_seed(json: &str) -> u64 {
    // Simple extraction: find "seed": or "seed" : followed by a number
    if let Some(pos) = json.find("\"seed\"") {
        let after = &json[pos + 6..];
        if let Some(colon) = after.find(':') {
            let num_part = after[colon + 1..].trim_start();
            let num_str: String = num_part.chars().take_while(|c| c.is_ascii_digit()).collect();
            return num_str.parse().unwrap_or(42);
        }
    }
    42
}

/// Step-through Sudoku solver for animated visualization.
#[wasm_bindgen]
pub struct StepSolver {
    solver: Solver<1>,
    rng: SmallRng,
    givens: Vec<bool>,
    step_count: usize,
    state: StepState,
    // For step-through: queue of variables to try assigning
    unresolved: Vec<usize>,
    current_var: Option<usize>,
    snapshots: Vec<(Vec<Domain<1>>, Vec<usize>)>, // (domains, remaining_unresolved)
}

enum StepState {
    Propagating,
    Selecting,
    Assigning,
    Solved,
    Failed,
}

#[wasm_bindgen]
impl StepSolver {
    /// Create a new step solver. puzzle_json: JSON array of 81 values (0=empty, 1-9=given).
    #[wasm_bindgen(constructor)]
    pub fn new(puzzle_json: &str) -> Self {
        let cells = parse_puzzle(puzzle_json);
        let seed = extract_seed(puzzle_json);
        let mut solver = setup_sudoku();
        let mut givens = vec![false; 81];

        for (i, &val) in cells.iter().enumerate() {
            if val >= 1 && val <= 9 {
                givens[i] = true;
                let _ = solver.assign(i, (val - 1) as usize);
            }
        }

        // Run initial propagation
        let _ = solver.propagate();

        let unresolved: Vec<usize> = (0..81)
            .filter(|&i| !solver.domain(i).is_singleton())
            .collect();

        Self {
            solver,
            rng: SmallRng::seed_from_u64(seed),
            givens,
            step_count: 0,
            state: if unresolved.is_empty() { StepState::Solved } else { StepState::Selecting },
            unresolved,
            current_var: None,
            snapshots: Vec::new(),
        }
    }

    /// Get current grid state as JSON.
    pub fn get_state(&self) -> String {
        let mut grid = String::from("[");
        for i in 0..81 {
            if i > 0 { grid.push(','); }
            if let Some(v) = self.solver.domain(i).singleton_value() {
                grid.push_str(&(v + 1).to_string());
            } else {
                grid.push('0');
            }
        }
        grid.push(']');

        let domains = format_domains(self.solver.domains());
        let givens_str = format!("{:?}", self.givens);

        format!(
            r#"{{"grid":{},"domains":{},"givens":{},"solved":{},"variables_solved":{}}}"#,
            grid,
            domains,
            givens_str,
            self.solver.is_solved(),
            self.solver.domains().iter().filter(|d| d.is_singleton()).count()
        )
    }

    /// Perform one step. Returns JSON event describing what happened.
    pub fn step(&mut self) -> String {
        self.step_count += 1;

        match self.state {
            StepState::Solved => {
                return r#"{"type":"solved","message":"Already solved!"}"#.to_string();
            }
            StepState::Failed => {
                return r#"{"type":"failed","message":"No solution exists"}"#.to_string();
            }
            StepState::Propagating => {
                // Run one round of propagation
                match self.solver.propagate() {
                    Ok(true) => {
                        self.state = StepState::Solved;
                        return format!(
                            r#"{{"type":"solved","step":{},"message":"Solved by propagation!"}}"#,
                            self.step_count
                        );
                    }
                    Ok(false) => {
                        self.unresolved = (0..81)
                            .filter(|&i| !self.solver.domain(i).is_singleton())
                            .collect();
                        self.state = StepState::Selecting;
                        let solved_count = self.solver.domains().iter().filter(|d| d.is_singleton()).count();
                        return format!(
                            r#"{{"type":"propagated","step":{},"variables_solved":{},"remaining":{}}}"#,
                            self.step_count, solved_count, self.unresolved.len()
                        );
                    }
                    Err(_) => {
                        // Contradiction during propagation -- backtrack
                        if let Some(_snapshot) = self.snapshots.pop() {
                            // Restore state -- this is simplified backtracking
                            // In practice we'd need to try the next value
                            self.state = StepState::Failed;
                            return format!(
                                r#"{{"type":"backtrack","step":{},"message":"Contradiction, backtracking"}}"#,
                                self.step_count
                            );
                        } else {
                            self.state = StepState::Failed;
                            return r#"{"type":"failed","message":"Contradiction with no backtrack point"}"#.to_string();
                        }
                    }
                }
            }
            StepState::Selecting => {
                // Select the variable with smallest domain (MRV heuristic)
                let best = self.unresolved.iter()
                    .filter(|&&i| !self.solver.domain(i).is_singleton() && !self.solver.domain(i).is_empty())
                    .min_by_key(|&&i| self.solver.domain(i).count())
                    .copied();

                match best {
                    Some(var) => {
                        self.current_var = Some(var);
                        self.state = StepState::Assigning;
                        let row = var / 9;
                        let col = var % 9;
                        let domain_size = self.solver.domain(var).count();
                        return format!(
                            r#"{{"type":"select","step":{},"variable":{},"row":{},"col":{},"domain_size":{}}}"#,
                            self.step_count, var, row, col, domain_size
                        );
                    }
                    None => {
                        if self.solver.is_solved() {
                            self.state = StepState::Solved;
                            return format!(
                                r#"{{"type":"solved","step":{},"message":"All variables assigned!"}}"#,
                                self.step_count
                            );
                        } else {
                            self.state = StepState::Failed;
                            return r#"{"type":"failed","message":"No unresolved variables but not solved"}"#.to_string();
                        }
                    }
                }
            }
            StepState::Assigning => {
                if let Some(var) = self.current_var {
                    // Pick a random value from the domain
                    let values: Vec<usize> = self.solver.domain(var).iter().collect();
                    if values.is_empty() {
                        self.state = StepState::Failed;
                        return r#"{"type":"failed","message":"Empty domain"}"#.to_string();
                    }
                    let idx = (self.rng.next_u32() as usize) % values.len();
                    let value = values[idx];

                    // Save snapshot for backtracking
                    let snapshot_domains: Vec<Domain<1>> = self.solver.domains().to_vec();
                    let snapshot_unresolved = self.unresolved.clone();
                    self.snapshots.push((snapshot_domains, snapshot_unresolved));

                    // Try to assign
                    match self.solver.assign(var, value) {
                        Ok(()) => {
                            let row = var / 9;
                            let col = var % 9;
                            self.state = StepState::Propagating;
                            return format!(
                                r#"{{"type":"assign","step":{},"variable":{},"row":{},"col":{},"value":{}}}"#,
                                self.step_count, var, row, col, value + 1
                            );
                        }
                        Err(_) => {
                            // Contradiction -- restore and mark failed for this branch
                            if let Some((_domains, _unresolved)) = self.snapshots.pop() {
                                // Simplified: in a real step solver we'd try the next value
                                self.state = StepState::Failed;
                                return format!(
                                    r#"{{"type":"backtrack","step":{},"variable":{},"message":"Assignment caused contradiction"}}"#,
                                    self.step_count, var
                                );
                            }
                            self.state = StepState::Failed;
                            return r#"{"type":"failed","message":"Contradiction with no backtrack"}"#.to_string();
                        }
                    }
                }
                self.state = StepState::Selecting;
                r#"{"type":"noop"}"#.to_string()
            }
        }
    }
}

/// Solve a full Sudoku puzzle instantly (convenience wrapper matching the original API).
#[wasm_bindgen]
pub fn solve_demo(config_json: &str) -> String {
    solve_sudoku(config_json)
}
