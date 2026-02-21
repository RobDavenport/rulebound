//! Efficient bit set for representing variable domains.

use core::fmt;

/// A fixed-size bit set backed by an array of `u64` words.
/// `W` is the number of 64-bit words, supporting up to `W * 64` values.
#[derive(Clone, PartialEq, Eq)]
pub struct BitSet<const W: usize> {
    words: [u64; W],
}

impl<const W: usize> BitSet<W> {
    /// Maximum number of values this bitset can represent.
    pub const CAPACITY: usize = W * 64;

    /// Create an empty bitset.
    pub const fn empty() -> Self {
        Self { words: [0; W] }
    }

    /// Create a bitset with all bits set up to `n` (exclusive).
    pub fn full(n: usize) -> Self {
        let mut bs = Self::empty();
        for i in 0..n.min(Self::CAPACITY) {
            bs.insert(i);
        }
        bs
    }

    /// Insert a value into the set.
    pub fn insert(&mut self, value: usize) {
        if value < Self::CAPACITY {
            self.words[value / 64] |= 1u64 << (value % 64);
        }
    }

    /// Remove a value from the set. Returns true if the value was present.
    pub fn remove(&mut self, value: usize) -> bool {
        if value < Self::CAPACITY {
            let word = &mut self.words[value / 64];
            let mask = 1u64 << (value % 64);
            let was_set = *word & mask != 0;
            *word &= !mask;
            was_set
        } else {
            false
        }
    }

    /// Check if a value is in the set.
    pub fn contains(&self, value: usize) -> bool {
        if value < Self::CAPACITY {
            self.words[value / 64] & (1u64 << (value % 64)) != 0
        } else {
            false
        }
    }

    /// Number of values in the set.
    pub fn count(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    /// Check if the set contains exactly one value.
    pub fn is_singleton(&self) -> bool {
        self.count() == 1
    }

    /// Get the single value if this is a singleton set.
    pub fn singleton_value(&self) -> Option<usize> {
        if self.is_singleton() {
            self.iter().next()
        } else {
            None
        }
    }

    /// Get the smallest value in the set.
    pub fn min_value(&self) -> Option<usize> {
        self.iter().next()
    }

    /// Iterate over all values in the set.
    pub fn iter(&self) -> BitSetIter<'_, W> {
        BitSetIter { bitset: self, word_idx: 0, bit_idx: 0 }
    }

    /// Intersect with another bitset in place.
    pub fn intersect_with(&mut self, other: &Self) {
        for (a, b) in self.words.iter_mut().zip(other.words.iter()) {
            *a &= *b;
        }
    }

    /// Union with another bitset in place.
    pub fn union_with(&mut self, other: &Self) {
        for (a, b) in self.words.iter_mut().zip(other.words.iter()) {
            *a |= *b;
        }
    }

    /// Subtract another bitset from this one.
    pub fn subtract(&mut self, other: &Self) {
        for (a, b) in self.words.iter_mut().zip(other.words.iter()) {
            *a &= !*b;
        }
    }
}

impl<const W: usize> fmt::Debug for BitSet<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BitSet{{")?;
        let mut first = true;
        for v in self.iter() {
            if !first { write!(f, ", ")?; }
            write!(f, "{}", v)?;
            first = false;
        }
        write!(f, "}}")
    }
}

/// Iterator over values in a [`BitSet`].
pub struct BitSetIter<'a, const W: usize> {
    bitset: &'a BitSet<W>,
    word_idx: usize,
    bit_idx: usize,
}

impl<'a, const W: usize> Iterator for BitSetIter<'a, W> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        while self.word_idx < W {
            let word = self.bitset.words[self.word_idx];
            // Skip remaining bits in this word starting from bit_idx
            let masked = word >> self.bit_idx;
            if masked != 0 {
                let tz = masked.trailing_zeros() as usize;
                self.bit_idx += tz;
                let value = self.word_idx * 64 + self.bit_idx;
                self.bit_idx += 1;
                if self.bit_idx >= 64 {
                    self.word_idx += 1;
                    self.bit_idx = 0;
                }
                return Some(value);
            }
            self.word_idx += 1;
            self.bit_idx = 0;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    #[test]
    fn empty_bitset() {
        let bs = BitSet::<2>::empty();
        assert!(bs.is_empty());
        assert_eq!(bs.count(), 0);
    }

    #[test]
    fn insert_and_contains() {
        let mut bs = BitSet::<2>::empty();
        bs.insert(0);
        bs.insert(42);
        bs.insert(127);
        assert!(bs.contains(0));
        assert!(bs.contains(42));
        assert!(bs.contains(127));
        assert!(!bs.contains(1));
        assert_eq!(bs.count(), 3);
    }

    #[test]
    fn full_bitset() {
        let bs = BitSet::<1>::full(10);
        assert_eq!(bs.count(), 10);
        for i in 0..10 {
            assert!(bs.contains(i));
        }
        assert!(!bs.contains(10));
    }

    #[test]
    fn singleton() {
        let mut bs = BitSet::<1>::empty();
        bs.insert(5);
        assert!(bs.is_singleton());
        assert_eq!(bs.singleton_value(), Some(5));
    }

    #[test]
    fn iter() {
        let mut bs = BitSet::<1>::empty();
        bs.insert(3);
        bs.insert(7);
        bs.insert(11);
        let vals: std::vec::Vec<_> = bs.iter().collect();
        assert_eq!(vals, std::vec![3, 7, 11]);
    }

    #[test]
    fn remove() {
        let mut bs = BitSet::<1>::full(5);
        assert!(bs.remove(2));
        assert!(!bs.contains(2));
        assert!(!bs.remove(2));
        assert_eq!(bs.count(), 4);
    }

    #[test]
    fn set_operations() {
        let mut a = BitSet::<1>::full(10);
        let mut b = BitSet::<1>::empty();
        for i in (0..10).step_by(2) {
            b.insert(i);
        }
        a.intersect_with(&b);
        assert_eq!(a.count(), 5); // 0, 2, 4, 6, 8
    }
}
