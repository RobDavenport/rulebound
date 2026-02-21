//! Backtracking strategies for constraint solving.

/// Strategy for backtracking during search.
#[derive(Debug, Clone, Copy)]
pub enum BacktrackStrategy {
    /// No backtracking -- fail immediately on contradiction.
    None,
    /// Restart from scratch on contradiction (up to max_restarts).
    Restart { max_restarts: usize },
    /// Chronological backtracking (up to max_depth).
    Chronological { max_depth: usize },
}

impl Default for BacktrackStrategy {
    fn default() -> Self {
        Self::Chronological { max_depth: 1000 }
    }
}
