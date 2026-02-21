//! Main constraint solver.

use alloc::vec::Vec;
use alloc::boxed::Box;
use crate::domain::Domain;
use crate::constraint::Constraint;
use crate::config::{Heuristic, SolverConfig};
use crate::domain::DomainSnapshot;
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
        self.domains[variable] = Domain::empty();
        self.domains[variable].insert(value);
        let refs: Vec<&dyn Constraint<W>> = self.constraints.iter().map(|c| &**c).collect();
        self.propagator.propagate(&mut self.domains, &refs)?;
        Ok(())
    }

    /// Run propagation only (no search).
    pub fn propagate(&mut self) -> Result<bool, Contradiction> {
        let refs: Vec<&dyn Constraint<W>> = self.constraints.iter().map(|c| &**c).collect();
        self.propagator.propagate(&mut self.domains, &refs)?;
        Ok(self.is_solved())
    }

    /// Run incremental propagation after adding new constraints.
    pub fn propagate_incremental(&mut self) -> Result<bool, Contradiction> {
        todo!()
    }

    /// Full solve: propagation + backtracking search.
    pub fn solve(&mut self, rng: &mut impl RngCore) -> Result<SolveResult, SolveError> {
        let mut rounds = 0usize;
        let mut backtracks = 0usize;

        // Initial propagation
        let refs: Vec<&dyn Constraint<W>> = self.constraints.iter().map(|c| &**c).collect();
        self.propagator.propagate(&mut self.domains, &refs)?;
        rounds += 1;

        if self.is_solved() {
            return Ok(SolveResult {
                solution: self.extract_solution(),
                propagation_rounds: rounds,
                backtracks: 0,
            });
        }

        let max_depth = match self.config.backtrack {
            crate::backtrack::BacktrackStrategy::None => 0,
            crate::backtrack::BacktrackStrategy::Chronological { max_depth } => max_depth,
            crate::backtrack::BacktrackStrategy::Restart { max_restarts } => max_restarts,
        };

        self.search(rng, &mut rounds, &mut backtracks, 0, max_depth)
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

    fn select_variable(&self) -> Option<usize> {
        match self.config.heuristic {
            Heuristic::MinDomain => {
                self.domains.iter().enumerate()
                    .filter(|(_, d)| !d.is_singleton() && !d.is_empty())
                    .min_by_key(|(_, d)| d.count())
                    .map(|(i, _)| i)
            }
            Heuristic::FirstUnsolved => {
                self.domains.iter().enumerate()
                    .find(|(_, d)| !d.is_singleton() && !d.is_empty())
                    .map(|(i, _)| i)
            }
        }
    }

    fn extract_solution(&self) -> Solution {
        Solution {
            values: self.domains.iter().map(|d| d.singleton_value().unwrap()).collect(),
        }
    }

    fn search(
        &mut self,
        rng: &mut impl RngCore,
        rounds: &mut usize,
        backtracks: &mut usize,
        depth: usize,
        max_depth: usize,
    ) -> Result<SolveResult, SolveError> {
        if depth > max_depth {
            return Err(SolveError::BacktrackLimitExceeded);
        }

        let var = match self.select_variable() {
            Some(v) => v,
            None if self.is_solved() => {
                return Ok(SolveResult {
                    solution: self.extract_solution(),
                    propagation_rounds: *rounds,
                    backtracks: *backtracks,
                });
            }
            None => return Err(SolveError::Unsatisfiable),
        };

        // Collect and shuffle values
        let mut values: Vec<usize> = self.domains[var].iter().collect();
        for i in (1..values.len()).rev() {
            let j = (rng.next_u32() as usize) % (i + 1);
            values.swap(i, j);
        }

        for value in values {
            let snapshot = DomainSnapshot { domains: self.domains.clone() };

            self.domains[var] = Domain::empty();
            self.domains[var].insert(value);

            let refs: Vec<&dyn Constraint<W>> = self.constraints.iter().map(|c| &**c).collect();
            match self.propagator.propagate(&mut self.domains, &refs) {
                Ok(_) => {
                    *rounds += 1;
                    if self.is_solved() {
                        return Ok(SolveResult {
                            solution: self.extract_solution(),
                            propagation_rounds: *rounds,
                            backtracks: *backtracks,
                        });
                    }
                    match self.search(rng, rounds, backtracks, depth + 1, max_depth) {
                        Ok(result) => return Ok(result),
                        Err(SolveError::BacktrackLimitExceeded) => {
                            return Err(SolveError::BacktrackLimitExceeded);
                        }
                        Err(_) => {
                            *backtracks += 1;
                            self.domains = snapshot.domains;
                        }
                    }
                }
                Err(_) => {
                    *backtracks += 1;
                    self.domains = snapshot.domains;
                }
            }
        }

        Err(SolveError::Unsatisfiable)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::constraint::NotEqual;
    use crate::config::SolverConfig;

    #[test]
    fn assign_propagates() {
        // 2 vars {0,1,2}, A!=B, assign A=1 → B loses 1
        let mut solver = Solver::<1>::new(2, 3, SolverConfig::default());
        solver.add_constraint(NotEqual::new(0, 1));
        solver.assign(0, 1).unwrap();
        assert_eq!(solver.domain(0).singleton_value(), Some(1));
        assert!(!solver.domain(1).contains(1));
        assert_eq!(solver.domain(1).count(), 2);
    }

    #[test]
    fn propagate_returns_solved() {
        // 2 vars, assign both → is_solved
        let mut solver = Solver::<1>::new(2, 3, SolverConfig::default());
        solver.add_constraint(NotEqual::new(0, 1));
        solver.assign(0, 0).unwrap();
        solver.assign(1, 1).unwrap();
        let solved = solver.propagate().unwrap();
        assert!(solved);
        assert!(solver.is_solved());
    }

    #[test]
    fn solve_simple_csp() {
        // 3 vars {0,1,2}, all different → finds valid assignment
        use rand::SeedableRng;
        let mut solver = Solver::<1>::new(3, 3, SolverConfig::default());
        solver.add_constraint(crate::constraint::AllDifferent::new(&[0, 1, 2]));
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let result = solver.solve(&mut rng).unwrap();
        let vals = &result.solution.values;
        // All different and in range
        assert_ne!(vals[0], vals[1]);
        assert_ne!(vals[1], vals[2]);
        assert_ne!(vals[0], vals[2]);
        for &v in vals { assert!(v < 3); }
    }

    #[test]
    fn solve_unsatisfiable() {
        // 3 vars {0,1}, all different → impossible
        use rand::SeedableRng;
        let mut solver = Solver::<1>::new(3, 2, SolverConfig::default());
        solver.add_constraint(crate::constraint::AllDifferent::new(&[0, 1, 2]));
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        assert!(solver.solve(&mut rng).is_err());
    }
}
