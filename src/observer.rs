//! Observer trait for monitoring solving progress.

/// Observer for monitoring constraint solver progress.
pub trait Observer<const W: usize = 2> {
    /// Called when a variable is assigned a value.
    fn on_assign(&mut self, _variable: usize, _value: usize) {}
    /// Called when a value is pruned from a variable's domain.
    fn on_prune(&mut self, _variable: usize, _value: usize) {}
    /// Called when propagation completes a round.
    fn on_propagation_round(&mut self, _round: usize) {}
    /// Called when backtracking occurs.
    fn on_backtrack(&mut self, _depth: usize) {}
    /// Called when solving is complete.
    fn on_complete(&mut self) {}
}

/// No-op observer that does nothing.
pub struct NoOpObserver;

impl<const W: usize> Observer<W> for NoOpObserver {}
