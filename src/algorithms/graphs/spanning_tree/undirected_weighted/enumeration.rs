use std::{cmp::Ordering, marker::PhantomData};

use compare::{Compare, Extract};

use crate::{
    algorithms::graphs::search::dfs::dfs_forest,
    data_structures::{
        graphs::{
            Adjacency, Direction, Edge, EdgeData, Forest, Graph, UndirectedAdjacencyArrayGraph,
            UndirectedGraph,
        },
        Index,
    },
    experiments::{ExperimentAlgorithm, PreparedEnumerationAlgorithm},
};

use super::{AlgorithmType, Boruvka, Kruskal, MstAlgorithm, MstPartial, Prim};

/// Enumeration algorithm for MSTs using Borůvka's MST algorithm as total-time black box
pub const ENUMERATE_WITH_BORUVKA: AlgorithmType = ExperimentAlgorithm::EnumerationAlgorithm(
    "enum-boruvka",
    EnumMST::<_, _, Boruvka>::enumerator_for,
);

pub const ENUMERATE_WITH_KRUSKAL: AlgorithmType = ExperimentAlgorithm::EnumerationAlgorithm(
    "enum-kruskal",
    EnumMST::<_, _, Kruskal>::enumerator_for,
);

pub const ENUMERATE_WITH_PRIM: AlgorithmType =
    ExperimentAlgorithm::EnumerationAlgorithm("enum-prim", EnumMST::<_, _, Prim>::enumerator_for);

pub struct EnumMST<I: Index, ED: EdgeData, BB: MstAlgorithm<I, ED>> {
    _phantom: PhantomData<(I, ED, BB)>,
}

impl<I: Index, ED: EdgeData, BB> MstAlgorithm<I, ED> for EnumMST<I, ED, BB>
where
    BB: MstAlgorithm<I, ED> + 'static,
{
    fn mst_for(graph: &impl UndirectedGraph<I, ED>) -> Forest<I, ED>
    where
        ED: Ord,
    {
        Self::comparator_st_for(graph, Extract::new(|e: &(I, I, ED)| e.data()))
    }

    fn comparator_st_for<C: Compare<(I, I, ED)>>(
        graph: &impl UndirectedGraph<I, ED>,
        comparator: C,
    ) -> Forest<I, ED> {
        let edges: Vec<_> =
            MstEnumerator::<_, _, _, _, BB>::with_comparator(graph, comparator).collect();
        let tree_graph =
            UndirectedAdjacencyArrayGraph::new_with_edge_data(graph.num_vertices(), &edges);
        dfs_forest(&tree_graph)
    }
}

impl<I: Index, ED: EdgeData, BB> EnumMST<I, ED, BB>
where
    BB: MstAlgorithm<I, ED> + 'static,
{
    pub fn enumerator_for(
        graph: &impl UndirectedGraph<I, ED>,
    ) -> PreparedEnumerationAlgorithm<MstPartial<I, ED>>
    where
        ED: Ord,
    {
        Self::comparator_enumerator_for(graph, Extract::new(|e: &(I, I, ED)| e.data()))
    }

    pub fn comparator_enumerator_for<C: Compare<(I, I, ED)> + 'static>(
        graph: &impl UndirectedGraph<I, ED>,
        comparator: C,
    ) -> PreparedEnumerationAlgorithm<MstPartial<I, ED>> {
        Box::new(MstEnumerator::<_, _, _, _, BB>::with_comparator(
            graph, comparator,
        ))
    }
}

/// Enumerating the edges of a minimum spanning tree.
enum MstEnumerator<'a, G, I, ED, C, BB>
where
    G: UndirectedGraph<I, ED>,
    I: Index,
    ED: EdgeData,
    C: Compare<(I, I, ED)>,
    BB: MstAlgorithm<I, ED>,
{
    CreditAccumulationPhase {
        graph: &'a G,
        comparator: C,
        iterator: I::IndexIterator,
        selected_edges: Vec<(I, I, ED)>,
        _phantom: PhantomData<BB>,
    },
    // ExtensionPhase happens immediately once CreditAccumulation does not produce a partial
    OutputFinalizationPhase {
        forest: Forest<I, ED>,
        mst: Forest<I, ED>,
        iterator: I::IndexIterator,
    },
}

