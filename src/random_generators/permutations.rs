use super::numbers::Rng;

pub struct Permutation {}

impl Permutation {
    /// Permute the given slice in-place with the Fisher-Yates shuffle.
    pub fn shuffle<T>(rng: &mut dyn Rng, slice: &mut [T]) {
        for i in 0..slice.len() {
            let j = rng.next_usize(i..=slice.len() - 1);
            slice.swap(i, j);
        }
    }

    /// Generate a random permutation of the elements in the given iterator.
    ///
    /// Use it e.g. to generate a random permutation of the number 0 to n-1:
    ///
    /// ```
    /// use exp_lib::{
    ///     random_generators::{
    ///         numbers::{Rng, TaillardLCG},
    ///         permutations::Permutation,
    ///     },
    /// };
    ///
    /// let mut rng = TaillardLCG::from_seed(42);
    /// let p = Permutation::permutation(&mut rng, 0..10);
    /// assert_eq!(p, [0, 5, 7, 4, 6, 1, 9, 8, 3, 2]);
    /// ```
    pub fn permutation<T, Iter: Iterator<Item = T>>(rng: &mut dyn Rng, range: Iter) -> Vec<T> {
        let mut permutation: Vec<T> = range.collect();
        Permutation::shuffle(rng, &mut permutation);

        permutation
    }
}
