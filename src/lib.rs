//! A compact set of non-negative integers backed by a dynamically sized bit vector.
//!
//! The set stores membership as bits in a `Vec<u64>`. Each word holds 64 bits, so
//! bit `n` lives in word `n / 64` at position `n % 64`. This keeps memory proportional
//! to the largest element rather than the number of elements, which is the right
//! trade-off for dense or clustered sets.

/// A set of non-negative integers stored as a bit vector.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    /// Creates an empty set.
    pub fn new() -> Self {
        BitSet { words: Vec::new() }
    }

    /// Returns the number of bits the backing vector can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.words.capacity() * 64
    }

    /// Returns `true` if the set contains no elements.
    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    /// Inserts `value` into the set.
    ///
    /// Returns `true` if the value was not already present.
    pub fn insert(&mut self, value: usize) -> bool {
        let (word, bit) = Self::split(value);
        self.ensure(word);
        let mask = 1u64 << bit;
        let was_present = (self.words[word] & mask) != 0;
        self.words[word] |= mask;
        !was_present
    }

    /// Removes `value` from the set.
    ///
    /// Returns `true` if the value was present.
    pub fn remove(&mut self, value: usize) -> bool {
        let (word, bit) = Self::split(value);
        if word >= self.words.len() {
            return false;
        }
        let mask = 1u64 << bit;
        let was_present = (self.words[word] & mask) != 0;
        self.words[word] &= !mask;
        was_present
    }

    /// Returns `true` if `value` is in the set.
    pub fn contains(&self, value: usize) -> bool {
        let (word, bit) = Self::split(value);
        word < self.words.len() && (self.words[word] & (1u64 << bit)) != 0
    }

    /// Removes every element, keeping the allocated capacity.
    pub fn clear(&mut self) {
        self.words.clear();
    }

    /// Extends the set with every value produced by `iter`.
    pub fn extend<I: IntoIterator<Item = usize>>(&mut self, iter: I) {
        for value in iter {
            self.insert(value);
        }
    }

    /// Returns an iterator over the elements in ascending order.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            words: &self.words,
            word_index: 0,
            current: self.words.first().copied().unwrap_or(0),
        }
    }

    fn split(value: usize) -> (usize, u32) {
        (value / 64, (value % 64) as u32)
    }

    fn ensure(&mut self, word: usize) {
        if word >= self.words.len() {
            self.words.resize(word + 1, 0);
        }
    }
}

impl FromIterator<usize> for BitSet {
    fn from_iter<I: IntoIterator<Item = usize>>(iter: I) -> Self {
        let mut set = BitSet::new();
        set.extend(iter);
        set
    }
}

impl<'a> IntoIterator for &'a BitSet {
    type Item = usize;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the elements of a [`BitSet`] in ascending order.
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    words: &'a [u64],
    word_index: usize,
    current: u64,
}

impl<'a> Iterator for Iter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        loop {
            if self.current != 0 {
                let bit = self.current.trailing_zeros() as usize;
                self.current &= self.current - 1;
                return Some(self.word_index * 64 + bit);
            }
            self.word_index += 1;
            if self.word_index >= self.words.len() {
                return None;
            }
            self.current = self.words[self.word_index];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_is_empty() {
        let set = BitSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
        assert!(!set.contains(0));
    }

    #[test]
    fn insert_reports_novelty() {
        let mut set = BitSet::new();
        assert!(set.insert(3));
        assert!(!set.insert(3));
        assert_eq!(set.len(), 1);
        assert!(set.contains(3));
    }

    #[test]
    fn remove_reports_presence() {
        let mut set = BitSet::new();
        set.insert(7);
        assert!(set.remove(7));
        assert!(!set.remove(7));
        assert!(!set.contains(7));
        assert!(set.is_empty());
    }

    #[test]
    fn remove_absent_value_is_false() {
        let mut set = BitSet::new();
        assert!(!set.remove(100));
        assert!(set.is_empty());
    }

    #[test]
    fn contains_handles_out_of_range() {
        let mut set = BitSet::new();
        set.insert(5);
        assert!(!set.contains(1000));
    }

    #[test]
    fn len_counts_bits_across_words() {
        let mut set = BitSet::new();
        set.insert(0);
        set.insert(63);
        set.insert(64);
        set.insert(200);
        assert_eq!(set.len(), 4);
    }

    #[test]
    fn clear_resets_elements() {
        let mut set = BitSet::new();
        set.insert(1);
        set.insert(2);
        set.clear();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
        assert!(!set.contains(1));
    }

    #[test]
    fn extend_adds_all_values() {
        let mut set = BitSet::new();
        set.extend([10, 20, 10, 30]);
        assert_eq!(set.len(), 3);
        assert!(set.contains(10));
        assert!(set.contains(20));
        assert!(set.contains(30));
    }

    #[test]
    fn from_iterator_collects() {
        let set: BitSet = (0..5).collect();
        assert_eq!(set.len(), 5);
        for i in 0..5 {
            assert!(set.contains(i));
        }
    }

    #[test]
    fn iter_yields_ascending_order() {
        let mut set = BitSet::new();
        set.insert(65);
        set.insert(1);
        set.insert(64);
        set.insert(0);
        let collected: Vec<usize> = set.iter().collect();
        assert_eq!(collected, vec![0, 1, 64, 65]);
    }

    #[test]
    fn iter_on_empty_set_is_empty() {
        let set = BitSet::new();
        assert_eq!(set.iter().count(), 0);
    }

    #[test]
    fn into_iterator_for_reference() {
        let mut set = BitSet::new();
        set.insert(2);
        set.insert(4);
        let collected: Vec<usize> = (&set).into_iter().collect();
        assert_eq!(collected, vec![2, 4]);
    }

    #[test]
    fn capacity_grows_but_is_not_required_to_shrink() {
        let mut set = BitSet::new();
        set.insert(1000);
        assert!(set.capacity() >= 1001);
        set.clear();
        // Capacity may remain, but the set must be empty.
        assert!(set.is_empty());
    }
}
