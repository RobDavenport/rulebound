//! A lightweight, no_std constraint propagation solver for real-time game logic.
//!
//! `rulebound` provides arc-consistency (AC-3) constraint propagation with optional
//! backtracking search. Designed for game-time use: small problems (10-100 variables),
//! incremental solving, frame-budget-aware.
//!
//! # Features
//!
//! - **Game-time constraint solving**: Incremental propagation for per-frame use
//! - **Built-in constraints**: AllDifferent, NotEqual, LessThan, ExactlyN, Table, Implication
//! - **Configurable search**: AC-3 propagation + optional backtracking with multiple strategies
//! - **Observable**: Monitor solving progress via the [`Observer`] trait
//! - **`no_std` compatible**: Works in embedded and WASM environments
//!
//! # Quick Start
//!
//! 1. Create a [`Solver`] with [`SolverConfig`]
//! 2. Add [`Constraint`]s (built-in or custom)
//! 3. Optionally [`assign`](Solver::assign) known values
//! 4. Call [`propagate`](Solver::propagate) or [`solve`](Solver::solve)

#![no_std]

extern crate alloc;

pub mod bitset;
pub mod domain;
pub mod constraint;
pub mod propagator;
pub mod solver;
pub mod config;
pub mod observer;
pub mod error;
pub mod backtrack;

// Re-export primary API
pub use bitset::BitSet;
pub use domain::Domain;
pub use constraint::{Constraint, AllDifferent, NotEqual, LessThan, ExactlyN, Table, Implication};
pub use propagator::Propagator;
pub use solver::{Solver, Solution, SolveResult};
pub use config::SolverConfig;
pub use observer::{Observer, NoOpObserver};
pub use error::{Contradiction, SolveError};
pub use backtrack::BacktrackStrategy;
