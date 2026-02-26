//! WASM bindings for the rulebound interactive demo.
//!
//! Provides both instant solving and step-through visualization for
//! Sudoku and Map Coloring modes.

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

/// Parse Sudoku cells from either:
/// - a raw JSON array: `[0,0,...]`
/// - an object containing a puzzle array/string: `{"puzzle":[...],"seed":42}`
fn parse_puzzle(json: &str) -> Vec<u8> {
    extract_array_for_key(json, "\"puzzle\"")
        .or_else(|| extract_first_array(json))
        .map(parse_puzzle_array)
        .unwrap_or_default()
}

fn parse_puzzle_array(array_json: &str) -> Vec<u8> {
    let trimmed = array_json.trim();
    let inner = trimmed.strip_prefix('[').unwrap_or(trimmed);
    let inner = inner.strip_suffix(']').unwrap_or(inner);
    inner.split(',')
        .map(|s| s.trim().trim_matches('"').parse::<u8>().unwrap_or(0))
        .collect()
}

fn extract_array_for_key<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let key_pos = json.find(key)?;
    let after_key = &json[key_pos + key.len()..];
    let rel_start = after_key.find('[')?;
    extract_balanced_array(&json[key_pos + key.len() + rel_start..])
}

fn extract_first_array(json: &str) -> Option<&str> {
    let start = json.find('[')?;
    extract_balanced_array(&json[start..])
}

fn extract_balanced_array(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    if bytes.first().copied()? != b'[' {
        return None;
    }

    let mut depth = 0usize;
    for (idx, &b) in bytes.iter().enumerate() {
        match b {
            b'[' => depth += 1,
            b']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(&s[..=idx]);
                }
            }
            _ => {}
        }
    }

    None
}

/// Format solution as JSON array of values (1-indexed for display).
fn format_solution(values: &[usize]) -> String {
    let mut s = String::from("[");
    for (i, &v) in values.iter().enumerate() {
        if i > 0 { s.push(','); }
        s.push_str(&(v + 1).to_string());
    }
    s.push(']');
    s
}

/// Format a single domain as a JSON array.
fn format_domain(d: &Domain<1>) -> String {
    let mut s = String::from("[");
    let mut first = true;
    for v in d.iter() {
        if !first { s.push(','); }
        s.push_str(&(v + 1).to_string());
        first = false;
    }
    s.push(']');
    s
}

/// Format domains as JSON array of arrays.
fn format_domains(domains: &[Domain<1>]) -> String {
    let mut s = String::from("[");
    for (i, d) in domains.iter().enumerate() {
        if i > 0 { s.push(','); }
        s.push_str(&format_domain(d));
    }
    s.push(']');
    s
}

