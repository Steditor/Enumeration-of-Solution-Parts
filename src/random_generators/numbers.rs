//! # Random number generators

#[doc(hidden)]
mod taillard_lcg;

use std::ops::RangeInclusive;

pub use taillard_lcg::TaillardLCG;

pub trait Rng {
    /// Returns a new rng initialized with the given seed.
    fn from_seed(seed: usize) -> Self
    where
        Self: Sized;
    /// Returns the current seed that can be used to replicate the following sequence of random numbers.
    fn current_seed(&self) -> usize;
    /// Advances the Rng and returns the new seed.
    fn next_seed(&mut self) -> usize {
        self.next_double();
        self.current_seed()
    }
    /// Returns a string that identifies both the rng used and its current seed for replication.
    ///
    /// The identifier must be suitable for use in file systems and should have the form `"{rng_id}-{seed}"`.
    fn state_id(&self) -> String;
    /// Returns a random usize in the given range.
    fn next_usize(&mut self, range: RangeInclusive<usize>) -> usize;
    /// Returns a random i32 in the given range.
    fn next_i32(&mut self, range: RangeInclusive<i32>) -> i32;
    /// Returns a random f64 between 0 and 1 (both exclusive).
    fn next_double(&mut self) -> f64;
}
