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
        todo!()
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