fn extract_seed(json: &str) -> u64 {
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

// ─── Sudoku: Instant Solve ──────────────────────────────────────────────────

/// Solve a Sudoku puzzle instantly. Input: JSON with "puzzle" (81-element array, 0=empty, 1-9=given)
/// and "seed" (integer). Returns JSON with solution, stats, or error.
#[wasm_bindgen]
pub fn solve_sudoku(puzzle_json: &str) -> String {
    let cells = parse_puzzle(puzzle_json);
    let seed = extract_seed(puzzle_json);

    let mut solver = setup_sudoku();

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

/// Convenience wrapper matching the original API.
#[wasm_bindgen]
pub fn solve_demo(config_json: &str) -> String {
    solve_sudoku(config_json)
}

// ─── Map Coloring: Instant Solve ────────────────────────────────────────────

/// Solve a map coloring problem. Input JSON: {"nodes":N,"edges":[[a,b],...],"colors":C,"seed":S}
/// Returns JSON with solution (color per node) or error.
#[wasm_bindgen]
pub fn solve_map_coloring(config_json: &str) -> String {
    let seed = extract_seed(config_json);
    let (num_nodes, num_colors, edges) = parse_map_config(config_json);

    let mut solver = Solver::<1>::new(num_nodes, num_colors, SolverConfig::default());
    for &(a, b) in &edges {
        solver.add_constraint(NotEqual::new(a, b));
    }

    let mut rng = SmallRng::seed_from_u64(seed);
    match solver.solve(&mut rng) {
        Ok(result) => {
            let mut vals = String::from("[");
            for (i, &v) in result.solution.values.iter().enumerate() {
                if i > 0 { vals.push(','); }
                vals.push_str(&v.to_string());
            }
            vals.push(']');
            format!(
                r#"{{"status":"solved","solution":{},"propagations":{},"backtracks":{}}}"#,
                vals, result.propagation_rounds, result.backtracks
            )
        }
        Err(e) => {
            format!(r#"{{"status":"error","message":"{}"}}"#, e)
        }
    }
}

fn parse_map_config(json: &str) -> (usize, usize, Vec<(usize, usize)>) {
    // Simple parser: extract nodes, colors, and edges from JSON
    let nodes = extract_number(json, "\"nodes\"").unwrap_or(7);
    let colors = extract_number(json, "\"colors\"").unwrap_or(3);

    let mut edges = Vec::new();
    if let Some(pos) = json.find("\"edges\"") {
        let after = &json[pos..];
        if let Some(arr_start) = after.find("[[") {
            let inner = &after[arr_start + 1..];
            // Parse pairs like [a,b],[c,d],...
            let mut i = 0;
            while i < inner.len() {
                if inner.as_bytes()[i] == b'[' {
                    let end = inner[i..].find(']').map(|e| i + e).unwrap_or(inner.len());
                    let pair = &inner[i + 1..end];
                    let nums: Vec<usize> = pair.split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();
                    if nums.len() == 2 {
                        edges.push((nums[0], nums[1]));
                    }
                    i = end + 1;
                } else if inner.as_bytes()[i] == b']' && (i + 1 >= inner.len() || inner.as_bytes()[i + 1] != b',') {
                    break;
                } else {
                    i += 1;
                }
            }
        }
    }

    (nodes, colors, edges)
}

fn extract_number(json: &str, key: &str) -> Option<usize> {
    if let Some(pos) = json.find(key) {
        let after = &json[pos + key.len()..];
        if let Some(colon) = after.find(':') {
            let num_part = after[colon + 1..].trim_start();
            let num_str: String = num_part.chars().take_while(|c| c.is_ascii_digit()).collect();
            return num_str.parse().ok();
        }
    }
    None
}

// ─── StepSolver: Step-through Sudoku ────────────────────────────────────────

/// A snapshot frame for backtracking in the step solver.
struct BacktrackFrame {
    domains: Vec<Domain<1>>,
    variable: usize,
    remaining_values: Vec<usize>,
}

/// Step-through Sudoku solver for animated visualization.
#[wasm_bindgen]
pub struct StepSolver {
    solver: Solver<1>,
    rng: SmallRng,
    givens: Vec<bool>,
    step_count: usize,
    state: StepState,
    unresolved: Vec<usize>,
    current_var: Option<usize>,
    backtrack_stack: Vec<BacktrackFrame>,
    depth: usize,
}

enum StepState {
    Propagating,
    Selecting,
    Assigning,
    Backtracking,
    Solved,
    Failed,
}

#[wasm_bindgen]
impl StepSolver {
    /// Create a new step solver. puzzle_json: JSON with "puzzle" array and optional "seed".
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
            backtrack_stack: Vec::new(),
            depth: 0,
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

    /// Perform one step. Returns JSON event matching the spec format.
    pub fn step(&mut self) -> String {
        self.step_count += 1;

        match self.state {
            StepState::Solved => {
                let values = format_solution(
                    &self.solver.domains().iter()
                        .map(|d| d.singleton_value().unwrap_or(0))
                        .collect::<Vec<_>>()
                );
                return format!(r#"{{"type":"solved","values":{}}}"#, values);
            }
            StepState::Failed => {
                return r#"{"type":"contradiction"}"#.to_string();
            }
            StepState::Backtracking => {
                // Try next value from backtrack frame
                if let Some(frame) = self.backtrack_stack.last_mut() {
                    if let Some(value) = frame.remaining_values.pop() {
                        // Restore domains from frame snapshot
                        let restored_domains = frame.domains.clone();
                        let var = frame.variable;
                        self.solver.restore_domains(restored_domains);

                        // Assign the next value
                        match self.solver.assign(var, value) {
                            Ok(()) => {
                                self.state = StepState::Propagating;
                                let domain = format_domain(self.solver.domain(var));
                                return format!(
                                    r#"{{"type":"assigned","variable":{},"value":{},"domain":{}}}"#,
                                    var, value + 1, domain
                                );
                            }
                            Err(_) => {
                                // This value also contradicts, restore and keep backtracking
                                let restored = frame.domains.clone();
                                self.solver.restore_domains(restored);
                                if frame.remaining_values.is_empty() {
                                    self.backtrack_stack.pop();
                                    self.depth = self.depth.saturating_sub(1);
                                    if self.backtrack_stack.is_empty() {
                                        self.state = StepState::Failed;
                                        return r#"{"type":"contradiction"}"#.to_string();
                                    }
                                }
                                return format!(
                                    r#"{{"type":"backtrack","depth":{}}}"#,
                                    self.depth
                                );
                            }
                        }
                    } else {
                        // No more values for this frame — pop and go deeper
                        let restored = frame.domains.clone();
                        self.solver.restore_domains(restored);
                        self.backtrack_stack.pop();
                        self.depth = self.depth.saturating_sub(1);
                        if self.backtrack_stack.is_empty() {
                            self.state = StepState::Failed;
                            return r#"{"type":"contradiction"}"#.to_string();
                        }
                        return format!(
                            r#"{{"type":"backtrack","depth":{}}}"#,
                            self.depth
                        );
                    }
                } else {
                    self.state = StepState::Failed;
                    return r#"{"type":"contradiction"}"#.to_string();
                }
            }
            StepState::Propagating => {
                match self.solver.propagate() {
                    Ok(true) => {
                        self.state = StepState::Solved;
                        let values = format_solution(
                            &self.solver.domains().iter()
                                .map(|d| d.singleton_value().unwrap_or(0))
                                .collect::<Vec<_>>()
                        );
                        return format!(r#"{{"type":"solved","values":{}}}"#, values);
                    }
                    Ok(false) => {
                        // Find the variable whose domain changed most recently
                        // (use MRV heuristic variable as the representative)
                        self.unresolved = (0..81)
                            .filter(|&i| !self.solver.domain(i).is_singleton())
                            .collect();

                        // Report the most constrained unresolved variable
                        let report_var = self.unresolved.iter()
                            .min_by_key(|&&i| self.solver.domain(i).count())
                            .copied()
                            .unwrap_or(0);

                        let domain = format_domain(self.solver.domain(report_var));
                        self.state = StepState::Selecting;
                        return format!(
                            r#"{{"type":"propagated","variable":{},"domain":{}}}"#,
                            report_var, domain
                        );
                    }
                    Err(_) => {
                        // Contradiction during propagation — backtrack
                        self.state = StepState::Backtracking;
                        return format!(
                            r#"{{"type":"backtrack","depth":{}}}"#,
                            self.depth
                        );
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
                        // Immediately proceed to assigning (selecting is internal)
                        return self.do_assign(var);
                    }
                    None => {
                        if self.solver.is_solved() {
                            self.state = StepState::Solved;
                            let values = format_solution(
                                &self.solver.domains().iter()
                                    .map(|d| d.singleton_value().unwrap_or(0))
                                    .collect::<Vec<_>>()
                            );
                            return format!(r#"{{"type":"solved","values":{}}}"#, values);
                        } else {
                            // Dead end — backtrack
                            self.state = StepState::Backtracking;
                            return format!(
                                r#"{{"type":"backtrack","depth":{}}}"#,
                                self.depth
                            );
                        }
                    }
                }
            }
            StepState::Assigning => {
                if let Some(var) = self.current_var {
                    return self.do_assign(var);
                }
                self.state = StepState::Selecting;
                return self.step();
            }
        }
    }
}

impl StepSolver {
    fn do_assign(&mut self, var: usize) -> String {
        let mut values: Vec<usize> = self.solver.domain(var).iter().collect();
        if values.is_empty() {
            self.state = StepState::Backtracking;
            return format!(r#"{{"type":"backtrack","depth":{}}}"#, self.depth);
        }

        // Shuffle values
        for i in (1..values.len()).rev() {
            let j = (self.rng.next_u32() as usize) % (i + 1);
            values.swap(i, j);
        }

        let value = values.pop().unwrap();

        // Save snapshot for backtracking (remaining_values are what we haven't tried yet)
        let snapshot_domains: Vec<Domain<1>> = self.solver.domains().to_vec();
        self.backtrack_stack.push(BacktrackFrame {
            domains: snapshot_domains,
            variable: var,
            remaining_values: values, // remaining untried values
        });
        self.depth += 1;

        // Try to assign
        match self.solver.assign(var, value) {
            Ok(()) => {
                let domain = format_domain(self.solver.domain(var));
                self.state = StepState::Propagating;
                format!(
                    r#"{{"type":"assigned","variable":{},"value":{},"domain":{}}}"#,
                    var, value + 1, domain
                )
            }
            Err(_) => {
                // Contradiction — will backtrack on next step
                self.state = StepState::Backtracking;
                format!(r#"{{"type":"backtrack","depth":{}}}"#, self.depth)
            }
        }
    }
}

// ─── MapStepSolver: Step-through Map Coloring ───────────────────────────────

/// Step-through Map Coloring solver for animated visualization.
#[wasm_bindgen]
pub struct MapStepSolver {
    solver: Solver<1>,
    rng: SmallRng,
    num_nodes: usize,
    step_count: usize,
    state: StepState,
    unresolved: Vec<usize>,
    current_var: Option<usize>,
    backtrack_stack: Vec<BacktrackFrame>,
    depth: usize,
}

#[wasm_bindgen]
impl MapStepSolver {
    /// Create a new map coloring step solver. config_json: {"nodes":N,"edges":[[a,b],...],"colors":C,"seed":S}
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str) -> Self {
        let seed = extract_seed(config_json);
        let (num_nodes, num_colors, edges) = parse_map_config(config_json);

        let mut solver = Solver::<1>::new(num_nodes, num_colors, SolverConfig::default());
        for &(a, b) in &edges {
            solver.add_constraint(NotEqual::new(a, b));
        }

        let _ = solver.propagate();

        let unresolved: Vec<usize> = (0..num_nodes)
            .filter(|&i| !solver.domain(i).is_singleton())
            .collect();

        Self {
            solver,
            rng: SmallRng::seed_from_u64(seed),
            num_nodes,
            step_count: 0,
            state: if unresolved.is_empty() { StepState::Solved } else { StepState::Selecting },
            unresolved,
            current_var: None,
            backtrack_stack: Vec::new(),
            depth: 0,
        }
    }

    /// Get current state as JSON: colors array and domains.
    pub fn get_state(&self) -> String {
        let mut colors = String::from("[");
        for i in 0..self.num_nodes {
            if i > 0 { colors.push(','); }
            if let Some(v) = self.solver.domain(i).singleton_value() {
                colors.push_str(&v.to_string());
            } else {
                colors.push_str("-1");
            }
        }
        colors.push(']');

        let domains = format_domains(self.solver.domains());

        format!(
            r#"{{"colors":{},"domains":{},"solved":{},"variables_solved":{}}}"#,
            colors,
            domains,
            self.solver.is_solved(),
            self.solver.domains().iter().filter(|d| d.is_singleton()).count()
        )
    }

    /// Perform one step. Returns JSON event matching the spec format.
    pub fn step(&mut self) -> String {
        self.step_count += 1;

        match self.state {
            StepState::Solved => {
                let mut vals = String::from("[");
                for i in 0..self.num_nodes {
                    if i > 0 { vals.push(','); }
                    vals.push_str(&self.solver.domain(i).singleton_value().unwrap_or(0).to_string());
                }
                vals.push(']');
                return format!(r#"{{"type":"solved","values":{}}}"#, vals);
            }
            StepState::Failed => {
                return r#"{"type":"contradiction"}"#.to_string();
            }
            StepState::Backtracking => {
                return self.do_backtrack();
            }
            StepState::Propagating => {
                match self.solver.propagate() {
                    Ok(true) => {
                        self.state = StepState::Solved;
                        let mut vals = String::from("[");
                        for i in 0..self.num_nodes {
                            if i > 0 { vals.push(','); }
                            vals.push_str(&self.solver.domain(i).singleton_value().unwrap_or(0).to_string());
                        }
                        vals.push(']');
                        return format!(r#"{{"type":"solved","values":{}}}"#, vals);
                    }
                    Ok(false) => {
                        self.unresolved = (0..self.num_nodes)
                            .filter(|&i| !self.solver.domain(i).is_singleton())
                            .collect();
                        let report_var = self.unresolved.iter()
                            .min_by_key(|&&i| self.solver.domain(i).count())
                            .copied()
                            .unwrap_or(0);
                        let domain = format_domain(self.solver.domain(report_var));
                        self.state = StepState::Selecting;
                        return format!(
                            r#"{{"type":"propagated","variable":{},"domain":{}}}"#,
                            report_var, domain
                        );
                    }
                    Err(_) => {
                        self.state = StepState::Backtracking;
                        return format!(r#"{{"type":"backtrack","depth":{}}}"#, self.depth);
                    }
                }
            }
            StepState::Selecting => {
                let best = self.unresolved.iter()
                    .filter(|&&i| !self.solver.domain(i).is_singleton() && !self.solver.domain(i).is_empty())
                    .min_by_key(|&&i| self.solver.domain(i).count())
                    .copied();

                match best {
                    Some(var) => {
                        self.current_var = Some(var);
                        self.state = StepState::Assigning;
                        return self.do_assign(var);
                    }
                    None => {
                        if self.solver.is_solved() {
                            self.state = StepState::Solved;
                            let mut vals = String::from("[");
                            for i in 0..self.num_nodes {
                                if i > 0 { vals.push(','); }
                                vals.push_str(&self.solver.domain(i).singleton_value().unwrap_or(0).to_string());
                            }
                            vals.push(']');
                            return format!(r#"{{"type":"solved","values":{}}}"#, vals);
                        } else {
                            self.state = StepState::Backtracking;
                            return format!(r#"{{"type":"backtrack","depth":{}}}"#, self.depth);
                        }
                    }
                }
            }
            StepState::Assigning => {
                if let Some(var) = self.current_var {
                    return self.do_assign(var);
                }
                self.state = StepState::Selecting;
                return self.step();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_puzzle_from_raw_array() {
        let cells = parse_puzzle("[1,0,9]");
        assert_eq!(cells, vec![1, 0, 9]);
    }

    #[test]
    fn parse_puzzle_from_object_array() {
        let cells = parse_puzzle(r#"{"puzzle":[1,0,9],"seed":7}"#);
        assert_eq!(cells, vec![1, 0, 9]);
    }

    #[test]
    fn parse_puzzle_from_object_string_array() {
        let cells = parse_puzzle(r#"{"puzzle":"[1,0,9]","seed":7}"#);
        assert_eq!(cells, vec![1, 0, 9]);
    }

    #[test]
    fn extract_seed_from_object() {
        assert_eq!(extract_seed(r#"{"puzzle":[0,0,0],"seed":1234}"#), 1234);
    }
}

impl MapStepSolver {
    fn do_assign(&mut self, var: usize) -> String {
        let mut values: Vec<usize> = self.solver.domain(var).iter().collect();
        if values.is_empty() {
            self.state = StepState::Backtracking;
            return format!(r#"{{"type":"backtrack","depth":{}}}"#, self.depth);
        }

        for i in (1..values.len()).rev() {
            let j = (self.rng.next_u32() as usize) % (i + 1);
            values.swap(i, j);
        }

        let value = values.pop().unwrap();
        let snapshot_domains: Vec<Domain<1>> = self.solver.domains().to_vec();
        self.backtrack_stack.push(BacktrackFrame {
            domains: snapshot_domains,
            variable: var,
            remaining_values: values,
        });
        self.depth += 1;

        match self.solver.assign(var, value) {
            Ok(()) => {
                let domain = format_domain(self.solver.domain(var));
                self.state = StepState::Propagating;
                format!(
                    r#"{{"type":"assigned","variable":{},"value":{},"domain":{}}}"#,
                    var, value, domain
                )
            }
            Err(_) => {
                self.state = StepState::Backtracking;
                format!(r#"{{"type":"backtrack","depth":{}}}"#, self.depth)
            }
        }
    }

    fn do_backtrack(&mut self) -> String {
        if let Some(frame) = self.backtrack_stack.last_mut() {
            if let Some(value) = frame.remaining_values.pop() {
                let restored_domains = frame.domains.clone();
                let var = frame.variable;
                self.solver.restore_domains(restored_domains);

                match self.solver.assign(var, value) {
                    Ok(()) => {
                        self.state = StepState::Propagating;
                        let domain = format_domain(self.solver.domain(var));
                        return format!(
                            r#"{{"type":"assigned","variable":{},"value":{},"domain":{}}}"#,
                            var, value, domain
                        );
                    }
                    Err(_) => {
                        let restored = frame.domains.clone();
                        self.solver.restore_domains(restored);
                        if frame.remaining_values.is_empty() {
                            self.backtrack_stack.pop();
                            self.depth = self.depth.saturating_sub(1);
                            if self.backtrack_stack.is_empty() {
                                self.state = StepState::Failed;
                                return r#"{"type":"contradiction"}"#.to_string();
                            }
                        }
                        return format!(r#"{{"type":"backtrack","depth":{}}}"#, self.depth);
                    }
                }
            } else {
                let restored = frame.domains.clone();
                self.solver.restore_domains(restored);
                self.backtrack_stack.pop();
                self.depth = self.depth.saturating_sub(1);
                if self.backtrack_stack.is_empty() {
                    self.state = StepState::Failed;
                    return r#"{"type":"contradiction"}"#.to_string();
                }
                return format!(r#"{{"type":"backtrack","depth":{}}}"#, self.depth);
            }
        } else {
            self.state = StepState::Failed;
            return r#"{"type":"contradiction"}"#.to_string();
        }
    }
}
