use std::ops::RangeInclusive;

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

impl TaillardLCG {
    /// Returns a new rng initialized with the given seed.
    pub fn from_seed(seed: u64) -> Self {
        let mut seed = i32::try_from(seed % ((i32::MAX as u64) + 1)).unwrap();
        if seed == 0 {
            seed = 0xBAD;
        }
        Self { seed }
    }

    /// Returns a random i32 in the given range.
    ///
    /// # Panics
    ///
    /// Panics if `range` is empty.
    pub fn next_i32(&mut self, range: RangeInclusive<i32>) -> i32 {
        assert!(!range.is_empty());
        let value_0_1 = self.next_double();
        range.start() + (value_0_1 * f64::from(range.end() - range.start() + 1)) as i32
    }

    /// Returns a random double in the open interval (0, 1).
    fn next_double(&mut self) -> f64 {
        let k = self.seed / B;
        self.seed = A * (self.seed % B) - k * C;
        if self.seed < 0 {
            self.seed += M;
        }
        f64::from(self.seed) / f64::from(M)
    }
}
