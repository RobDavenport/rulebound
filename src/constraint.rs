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
        for i in 0..self.variables.len() {
            let vi = self.variables[i];
            if let Some(val) = domains[vi].singleton_value() {
                for j in 0..self.variables.len() {
                    if i != j {
                        let vj = self.variables[j];
                        domains[vj].remove(val);
                        if domains[vj].is_empty() {
                            return Err(Contradiction { variable: vj });
                        }
                    }
                }
            }
        }
        Ok(())
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
        let [a, b] = self.scope;

        // A must be < max(B): remove values from A that are >= max(B)
        if let Some(max_b) = domains[b].iter().last() {
            let to_remove: alloc::vec::Vec<usize> = domains[a].iter().filter(|&v| v >= max_b).collect();
            for v in to_remove { domains[a].remove(v); }
        } else {
            return Err(Contradiction { variable: b });
        }

        // B must be > min(A): remove values from B that are <= min(A)
        if let Some(min_a) = domains[a].min_value() {
            let to_remove: alloc::vec::Vec<usize> = domains[b].iter().filter(|&v| v <= min_a).collect();
            for v in to_remove { domains[b].remove(v); }
        } else {
            return Err(Contradiction { variable: a });
        }

        if domains[a].is_empty() { return Err(Contradiction { variable: a }); }
        if domains[b].is_empty() { return Err(Contradiction { variable: b }); }
        Ok(())
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

    #[test]
    fn all_different_removes_singleton_values() {
        // 3 vars: {1,2,3}, assign var0=1 → var1,var2 lose 1
        let mut domains = alloc::vec![Domain::<1>::empty(); 3];
        for d in domains.iter_mut() {
            d.insert(1); d.insert(2); d.insert(3);
        }
        domains[0] = Domain::empty();
        domains[0].insert(1); // singleton
        let c = AllDifferent::new(&[0, 1, 2]);
        c.propagate(&mut domains).unwrap();
        assert!(!domains[1].contains(1));
        assert!(!domains[2].contains(1));
        assert!(domains[1].contains(2));
        assert!(domains[1].contains(3));
    }

    #[test]
    fn all_different_cascade_two_singletons() {
        // var0={1}, var1={2}, var2={1,2,3} → var2={3}
        let mut domains = alloc::vec![Domain::<1>::empty(); 3];
        domains[0].insert(1);
        domains[1].insert(2);
        domains[2].insert(1); domains[2].insert(2); domains[2].insert(3);
        let c = AllDifferent::new(&[0, 1, 2]);
        c.propagate(&mut domains).unwrap();
        assert_eq!(domains[2].count(), 1);
        assert!(domains[2].contains(3));
    }

    #[test]
    fn less_than_prunes_both_domains() {
        // A in {1..5}, B in {1..5}, A < B → A loses 5, B loses 1
        let mut domains = [const { Domain::<1>::empty() }; 2];
        for v in 1..=5 { domains[0].insert(v); domains[1].insert(v); }
        let c = LessThan::new(0, 1);
        c.propagate(&mut domains).unwrap();
        assert!(!domains[0].contains(5), "A should lose max(B)=5");
        assert!(!domains[1].contains(1), "B should lose min(A)=1");
        assert_eq!(domains[0].count(), 4); // {1,2,3,4}
        assert_eq!(domains[1].count(), 4); // {2,3,4,5}
    }

    #[test]
    fn less_than_tight_domains() {
        // A={3}, B={3} → contradiction (3 < 3 is false)
        let mut domains = [const { Domain::<1>::empty() }; 2];
        domains[0].insert(3);
        domains[1].insert(3);
        let c = LessThan::new(0, 1);
        assert!(c.propagate(&mut domains).is_err());
    }

    #[test]
    fn exactly_n_forces_remaining() {
        // 3 vars, exactly 2 must be value 1. var0={1}, var1={0}, var2={0,1}
        // var0 already has 1, var1 can't have 1, so var2 must have 1
        let mut domains = alloc::vec![Domain::<1>::empty(); 3];
        domains[0].insert(1);
        domains[1].insert(0);
        domains[2].insert(0); domains[2].insert(1);
        let c = ExactlyN::new(&[0, 1, 2], 1, 2);
        c.propagate(&mut domains).unwrap();
        assert!(domains[2].contains(1));
    }

    #[test]
    fn exactly_n_removes_when_count_met() {
        // 3 vars, exactly 1 must be value 5. var0={5}. → var1,var2 lose 5.
        let mut domains = alloc::vec![Domain::<1>::empty(); 3];
        domains[0].insert(5);
        domains[1].insert(5); domains[1].insert(6);
        domains[2].insert(5); domains[2].insert(7);
        let c = ExactlyN::new(&[0, 1, 2], 5, 1);
        c.propagate(&mut domains).unwrap();
        assert!(!domains[1].contains(5));
        assert!(!domains[2].contains(5));
    }

    #[test]
    fn table_prunes_inconsistent_values() {
        // 2 vars, allowed: (0,1), (1,0). var0={0,1,2}, var1={0,1,2}
        // After propagation: var0={0,1}, var1={0,1} (2 removed from both)
        let mut domains = alloc::vec![Domain::<1>::full(3); 2];
        let c = Table::new(
            &[0, 1],
            std::vec![std::vec![0, 1], std::vec![1, 0]],
        );
        c.propagate(&mut domains).unwrap();
        assert!(!domains[0].contains(2));
        assert!(!domains[1].contains(2));
        assert!(domains[0].contains(0));
        assert!(domains[0].contains(1));
    }

    #[test]
    fn implication_fires_when_triggered() {
        // If A=2, then B in {0, 3}. A={2}, B={0,1,2,3} → B={0,3}
        let mut domains = [const { Domain::<1>::empty() }; 2];
        domains[0].insert(2);
        let mut allowed_b = Domain::<1>::empty();
        allowed_b.insert(0); allowed_b.insert(3);
        domains[1] = Domain::full(4); // {0,1,2,3}
        let c = Implication::new(0, 2, 1, allowed_b);
        c.propagate(&mut domains).unwrap();
        assert_eq!(domains[1].count(), 2);
        assert!(domains[1].contains(0));
        assert!(domains[1].contains(3));
    }

    #[test]
    fn implication_no_op_when_not_triggered() {
        // A={1,2}, val_a=2. A is not singleton, so no action.
        let mut domains = [const { Domain::<1>::empty() }; 2];
        domains[0].insert(1); domains[0].insert(2);
        domains[1] = Domain::full(4);
        let allowed_b = Domain::<1>::empty();
        let c = Implication::<1>::new(0, 2, 1, allowed_b);
        c.propagate(&mut domains).unwrap();
        assert_eq!(domains[1].count(), 4); // unchanged
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
        let val = self.value;
        let mut must_count = 0usize;  // singletons that ARE val
        let mut can_count = 0usize;   // non-singletons that contain val

        for &v in &self.variables {
            if domains[v].is_singleton() && domains[v].contains(val) {
                must_count += 1;
            } else if domains[v].contains(val) {
                can_count += 1;
            }
        }

        if must_count > self.count {
            return Err(Contradiction { variable: self.variables[0] });
        }

        if must_count + can_count < self.count {
            return Err(Contradiction { variable: self.variables[0] });
        }

        // If count already met, remove val from all non-singleton vars
        if must_count == self.count {
            for &v in &self.variables {
                if !domains[v].is_singleton() {
                    domains[v].remove(val);
                    if domains[v].is_empty() {
                        return Err(Contradiction { variable: v });
                    }
                }
            }
        }

        // If remaining slots == remaining candidates, force them all
        let remaining_needed = self.count - must_count;
        if remaining_needed == can_count {
            for &v in &self.variables {
                if !domains[v].is_singleton() && domains[v].contains(val) {
                    domains[v] = Domain::empty();
                    domains[v].insert(val);
                }
            }
        }

        Ok(())
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
        let n = self.variables.len();
        let mut allowed_vals: Vec<Domain<W>> = (0..n).map(|_| Domain::empty()).collect();

        for tuple in &self.allowed {
            // Check if this tuple is consistent with current domains
            let consistent = tuple.iter().enumerate().all(|(i, &val)| {
                domains[self.variables[i]].contains(val)
            });
            if consistent {
                for (i, &val) in tuple.iter().enumerate() {
                    allowed_vals[i].insert(val);
                }
            }
        }

        // Intersect each variable's domain with its allowed values
        for (i, &v) in self.variables.iter().enumerate() {
            domains[v].intersect_with(&allowed_vals[i]);
            if domains[v].is_empty() {
                return Err(Contradiction { variable: v });
            }
        }

        Ok(())
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
        let [a, b] = self.scope;
        // Only fire if A is singleton and equals val_a
        if domains[a].is_singleton() && domains[a].contains(self.val_a) {
            domains[b].intersect_with(&self.allowed_b);
            if domains[b].is_empty() {
                return Err(Contradiction { variable: b });
            }
        }
        Ok(())
    }
}
