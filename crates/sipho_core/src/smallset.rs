// Modified from smallset: a Rust crate for small unordered sets of elements, built on top of
// `smallvec`.
//
// Copyright (c) 2016 Chris Fallin <cfallin@c1f.net>. Released under the MIT license.
use std::fmt;
use std::iter::{FromIterator, IntoIterator};

use bevy::prelude::*;
use bevy::utils::smallvec::{Array, SmallVec};

/// A `SmallSet` is an unordered set of elements. It is designed to work best
/// for very small sets (no more than ten or so elements). In order to support
/// small sets very efficiently, it stores elements in a simple unordered array.
/// When the set is smaller than the size of the array `A`, all elements are
/// stored inline, without heap allocation. This is accomplished by using a
/// `smallvec::SmallVec`.
///
/// The insert, remove, and query methods on `SmallSet` have `O(n)` time
/// complexity in the current set size: they perform a linear scan to determine
/// if the element in question is present. This is inefficient for large sets,
/// but fast and cache-friendly for small sets.
///
/// Example usage:
///
/// ```
/// use smallset::SmallSet;
///
/// // `s` and its elements will be completely stack-allocated in this example.
/// let mut s: SmallSet<[u32; 4]> = SmallSet::new();
/// s.insert(1);
/// s.insert(2);
/// s.insert(3);
/// assert!(s.len() == 3);
/// assert!(s.contains(&1));
/// ```
#[derive(Deref, DerefMut)]
pub struct SmallSet<A: Array>
where
    A::Item: PartialEq + Eq,
{
    #[deref]
    elements: SmallVec<A>,
}

impl<A: Array> Default for SmallSet<A>
where
    A::Item: PartialEq + Eq,
{
    /// Creates a new, empty `SmallSet`.
    fn default() -> SmallSet<A> {
        Self::new()
    }
}

impl<A: Array> SmallSet<A>
where
    A::Item: PartialEq + Eq,
{
    /// Creates a new, empty `SmallSet`.
    pub fn new() -> SmallSet<A> {
        SmallSet {
            elements: SmallVec::new(),
        }
    }

    /// Inserts `elem` into the set if not yet present. Returns `true` if the
    /// set did not have this element present, or `false` if it already had this
    /// element present.
    pub fn insert(&mut self, elem: A::Item) -> bool {
        if !self.contains(&elem) {
            self.elements.push(elem);
            true
        } else {
            false
        }
    }

    /// Removes `elem` from the set. Returns `true` if the element was removed,
    /// or `false` if it was not found.
    pub fn remove(&mut self, elem: &A::Item) -> bool {
        if let Some(pos) = self.elements.iter().position(|e| *e == *elem) {
            self.elements.remove(pos);
            true
        } else {
            false
        }
    }

    /// Tests whether `elem` is present. Returns `true` if it is present, or
    /// `false` if not.
    pub fn contains(&self, elem: &A::Item) -> bool {
        self.elements.iter().any(|e| *e == *elem)
    }
}

impl<A: Array> Clone for SmallSet<A>
where
    A::Item: PartialEq + Eq + Clone,
{
    fn clone(&self) -> SmallSet<A> {
        SmallSet {
            elements: self.elements.clone(),
        }
    }
}

impl<A: Array> fmt::Debug for SmallSet<A>
where
    A::Item: PartialEq + Eq + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.elements.fmt(f)
    }
}

impl<A: Array> FromIterator<A::Item> for SmallSet<A>
where
    A::Item: PartialEq + Eq,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = A::Item>,
    {
        SmallSet {
            elements: SmallVec::from_iter(iter),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn test_basic_set() {
        let mut s: SmallSet<[u32; 2]> = SmallSet::new();
        assert!(s.insert(1));
        assert!(s.insert(2));
        assert!(s.insert(2));
        assert!(s.insert(3));
        assert!(s.insert(2));
        assert!(s.insert(3));
        assert!(s.contains(&1));
        assert!(s.contains(&2));
        assert!(s.contains(&3));
        assert!(!s.contains(&4));
        assert!(s.len() == 3);
        assert!(s.iter().copied().collect::<Vec<u32>>() == vec![1, 2, 3]);
        s.clear();
        assert!(!s.contains(&1));
    }

    #[test]
    fn test_remove() {
        let mut s: SmallSet<[u32; 2]> = SmallSet::new();
        assert!(s.insert(1));
        assert!(s.insert(2));
        assert!(s.len() == 2);
        assert!(s.contains(&1));
        assert!(s.remove(&1));
        assert!(s.remove(&1));
        assert!(s.len() == 1);
        assert!(!s.contains(&1));
        assert!(s.insert(1));
        assert!(s.iter().copied().collect::<Vec<u32>>() == vec![2, 1]);
    }

    #[test]
    fn test_clone() {
        let mut s: SmallSet<[u32; 2]> = SmallSet::new();
        s.insert(1);
        s.insert(2);
        let c = s.clone();
        assert!(c.contains(&1));
        assert!(c.contains(&2));
        assert!(!c.contains(&3));
    }

    #[test]
    fn test_debug() {
        let mut s: SmallSet<[u32; 2]> = SmallSet::new();
        s.insert(1);
        s.insert(2);
        let mut buf = String::new();
        write!(buf, "{:?}", s).unwrap();
        assert!(&buf == "[1, 2]");
    }

    #[test]
    fn test_fromiter() {
        let s: SmallSet<[usize; 4]> = vec![1, 2, 3, 4].into_iter().collect();
        assert!(s.len() == 4);
    }
}
