//! AC-3 arc consistency propagation engine.

use alloc::vec::Vec;
use alloc::collections::VecDeque;
use crate::domain::Domain;
use crate::constraint::Constraint;
use crate::error::Contradiction;

/// Arc consistency propagator using the AC-3 algorithm.
pub struct Propagator {
    /// Queue of constraint indices to re-check.
    worklist: VecDeque<usize>,
}

impl Propagator {
    /// Create a new propagator.
    pub fn new() -> Self {
        Self { worklist: VecDeque::new() }
    }

    /// Run propagation to fixpoint.
    /// Returns the set of variable indices whose domains changed.
    pub fn propagate<const W: usize>(
        &mut self,
        domains: &mut [Domain<W>],
        constraints: &[&dyn Constraint<W>],
    ) -> Result<Vec<usize>, Contradiction> {
        let num_vars = domains.len();

        // Build variable -> constraint indices mapping
        let mut var_constraints: Vec<Vec<usize>> = (0..num_vars).map(|_| Vec::new()).collect();
        for (ci, c) in constraints.iter().enumerate() {
            for &v in c.scope() {
                var_constraints[v].push(ci);
            }
        }

        // Initialize worklist with all constraints
        self.worklist.clear();
        let mut in_worklist: Vec<bool> = (0..constraints.len()).map(|_| true).collect();
        for i in 0..constraints.len() {
            self.worklist.push_back(i);
        }

        let mut changed: Vec<bool> = (0..num_vars).map(|_| false).collect();

        while let Some(ci) = self.worklist.pop_front() {
            in_worklist[ci] = false;
            let constraint = constraints[ci];
            let scope = constraint.scope();

            // Snapshot domain sizes
            let old_sizes: Vec<usize> = scope.iter().map(|&v| domains[v].count()).collect();

            // Propagate
            constraint.propagate(domains)?;

            // Check for changes and re-queue
            for (i, &v) in scope.iter().enumerate() {
                if domains[v].is_empty() {
                    return Err(Contradiction { variable: v });
                }
                if domains[v].count() < old_sizes[i] {
                    changed[v] = true;
                    for &other_ci in &var_constraints[v] {
                        if other_ci != ci && !in_worklist[other_ci] {
                            self.worklist.push_back(other_ci);
                            in_worklist[other_ci] = true;
                        }
                    }
                }
            }
        }

        Ok((0..num_vars).filter(|&v| changed[v]).collect())
    }

    /// Incremental propagation: only re-check constraints involving the given variables.
    pub fn propagate_incremental<const W: usize>(
        &mut self,
        domains: &mut [Domain<W>],
        constraints: &[&dyn Constraint<W>],
        changed_vars: &[usize],
    ) -> Result<Vec<usize>, Contradiction> {
        let num_vars = domains.len();

        let mut var_constraints: Vec<Vec<usize>> = (0..num_vars).map(|_| Vec::new()).collect();
        for (ci, c) in constraints.iter().enumerate() {
            for &v in c.scope() {
                var_constraints[v].push(ci);
            }
        }

        // Only seed worklist with constraints involving changed variables
        self.worklist.clear();
        let mut in_worklist: Vec<bool> = (0..constraints.len()).map(|_| false).collect();
        for &v in changed_vars {
            for &ci in &var_constraints[v] {
                if !in_worklist[ci] {
                    self.worklist.push_back(ci);
                    in_worklist[ci] = true;
                }
            }
        }

        let mut changed: Vec<bool> = (0..num_vars).map(|_| false).collect();

        while let Some(ci) = self.worklist.pop_front() {
            in_worklist[ci] = false;
            let constraint = constraints[ci];
            let scope = constraint.scope();
            let old_sizes: Vec<usize> = scope.iter().map(|&v| domains[v].count()).collect();

            constraint.propagate(domains)?;

            for (i, &v) in scope.iter().enumerate() {
                if domains[v].is_empty() {
                    return Err(Contradiction { variable: v });
                }
                if domains[v].count() < old_sizes[i] {
                    changed[v] = true;
                    for &other_ci in &var_constraints[v] {
                        if other_ci != ci && !in_worklist[other_ci] {
                            self.worklist.push_back(other_ci);
                            in_worklist[other_ci] = true;
                        }
                    }
                }
            }
        }

        Ok((0..num_vars).filter(|&v| changed[v]).collect())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::constraint::NotEqual;

    #[test]
    fn propagate_chain() {
        // A={1}, B={1,2}, C={1,2} with A!=B, B!=C
        // After propagation: A={1}, B={2}, C={1}
        let mut domains = alloc::vec![Domain::<1>::empty(); 3];
        domains[0].insert(1);
        domains[1].insert(1); domains[1].insert(2);
        domains[2].insert(1); domains[2].insert(2);

        let c0 = NotEqual::new(0, 1);
        let c1 = NotEqual::new(1, 2);
        let constraints: std::vec::Vec<&dyn Constraint<1>> = std::vec![&c0, &c1];

        let mut prop = Propagator::new();
        let changed = prop.propagate(&mut domains, &constraints).unwrap();

        assert_eq!(domains[0].singleton_value(), Some(1));
        assert_eq!(domains[1].singleton_value(), Some(2));
        assert_eq!(domains[2].singleton_value(), Some(1));
        assert!(!changed.is_empty());
    }

    #[test]
    fn propagate_detects_contradiction() {
        // A={1}, B={1}, A!=B -> contradiction
        let mut domains = alloc::vec![Domain::<1>::empty(); 2];
        domains[0].insert(1);
        domains[1].insert(1);
        let c = NotEqual::new(0, 1);
        let constraints: std::vec::Vec<&dyn Constraint<1>> = std::vec![&c];
        let mut prop = Propagator::new();
        assert!(prop.propagate(&mut domains, &constraints).is_err());
    }

    #[test]
    fn propagate_incremental_only_touches_relevant() {
        // 3 vars, 2 constraints: A!=B, B!=C. Only seed with var B changed.
        let mut domains = alloc::vec![Domain::<1>::empty(); 3];
        domains[0].insert(1); domains[0].insert(2);
        domains[1].insert(1); // B is singleton
        domains[2].insert(1); domains[2].insert(2);

        let c0 = NotEqual::new(0, 1);
        let c1 = NotEqual::new(1, 2);
        let constraints: std::vec::Vec<&dyn Constraint<1>> = std::vec![&c0, &c1];

        let mut prop = Propagator::new();
        let changed = prop.propagate_incremental(&mut domains, &constraints, &[1]).unwrap();

        assert!(!domains[0].contains(1)); // A lost 1
        assert!(!domains[2].contains(1)); // C lost 1
        assert!(changed.contains(&0));
        assert!(changed.contains(&2));
    }
}
