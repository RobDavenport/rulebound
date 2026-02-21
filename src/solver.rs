//! Main constraint solver.

use alloc::vec::Vec;
use alloc::boxed::Box;
use crate::domain::Domain;
use crate::constraint::Constraint;
use crate::config::SolverConfig;
use crate::error::{Contradiction, SolveError};
use crate::propagator::Propagator;
use rand_core::RngCore;

/// A solution: mapping from variable index to assigned value.
#[derive(Debug, Clone)]
pub struct Solution {
    pub values: Vec<usize>,
}

/// Result of a successful solve.
#[derive(Debug, Clone)]
pub struct SolveResult {
    pub solution: Solution,
    pub propagation_rounds: usize,
    pub backtracks: usize,
}

/// The constraint solver.
pub struct Solver<const W: usize = 2> {
    domains: Vec<Domain<W>>,
    constraints: Vec<Box<dyn Constraint<W>>>,
    propagator: Propagator,
    config: SolverConfig,
}

impl<const W: usize> Solver<W> {
    /// Create a new solver with `num_variables` variables, each with domain [0, domain_size).
    pub fn new(num_variables: usize, domain_size: usize, config: SolverConfig) -> Self {
        let domains = (0..num_variables).map(|_| Domain::full(domain_size)).collect();
        Self {
            domains,
            constraints: Vec::new(),
            propagator: Propagator::new(),
            config,
        }
    }

    /// Add a constraint to the solver.
    pub fn add_constraint(&mut self, constraint: impl Constraint<W> + 'static) {
        self.constraints.push(Box::new(constraint));
    }

    /// Fix a variable to a specific value.
    pub fn assign(&mut self, variable: usize, value: usize) -> Result<(), Contradiction> {
        todo!()
    }

    /// Run propagation only (no search).
    pub fn propagate(&mut self) -> Result<bool, Contradiction> {
        todo!()
    }

    /// Run incremental propagation after adding new constraints.
    pub fn propagate_incremental(&mut self) -> Result<bool, Contradiction> {
        todo!()
    }

    /// Full solve: propagation + backtracking search.
    pub fn solve(&mut self, rng: &mut impl RngCore) -> Result<SolveResult, SolveError> {
        todo!()
    }

    /// Get the current domain of a variable.
    pub fn domain(&self, variable: usize) -> &Domain<W> {
        &self.domains[variable]
    }

    /// Get all current domains.
    pub fn domains(&self) -> &[Domain<W>] {
        &self.domains
    }

    /// Check if all variables are solved (singleton domains).
    pub fn is_solved(&self) -> bool {
        self.domains.iter().all(|d| d.is_singleton())
    }

    /// Number of variables.
    pub fn num_variables(&self) -> usize {
        self.domains.len()
    }
}
