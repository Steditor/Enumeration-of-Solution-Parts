use std::{
    fmt::{Debug, Display},
    ops::{Range, RangeInclusive},
};

/// Index to identify nodes and edges in a graph.
///
/// Heavily inspired by <https://docs.rs/graph_builder/latest/src/graph_builder/index.rs.html>
pub trait Index:
    Copy
    + std::ops::Add<Output = Self>
    + std::ops::AddAssign
    + std::ops::Sub<Output = Self>
    + std::ops::SubAssign
    + Ord
    + Debug
    + Display
    + Sized
    + 'static
{
    fn new(i: usize) -> Self;
    fn index(self) -> usize;

    type IndexIterator: Iterator<Item = Self>;
    fn range(self, end: Self) -> Self::IndexIterator;

    type IndexInclusiveIterator: Iterator<Item = Self>;
    fn range_inclusive(self, end: Self) -> Self::IndexInclusiveIterator;
}

impl Index for u32 {
    #[inline]
    fn new(i: usize) -> Self {
        assert!(i <= u32::MAX as usize);
        i as u32
    }

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    type IndexIterator = Range<Self>;
    #[inline]
    fn range(self, end: Self) -> Self::IndexIterator {
        self..end
    }

    type IndexInclusiveIterator = RangeInclusive<Self>;
    #[inline]
    fn range_inclusive(self, end: Self) -> Self::IndexInclusiveIterator {
        self..=end
    }
}
