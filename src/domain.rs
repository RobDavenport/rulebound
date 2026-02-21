//! Variable domains backed by [`BitSet`].

use crate::bitset::BitSet;

/// A variable's domain: the set of values it can take.
pub type Domain<const W: usize = 2> = BitSet<W>;

/// Snapshot of all domains for backtracking.
#[derive(Clone)]
pub struct DomainSnapshot<const W: usize> {
    pub(crate) domains: alloc::vec::Vec<Domain<W>>,
}
