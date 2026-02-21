//! Solver configuration.

use crate::backtrack::BacktrackStrategy;

/// Variable selection heuristic for search.
#[derive(Debug, Clone, Copy)]
pub enum Heuristic {
    /// Select the variable with the smallest domain (MRV).
    MinDomain,
    /// Select the first unsolved variable.
    FirstUnsolved,
}

impl Default for Heuristic {
    fn default() -> Self {
        Self::MinDomain
    }
}

/// Configuration for the constraint solver.
#[derive(Debug, Clone)]
pub struct SolverConfig {
    /// Maximum propagation steps before giving up.
    pub max_propagation_steps: usize,
    /// Variable selection heuristic for search.
    pub heuristic: Heuristic,
    /// Backtracking strategy.
    pub backtrack: BacktrackStrategy,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            max_propagation_steps: 100_000,
            heuristic: Heuristic::default(),
            backtrack: BacktrackStrategy::default(),
        }
    }
}

impl SolverConfig {
    /// Create a new config with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum propagation steps.
    pub fn max_propagation_steps(mut self, n: usize) -> Self {
        self.max_propagation_steps = n;
        self
    }

    /// Set the variable selection heuristic.
    pub fn heuristic(mut self, h: Heuristic) -> Self {
        self.heuristic = h;
        self
    }

    /// Set the backtracking strategy.
    pub fn backtrack(mut self, b: BacktrackStrategy) -> Self {
        self.backtrack = b;
        self
    }
}
