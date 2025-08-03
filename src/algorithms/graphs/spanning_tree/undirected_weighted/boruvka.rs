use std::{cmp::Ordering, collections::VecDeque, marker::PhantomData};

use compare::{Compare, Extract};

use crate::{
    algorithms::graphs::{
        search::dfs::dfs_forest,
        spanning_tree::undirected_weighted::{enumeration::credit_accumulation_step, MstPartial},
    },
    data_structures::{
        graphs::{Edge, EdgeData, Forest, Graph, UndirectedAdjacencyArrayGraph, UndirectedGraph},
        union_find::{DisjointSet, UnionFind},
        Index,
    },
    experiments::{ExperimentAlgorithm, PreparedEnumerationAlgorithm},
};

use super::{AlgorithmType, MstAlgorithm};

pub const BORUVKA: AlgorithmType =
    ExperimentAlgorithm::TotalTimeAlgorithm("total-boruvka", |graph| Ok(Boruvka::mst_for(graph)));

pub const INCREMENTAL_BORUVKA: AlgorithmType = ExperimentAlgorithm::EnumerationAlgorithm(
    "incremental-boruvka",
    IncrementalBoruvka::enumerator_for,
);

/// Borůvka's MST algorithm
pub struct Boruvka {}

impl<I: Index, ED: EdgeData> MstAlgorithm<I, ED> for Boruvka {
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
        let mut tree_edges: Vec<(I, I, ED)> = Vec::with_capacity(graph.num_vertices().index() - 1);
        let mut components = UnionFind::new_with_size(graph.num_vertices());

        let mut crossing_edges: Vec<_> = graph.edges().collect();
        while !crossing_edges.is_empty() {
            let mut new_edges: Vec<Option<(I, I, ED)>> =
                vec![Option::None; graph.num_vertices().index()];

            // find a minimum weight crossing edge per component
            for e in crossing_edges.iter() {
                let source_component = components.find(e.source());
                new_edges[source_component.index()] = match new_edges[source_component.index()] {
                    None => Some(*e),
                    Some(old_e) => match comparator.compare(e, &old_e) {
                        Ordering::Less | Ordering::Equal => Some(*e),
                        Ordering::Greater => Some(old_e),
                    },
                };
            }

            // merge components via crossing edges
            for e in new_edges.iter().flatten() {
                // Could be that the edge is no longer crossing,
                // because of a previous union.
                // This in turn eliminates the need for an additional tiebreaker.
                if !components.is_same(e.source(), e.sink()) {
                    tree_edges.push(*e);
                    components.union(e.source(), e.sink());
                }
            }

            // Remove all edges from consideration that do no longer cross components.
            // `flatten()` before, so the components are the same if their parents are identical.
            components.flatten();
            crossing_edges.retain(|&e| !components.is_same_parent(e.source(), e.sink()));
        }

        let tree_graph =
            UndirectedAdjacencyArrayGraph::new_with_edge_data(graph.num_vertices(), &tree_edges);
        dfs_forest(&tree_graph)
    }
}

pub struct IncrementalBoruvka<I: Index, ED: EdgeData> {
    _phantom: PhantomData<(I, ED)>,
}

impl<I: Index, ED: EdgeData> IncrementalBoruvka<I, ED> {
    pub fn enumerator_for(
        graph: &impl UndirectedGraph<I, ED>,
    ) -> PreparedEnumerationAlgorithm<MstPartial<I, ED>>
    where
        ED: Ord,
    {
        Self::comparator_enumerator_for(graph, Extract::new(|e: &(I, I, ED)| e.data()))
    }

    pub fn comparator_enumerator_for<C: Compare<(I, I, ED)> + Copy + 'static>(
        graph: &impl UndirectedGraph<I, ED>,
        comparator: C,
    ) -> PreparedEnumerationAlgorithm<MstPartial<I, ED>> {
        Box::new(MstEnumerator::with_comparator(graph, comparator))
    }
}

