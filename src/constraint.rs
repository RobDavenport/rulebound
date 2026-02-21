//! Constraint trait and built-in constraint implementations.

use alloc::vec::Vec;
use crate::domain::Domain;
use crate::error::Contradiction;

/// A constraint between variables. Implementations prune impossible values
/// from variable domains during propagation.
pub trait Constraint<const W: usize = 2> {
    /// The variable indices this constraint involves.
    fn scope(&self) -> &[usize];

    /// Prune impossible values from the given domains.
    /// Returns `Err(Contradiction)` if any domain becomes empty.
    fn propagate(&self, domains: &mut [Domain<W>]) -> Result<(), Contradiction>;
}

/// Two variables must take different values.
pub struct NotEqual {
    scope: [usize; 2],
}

impl NotEqual {
    pub fn new(a: usize, b: usize) -> Self {
        Self { scope: [a, b] }
    }
}

impl<const W: usize> Constraint<W> for NotEqual {
    fn scope(&self) -> &[usize] {
        &self.scope
    }

    fn propagate(&self, domains: &mut [Domain<W>]) -> Result<(), Contradiction> {
        let [a, b] = self.scope;
        if let Some(v) = domains[a].singleton_value() {
            domains[b].remove(v);
            if domains[b].is_empty() {
                return Err(Contradiction { variable: b });
            }
        }
        if let Some(v) = domains[b].singleton_value() {
            domains[a].remove(v);
            if domains[a].is_empty() {
                return Err(Contradiction { variable: a });
            }
        }
        Ok(())
    }
}

/// All variables in scope must take different values.
pub struct AllDifferent {
    pub variables: Vec<usize>,
}

impl AllDifferent {
    pub fn new(variables: &[usize]) -> Self {
        Self { variables: variables.into() }
    }
}

impl<const W: usize> Constraint<W> for AllDifferent {
    fn scope(&self) -> &[usize] {
        &self.variables
    }

    fn propagate(&self, domains: &mut [Domain<W>]) -> Result<(), Contradiction> {
        todo!()
    }
}

/// Variable A's value must be less than variable B's value.
pub struct LessThan {
    scope: [usize; 2],
}

impl LessThan {
    pub fn new(a: usize, b: usize) -> Self {
        Self { scope: [a, b] }
    }
}

impl<const W: usize> Constraint<W> for LessThan {
    fn scope(&self) -> &[usize] {
        &self.scope
    }

    fn propagate(&self, domains: &mut [Domain<W>]) -> Result<(), Contradiction> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    #[test]
    fn not_equal_propagates_singleton() {
        // A={3}, B={1,2,3} → B should lose 3
        let mut domains = [const { Domain::<1>::empty() }; 2];
        domains[0].insert(3);
        domains[1] = Domain::full(4); // {0,1,2,3}
        let c = NotEqual::new(0, 1);
        c.propagate(&mut domains).unwrap();
        assert!(!domains[1].contains(3));
        assert!(domains[1].contains(0));
        assert!(domains[1].contains(1));
        assert!(domains[1].contains(2));
    }

    #[test]
    fn not_equal_contradiction() {
        // A={5}, B={5} → contradiction
        let mut domains = [const { Domain::<1>::empty() }; 2];
        domains[0].insert(5);
        domains[1].insert(5);
        let c = NotEqual::new(0, 1);
        assert!(c.propagate(&mut domains).is_err());
    }

    #[test]
    fn not_equal_no_op_when_no_singleton() {
        let mut domains: [Domain<1>; 2] = core::array::from_fn(|_| Domain::full(5));
        let c = NotEqual::new(0, 1);
        c.propagate(&mut domains).unwrap();
        assert_eq!(domains[0].count(), 5);
        assert_eq!(domains[1].count(), 5);
    }
}

/// Exactly N variables in scope must take a specific value.
pub struct ExactlyN {
    pub variables: Vec<usize>,
    pub value: usize,
    pub count: usize,
}

impl ExactlyN {
    pub fn new(variables: &[usize], value: usize, count: usize) -> Self {
        Self { variables: variables.into(), value, count }
    }
}

impl<const W: usize> Constraint<W> for ExactlyN {
    fn scope(&self) -> &[usize] {
        &self.variables
    }

    fn propagate(&self, domains: &mut [Domain<W>]) -> Result<(), Contradiction> {
        todo!()
    }
}

/// Explicit table of allowed value tuples.
pub struct Table {
    pub variables: Vec<usize>,
    pub allowed: Vec<Vec<usize>>,
}

impl Table {
    pub fn new(variables: &[usize], allowed: Vec<Vec<usize>>) -> Self {
        Self { variables: variables.into(), allowed }
    }
}

impl<const W: usize> Constraint<W> for Table {
    fn scope(&self) -> &[usize] {
        &self.variables
    }

    fn propagate(&self, domains: &mut [Domain<W>]) -> Result<(), Contradiction> {
        todo!()
    }
}

/// If variable A = val_a, then variable B must be in allowed_b set.
pub struct Implication<const W: usize = 2> {
    scope: [usize; 2],
    pub val_a: usize,
    pub allowed_b: Domain<W>,
}

impl<const W: usize> Implication<W> {
    pub fn new(a: usize, val_a: usize, b: usize, allowed_b: Domain<W>) -> Self {
        Self { scope: [a, b], val_a, allowed_b }
    }
}

impl<const W: usize> Constraint<W> for Implication<W> {
    fn scope(&self) -> &[usize] {
        &self.scope
    }

    fn propagate(&self, domains: &mut [Domain<W>]) -> Result<(), Contradiction> {
        todo!()
    }
}
