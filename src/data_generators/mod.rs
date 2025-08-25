use rand::{prelude::Distribution, Rng};

pub mod graphs;
pub mod permutations;
pub mod scheduling;
pub mod sorting;

/// An object-safe Distribution trait using a fixed Rng.
pub trait FixedRngDistribution<T, R: Rng> {
    fn sample(&self, rng: &mut R) -> T;
}

impl<T, R: Rng, U> FixedRngDistribution<T, R> for U
where
    U: Distribution<T>,
{
    fn sample(&self, rng: &mut R) -> T {
        <Self as Distribution<T>>::sample(self, rng)
    }
}
