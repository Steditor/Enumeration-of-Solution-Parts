mod boruvka;
mod enumeration;
mod kruskal;
mod prim;

pub use boruvka::{Boruvka, IncrementalBoruvka, BORUVKA, INCREMENTAL_BORUVKA};
pub use enumeration::{
    EnumMST, ENUMERATE_WITH_BORUVKA, ENUMERATE_WITH_KRUSKAL, ENUMERATE_WITH_PRIM,
};
pub use kruskal::{
    IncrementalKruskal, Kruskal, INCREMENTAL_KRUSKAL, KRUSKAL_IQS, KRUSKAL_RUSTSORT,
};
pub use prim::{IncrementalPrim, Prim, INCREMENTAL_PRIM, PRIM};

use compare::Compare;

use crate::{
    data_structures::{
        graphs::{EdgeData, Forest, UndirectedAdjacencyArrayGraph, UndirectedGraph},
        Index,
    },
    experiments::ExperimentAlgorithm,
};

/// A partial for a minimum spanning tree is simply a weighted edge
pub type MstPartial<I, ED> = (I, I, ED);

pub type AlgorithmType = ExperimentAlgorithm<
    UndirectedAdjacencyArrayGraph<u32, u32>,
    MstPartial<u32, u32>,
    Forest<u32, u32>,
>;

/// An algorithm to compute a minimum spanning tree for an undirected graph
pub trait MstAlgorithm<I: Index, ED: EdgeData> {
    /// Compute a spanning tree that minimizes according to the natural order of the edge data.
    fn mst_for(graph: &impl UndirectedGraph<I, ED>) -> Forest<I, ED>
    where
        ED: Ord;

    /// Compute a spanning tree that minimizes according to the given comparator.
    fn comparator_st_for<C: Compare<(I, I, ED)>>(
        graph: &impl UndirectedGraph<I, ED>,
        comparator: C,
    ) -> Forest<I, ED>;
}

#[cfg(test)]
mod tests;
