use binary_heap_plus::{BinaryHeap, IntoIterSorted};
use compare::{Compare, Natural};

use crate::{
    algorithms::sorting::AlgorithmType,
    data_generators::sorting::SortingInstance,
    experiments::{ExperimentAlgorithm, PreparedEnumerationAlgorithm},
};

pub const ENUMERATE_WITH_IHS: AlgorithmType =
    ExperimentAlgorithm::EnumerationAlgorithm("enum-ihs", prepare_enumeration_algorithm);

fn prepare_enumeration_algorithm<T>(input: &SortingInstance<T>) -> PreparedEnumerationAlgorithm<T>
where
    T: Copy + Ord,
{
    Box::new(IHS::new(input))
}

pub struct IHS<T, C> {
    heap: IntoIterSorted<T, C>,
}

impl<T> IHS<T, Natural<T>>
where
    T: Copy + Ord,
{
    pub fn new(elements: &[T]) -> Self {
        Self {
            heap: BinaryHeap::from_vec(elements.to_vec()).into_iter_sorted(),
        }
    }
}

impl<T, C> IHS<T, C>
where
    T: Copy,
    C: Compare<T>,
{
    pub fn with_comparator(elements: &[T], comparator: C) -> Self {
        Self {
            heap: BinaryHeap::from_vec_cmp(elements.to_vec(), comparator).into_iter_sorted(),
        }
    }
}

impl<T, C> Iterator for IHS<T, C>
where
    T: Copy,
    C: Compare<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.heap.next()
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use rand::{distributions::Uniform, prelude::Distribution, SeedableRng};
    use rand_pcg::Pcg64;

    use super::*;

    #[test]
    fn test_delay() {
        // simple_logger::init_with_level(log::Level::Info).unwrap();

        let elements: Vec<_> = Uniform::new(0, u32::MAX)
            .sample_iter(Pcg64::seed_from_u64(42))
            .take(1000)
            .collect();

        let start = Instant::now();
        let ihs = IHS::new(&elements);

        let preprocessing_time = start.elapsed().as_nanos() as u64;
        log::info!("{preprocessing_time}");

        let mut delay_start = Instant::now();
        for _ in ihs.take(50) {
            let delay = delay_start.elapsed().as_nanos() as u64;
            log::info!("{delay}");
            assert!(delay < preprocessing_time);
            delay_start = Instant::now();
        }
    }
}
