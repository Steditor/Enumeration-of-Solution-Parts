use crate::{data_structures::Matrix, experiments::ExperimentAlgorithm};

use super::ShortestDistancePartial;

pub mod unweighted;
pub mod unweighted_no_self;
pub mod unweighted_sorted;
pub mod weighted;
pub mod weighted_no_self;
pub mod weighted_sorted;

#[cfg(test)]
mod tests;

pub type AlgorithmType<G, I, D = I> =
    ExperimentAlgorithm<G, ShortestDistancePartial<I, D>, Matrix<Option<D>>>;