enum MstEnumerator<'a, G, I, ED, C>
where
    G: UndirectedGraph<I, ED>,
    I: Index,
    ED: EdgeData,
    C: Compare<(I, I, ED)>,
{
    CreditAccumulationPhase {
        graph: &'a G,
        comparator: C,
        iterator: I::IndexIterator,
        components: UnionFind<I>,
    },
    ExtensionPhase {
        crossing_edges: Vec<(I, I, ED)>,
        comparator: C,
        components: UnionFind<I>,
        edge_queue: VecDeque<(I, I, ED)>,
    },
    Undefined {},
}

impl<G, I, ED, C> Default for MstEnumerator<'_, G, I, ED, C>
where
    G: UndirectedGraph<I, ED>,
    I: Index,
    ED: EdgeData,
    C: Compare<(I, I, ED)>,
{
    fn default() -> Self {
        Self::Undefined {}
    }
}

impl<'a, G, I, ED, C> MstEnumerator<'a, G, I, ED, C>
where
    G: UndirectedGraph<I, ED>,
    I: Index,
    ED: EdgeData,
    C: Compare<(I, I, ED)>,
{
    pub fn with_comparator(graph: &'a G, comparator: C) -> Self {
        Self::CreditAccumulationPhase {
            graph,
            comparator,
            iterator: I::zero().range(graph.num_vertices()),
            components: UnionFind::new_with_size(graph.num_vertices()),
        }
    }
}

impl<G, I, ED, C> Iterator for MstEnumerator<'_, G, I, ED, C>
where
    G: UndirectedGraph<I, ED>,
    I: Index,
    ED: EdgeData,
    C: Compare<(I, I, ED)>,
{
    type Item = MstPartial<I, ED>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Self::CreditAccumulationPhase {
            graph,
            comparator,
            iterator,
            components,
        } = self
        {
            // First, quickly select one minimum-weight crossing edge per vertex,
            // just like our enumeration algorithm does in the credit accumulation step
            if let Some(e) = credit_accumulation_step(*graph, comparator, iterator) {
                components.union(e.source(), e.sink());
                return Some(e);
            }
            // Still here?
            // Then prepare the extension!
            if let Self::CreditAccumulationPhase {
                graph,
                comparator,
                components,
                ..
            } = std::mem::take(self)
            {
                let crossing_edges: Vec<_> = graph
                    .edges()
                    .filter(|e| !components.is_same_parent(e.source(), e.sink()))
                    .collect();

                *self = Self::ExtensionPhase {
                    crossing_edges,
                    comparator,
                    components,
                    edge_queue: VecDeque::with_capacity(graph.num_vertices().index() / 2 + 1),
                }
            }
        }

        if let Self::ExtensionPhase {
            crossing_edges,
            comparator,
            components,
            edge_queue,
        } = self
        {
            // After the credit accumulation step, do regular Borůvka for extension.
            // This means selecting edges in batches, thus if there are still some left from the last batch,
            // emit them as solution part.
            if let Some(e) = edge_queue.pop_front() {
                return Some(e);
            }

            // No pre-computed solution parts and no more crossing edges? We're done.
            if crossing_edges.is_empty() {
                return None;
            }

            // else, compute a new batch of solution parts with standard boruvka.
            let mut new_edges: Vec<Option<(I, I, ED)>> =
                vec![Option::None; components.num_elements().index()];

            // find a minimum weight crossing edge per component
            for e in crossing_edges.iter() {
                let source_component = components.find(e.source());
                new_edges[source_component.index()] = match new_edges[source_component.index()] {
                    None => Some(*e),
                    Some(old_e) => match comparator.compare(e, &old_e) {
                        Ordering::Less | Ordering::Equal => Some(*e),
                        Ordering::Greater => Some(old_e),
                    },
                };
            }

            // merge components via crossing edges
            for e in new_edges.iter().flatten() {
                // Could be that the edge is no longer crossing,
                // because of a previous union.
                // This in turn eliminates the need for an additional tiebreaker.
                if !components.is_same(e.source(), e.sink()) {
                    edge_queue.push_back(*e);
                    components.union(e.source(), e.sink());
                }
            }

            // Remove all edges from consideration that do no longer cross components.
            // `flatten()` before, so the components are the same if their parents are identical.
            components.flatten();
            crossing_edges.retain(|&e| !components.is_same_parent(e.source(), e.sink()));

            return edge_queue.pop_front();
        }

        panic!("Iterating on an undefined state is not supported.");
    }
}
