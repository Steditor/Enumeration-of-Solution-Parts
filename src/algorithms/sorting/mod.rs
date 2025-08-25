pub mod counting_sort;

pub mod incremental_heap_sort;
pub mod incremental_quick_sort;
pub use incremental_quick_sort::IQS;

use crate::{data_generators::sorting::SortingInstance, experiments::ExperimentAlgorithm};

pub type AlgorithmType = ExperimentAlgorithm<SortingInstance<u32>, u32, Vec<u32>>;