impl<'a, G, I, ED, C, BB> MstEnumerator<'a, G, I, ED, C, BB>
where
    G: UndirectedGraph<I, ED>,
    I: Index,
    ED: EdgeData,
    C: Compare<(I, I, ED)>,
    BB: MstAlgorithm<I, ED>,
{
    /// Initialize a new enumerator for MST edges.
    ///
    /// The algorithm assumes that `graph` is connected and has undefined behavior if it is not.
    pub fn with_comparator(graph: &'a G, comparator: C) -> Self {
        Self::CreditAccumulationPhase {
            graph,
            comparator,
            iterator: I::zero().range(graph.num_vertices()),
            selected_edges: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<G, I, ED, C, BB> Iterator for MstEnumerator<'_, G, I, ED, C, BB>
where
    G: UndirectedGraph<I, ED>,
    I: Index,
    ED: EdgeData,
    C: Compare<(I, I, ED)>,
    BB: MstAlgorithm<I, ED>,
{
    type Item = MstPartial<I, ED>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Self::CreditAccumulationPhase {
            graph,
            comparator,
            iterator,
            selected_edges,
            ..
        } = self
        {
            match credit_accumulation_step(*graph, comparator, iterator) {
                Some(partial) => {
                    // still accumulating
                    selected_edges.push(partial);
                    return Some(partial);
                }
                None => {
                    // extension!
                    let mut forest = Forest::new_isolated_vertices(graph.num_vertices());
                    for e in selected_edges {
                        forest[e.source().index()] = Some((e.sink(), e.data()));
                    }
                    let mst = extension::<_, _, _, _, BB>(*graph, &forest, comparator);
                    let iterator = I::zero().range(mst.num_vertices());
                    *self = Self::OutputFinalizationPhase {
                        forest,
                        mst,
                        iterator,
                    }
                }
            }
        }

        if let Self::OutputFinalizationPhase {
            forest,
            mst,
            iterator,
        } = self
        {
            return output_finalization_step(forest, mst, iterator);
        }

        panic!("Iterating on an undefined state is not supported.");
    }
}

pub fn credit_accumulation_step<I: Index, ED: EdgeData, C, G>(
    graph: &G,
    comparator: &C,
    iterator: &mut I::IndexIterator,
) -> Option<MstPartial<I, ED>>
where
    G: UndirectedGraph<I, ED>,
    C: Compare<(I, I, ED)>,
{
    // consider edges with minimum weight; break ties in favor of smaller sink vertex id
    let edge_selection = |e1: &(I, I, ED), e2: &(I, I, ED)| {
        comparator
            .compare(e1, e2)
            .then_with(|| e1.sink().cmp(&e2.sink()))
    };

    for u in iterator {
        if let Some((_, v, w)) = graph
            .adjacencies(u, Direction::OUT)
            .map(|a| (u, a.sink(), a.data()))
            .min_by(edge_selection)
        {
            // vertex u is being processed before the edge's target vertex v?
            if u < v {
                return Some((u, v, w)); // first time we see this edge

            // else check whether the edge was already selected with origin v
            } else if let Some((_, p, _)) = graph
                .adjacencies(v, Direction::OUT)
                .map(|a| (u, a.sink(), a.data()))
                .min_by(edge_selection)
            {
                if p == u {
                    continue; // this edge was already selected with origin v
                } else {
                    return Some((u, v, w)); // v is the origin of some other selected edge
                }
            } else {
                panic!("There should have been a selected edge with origin v.")
            }
        } else {
            panic!("The graph is not connected.")
        }
    }
    None
}

fn extension<G, I, ED, C, BB>(graph: &G, forest: &Forest<I, ED>, comparator: &C) -> Forest<I, ED>
where
    G: UndirectedGraph<I, ED>,
    I: Index,
    ED: EdgeData,
    C: Compare<(I, I, ED)>,
    BB: MstAlgorithm<I, ED>,
{
    BB::comparator_st_for(graph, |e1: &(I, I, ED), e2: &(I, I, ED)| {
        let preselected1 = is_preselected(forest, e1);
        let preselected2 = is_preselected(forest, e2);
        if preselected1 == preselected2 {
            comparator.compare(e1, e2)
        } else if preselected1 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    })
}

fn output_finalization_step<I: Index, ED: EdgeData>(
    forest: &Forest<I, ED>,
    mst: &Forest<I, ED>,
    iterator: &mut I::IndexIterator,
) -> Option<MstPartial<I, ED>> {
    for u in iterator {
        match mst[u.index()] {
            Some((v, w)) => {
                let partial = (u, v, w);
                if is_preselected(forest, &partial) {
                    // this edge was already emitted in the credit accumulation phase
                    continue;
                }
                return Some(partial);
            }
            None => continue,
        }
    }
    None
}

fn is_preselected<I: Index, ED: EdgeData>(forest: &Forest<I, ED>, edge: &(I, I, ED)) -> bool {
    forest[edge.source().index()].is_some_and(|a| a.sink() == edge.sink())
        || forest[edge.sink().index()].is_some_and(|a| a.sink() == edge.source())
}
