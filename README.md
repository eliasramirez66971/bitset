# bitset

A compact set of non-negative integers backed by a dynamically sized bit vector.

```rust
use bitset::BitSet;

let mut set = BitSet::new();
set.insert(3);
set.insert(64);
assert!(set.contains(3));
assert_eq!(set.len(), 2);

let values: Vec<usize> = set.iter().collect();
assert_eq!(values, vec![3, 64]);

set.remove(3);
assert!(!set.contains(3));
```

## Why this exists

A `HashSet<usize>` stores each element with hashing overhead, and a sorted `Vec<usize>`
spends time shifting on insertion. When the integers being tracked are clustered or dense,
those costs are wasted: the set of values is itself a bit pattern. This crate stores that
pattern directly in `Vec<u64>`, so membership tests and insertions are a single word
access and bit mask. The trade-off is memory proportional to the largest element rather
than the number of elements; a set containing only `1_000_000` still allocates about
125 KiB. That is the right choice for dense or bounded domains and the wrong one for
sparse, unbounded ones.

## Edge cases

`contains` on a value beyond the current backing length returns `false` without
allocating, while `insert` grows the backing vector to fit. `remove` of an absent value
returns `false` and does not allocate. `clear` drops the allocated storage entirely, so
the next insertion may reallocate; use `remove` in a loop if you want to preserve
capacity across resets.

The iterator visits elements in ascending numeric order regardless of insertion order.
