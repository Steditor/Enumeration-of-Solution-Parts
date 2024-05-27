use std::ops::RangeInclusive;

use super::Rng;

/// # Taillard linear congruential random number generator
///
/// This lcg is presented in \[2\] by E. Taillard who cites \[1\] as source.
///
/// \[1\] P. Bratley, B. L. Fox, und L. E. Schrage, A Guide to Simulation. New York, NY: Springer US, 1983. doi: [10.1007/978-1-4684-0167-7](https://doi.org/10.1007/978-1-4684-0167-7).<br>
/// \[2\] E. Taillard, „Benchmarks for basic scheduling problems“, European Journal of Operational Research, Bd. 64, Nr. 2, S. 278–285, Jan. 1993, doi: [10.1016/0377-2217(93)90182-M](https://doi.org/10.1016/0377-2217(93)90182-M).
pub struct TaillardLCG {
    seed: i32,
}

const A: i32 = 16_807;
const B: i32 = 127_773;
const C: i32 = 2_836;
const M: i32 = i32::MAX;

impl Rng for TaillardLCG {
    /// Returns a new rng initialized with the given seed.
    ///
    /// # Panics
    ///
    /// Panics if `seed <= 0` or `seed >= 2^32 - 1`.
    fn from_seed(seed: usize) -> Self {
        assert!(seed > 0);
        let seed = i32::try_from(seed).unwrap();
        Self { seed }
    }

    fn current_seed(&self) -> usize {
        usize::try_from(self.seed).unwrap()
    }

    fn state_id(&self) -> String {
        format!("TLCG-{}", self.current_seed())
    }

    /// Returns a random usize in the given range.
    ///
    /// # Panics
    ///
    /// Panics if `range` is empty or too large to fit into u32.
    fn next_usize(&mut self, range: RangeInclusive<usize>) -> usize {
        assert!(!range.is_empty());
        let size_of_range = range.end() - range.start() + 1;
        let size_of_range = u32::try_from(size_of_range).unwrap();

        let value_0_1 = self.next_double();
        range.start() + (value_0_1 * f64::from(size_of_range)) as usize
    }

    /// Returns a random i32 in the given range.
    ///
    /// # Panics
    ///
    /// Panics if `range` is empty.
    fn next_i32(&mut self, range: RangeInclusive<i32>) -> i32 {
        assert!(!range.is_empty());
        let value_0_1 = self.next_double();
        range.start() + (value_0_1 * f64::from(range.end() - range.start() + 1)) as i32
    }

    fn next_double(&mut self) -> f64 {
        let k = self.seed / B;
        self.seed = A * (self.seed % B) - k * C;
        if self.seed < 0 {
            self.seed += M;
        }
        f64::from(self.seed) / f64::from(M)
    }
}
