use rand::{seq::SliceRandom, Rng};

pub struct Permutation {}

impl Permutation {
    /// Generate a random permutation of the elements in the given iterator.
    ///
    /// Use it e.g. to generate a random permutation of the number 0 to n-1:
    ///
    /// ```
    /// use rand::SeedableRng;
    /// use rand_pcg::Pcg64;
    ///
    /// use exp_lib::{
    ///     data_generators::{
    ///         permutations::Permutation,
    ///     },
    /// };
    ///
    /// let mut rng = Pcg64::seed_from_u64(42);
    /// let p = Permutation::permutation(&mut rng, 0..10);
    /// assert_eq!(p, [2, 4, 0, 5, 9, 3, 8, 7, 1, 6]);
    /// ```
    pub fn permutation<T, Iter: Iterator<Item = T>>(rng: &mut impl Rng, range: Iter) -> Vec<T> {
        let mut permutation: Vec<T> = range.collect();
        permutation.shuffle(rng);

        permutation
    }
}
