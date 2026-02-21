//! Error types for constraint solving.

use core::fmt;

/// A contradiction was detected: some variable has an empty domain.
#[derive(Debug, Clone)]
pub struct Contradiction {
    /// The variable index that became empty.
    pub variable: usize,
}

impl fmt::Display for Contradiction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "contradiction: variable {} has empty domain", self.variable)
    }
}

/// Error returned by [`Solver::solve`](crate::Solver::solve).
#[derive(Debug, Clone)]
pub enum SolveError {
    /// A contradiction was detected during propagation.
    Contradiction(Contradiction),
    /// Search exhausted all possibilities without finding a solution.
    Unsatisfiable,
    /// Maximum number of backtracks exceeded.
    BacktrackLimitExceeded,
}

impl fmt::Display for SolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Contradiction(c) => write!(f, "{}", c),
            Self::Unsatisfiable => write!(f, "no solution exists"),
            Self::BacktrackLimitExceeded => write!(f, "backtrack limit exceeded"),
        }
    }
}

impl From<Contradiction> for SolveError {
    fn from(c: Contradiction) -> Self {
        Self::Contradiction(c)
    }
}
