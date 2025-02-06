use std::{
    fmt::{Debug, Display},
    ops::Range,
};

use num::{One, Unsigned, Zero};

/// An index to a data structure that is smaller or equal to usize.
///
/// This is used to be able to store data types with indices that are pointer-like references more compactly
/// than always having to use usize.
/// The implementation is heavily inspired by <https://docs.rs/graph_builder/latest/src/graph_builder/index.rs.html>
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
    + Default
    + Unsigned
    + Zero
    + One
    + 'static
{
    fn new(i: usize) -> Self;
    fn index(self) -> usize;

    type IndexIterator: Iterator<Item = Self>;
    fn range(self, end: Self) -> Self::IndexIterator;
}

impl Index for u32 {
    #[inline]
    fn new(i: usize) -> Self {
        debug_assert!(i <= u32::MAX as usize);
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
}
