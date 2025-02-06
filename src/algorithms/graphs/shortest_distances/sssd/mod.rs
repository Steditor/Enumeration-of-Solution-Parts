use crate::experiments::ExperimentAlgorithm;

use super::ShortestDistancePartial;

pub mod unweighted;
pub mod unweighted_lazy;
pub mod weighted;

/// SSSD algorithms take a graph of type `G` and a source vertex of type `I`
/// and output shortest distances of type `D` from the source to all other vertices.
///
/// The default `D = I` distance implies hop distance.
pub type AlgorithmType<G, I, D = I> =
    ExperimentAlgorithm<(G, I), ShortestDistancePartial<I, D>, Vec<Option<D>>>;
