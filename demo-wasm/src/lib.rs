//! WASM bindings for the rulebound interactive demo.

use wasm_bindgen::prelude::*;

/// Placeholder: solve a constraint problem and return JSON result.
#[wasm_bindgen]
pub fn solve_demo(_config_json: &str) -> String {
    // TODO: Parse config, build solver, solve, return JSON
    r#"{"status":"not_implemented"}"#.to_string()
}

/// Placeholder: create a step-by-step solver.
#[wasm_bindgen]
pub struct StepSolver {
    // TODO
}

#[wasm_bindgen]
impl StepSolver {
    /// Create a new step solver from JSON config.
    #[wasm_bindgen(constructor)]
    pub fn new(_config_json: &str) -> Self {
        Self {}
    }

    /// Perform one propagation step. Returns JSON event.
    pub fn step(&mut self) -> String {
        r#"{"type":"not_implemented"}"#.to_string()
    }
}
