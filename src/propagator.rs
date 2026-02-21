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
        todo!()
    }

    /// Incremental propagation: only re-check constraints involving the given variables.
    pub fn propagate_incremental<const W: usize>(
        &mut self,
        domains: &mut [Domain<W>],
        constraints: &[&dyn Constraint<W>],
        changed_vars: &[usize],
    ) -> Result<Vec<usize>, Contradiction> {
        todo!()
    }
}
