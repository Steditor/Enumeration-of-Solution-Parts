use std::{collections::VecDeque, marker::PhantomData};

use crate::{
    algorithms::graphs::shortest_distances::ShortestDistancePartial,
    data_structures::{
        graphs::{Direction, EdgeData, Graph},
        Index, LazyArray,
    },
    experiments::ExperimentAlgorithm,
};

use super::AlgorithmType;

pub const fn algorithm_enum_bfs_lazy<G, I, ED>() -> AlgorithmType<G, I>
where
    G: Graph<I, ED>,
    I: Index,
    ED: EdgeData,
{
    ExperimentAlgorithm::EnumerationAlgorithm("enum-bfs-lazy", |(graph, source)| {
        Box::new(SssdEnumerator::new(graph, *source))
    })
}

/// Enumerate unweighted hop SSSD via incremental BFS
pub enum SssdEnumerator<'a, G, I, ED>
where
    G: Graph<I, ED> + ?Sized,
    I: Index,
    ED: EdgeData,
{
    CreditAccumulationPhase {
        graph: &'a G,
        source: I,
        distances: LazyArray<I>,
        queue: VecDeque<I>,
    },
    // No real extension phase; once ibfs finishes, all distances are known, just not necessarily emitted.
    OutputFinalizationPhase {
        source: I,
        distances: LazyArray<I>,
        iterator: I::IndexIterator,
    },
    Undefined {
        _phantom: PhantomData<ED>,
    },
}

impl<G, I, ED> Default for SssdEnumerator<'_, G, I, ED>
where
    G: Graph<I, ED> + ?Sized,
    I: Index,
    ED: EdgeData,
{
    fn default() -> Self {
        Self::Undefined {
            _phantom: PhantomData,
        }
    }
}

impl<'a, G, I, ED> SssdEnumerator<'a, G, I, ED>
where
    G: Graph<I, ED> + ?Sized,
    I: Index,
    ED: EdgeData,
{
    pub fn new(graph: &'a G, source: I) -> Self {
        let mut distances = LazyArray::new(graph.num_vertices().index());
        let mut queue = VecDeque::new();

        distances.set(source.index(), I::zero());
        queue.push_back(source);

        Self::CreditAccumulationPhase {
            graph,
            source,
            distances,
            queue,
        }
    }
}

impl<G, I, ED> Iterator for SssdEnumerator<'_, G, I, ED>
where
    G: Graph<I, ED> + ?Sized,
    I: Index,
    ED: EdgeData,
{
    type Item = ShortestDistancePartial<I, I>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Self::CreditAccumulationPhase {
            graph,
            source,
            distances,
            queue,
        } = self
        {
            if let Some(u) = queue.pop_front() {
                let d = unsafe {
                    // SAFETY: only elements with set distance are ever put in the queue
                    *distances.get(u.index()).unwrap_unchecked()
                };
                let new_d = d + I::one();
                for v in graph.neighbors(u, Direction::OUT) {
                    if distances.get(v.index()).is_none() {
                        distances.set(v.index(), new_d);
                        queue.push_back(v);
                    }
                }
                return Some((*source, u, Some(d)));
            }
            // still here? BFS is done; prepare the next phase!
            if let Self::CreditAccumulationPhase {
                graph,
                source,
                distances,
                ..
            } = std::mem::take(self)
            {
                let iterator = graph.vertices();
                *self = Self::OutputFinalizationPhase {
                    source,
                    distances,
                    iterator,
                };
            }
        }

        if let Self::OutputFinalizationPhase {
            source,
            distances,
            iterator,
        } = self
        {
            for v in iterator {
                match distances.get(v.index()) {
                    Some(_) => continue,
                    None => return Some((*source, v, None)),
                }
            }
            return None;
        }

        panic!("Iterating on an undefined state is not supported")
    }
}

#[cfg(test)]
mod test {
    use crate::data_structures::graphs::{
        DirectedAdjacencyArrayGraph, UndirectedAdjacencyArrayGraph,
    };

    use super::*;

    /// BFS example in Figure 22.3 of CRLS 3rd edition
    const CRLS_22_3_EDGES: [(u32, u32); 10] = [
        (0, 4),
        (1, 5),
        (0, 1),
        (2, 3),
        (2, 6),
        (3, 6),
        (3, 7),
        (5, 2),
        (5, 6),
        (6, 7),
    ];

    fn undirected_crls_22_3() -> UndirectedAdjacencyArrayGraph<u32> {
        UndirectedAdjacencyArrayGraph::new(8, &CRLS_22_3_EDGES)
    }

    fn directed_crls_22_3() -> DirectedAdjacencyArrayGraph<u32> {
        DirectedAdjacencyArrayGraph::new(8, &CRLS_22_3_EDGES)
    }

    #[test]
    fn test_undirected_sssd_enumeration_crls_22_3() {
        let graph = undirected_crls_22_3();
        let parts: Vec<_> = SssdEnumerator::new(&graph, 1).collect();
        assert_eq!(parts.len(), graph.num_vertices().index());

        let mut distances = vec![None; graph.num_vertices().index()];
        for (u, v, d) in parts {
            assert_eq!(u, 1);
            distances[v.index()] = d;
        }
        assert_eq!(distances, [1, 0, 2, 3, 2, 1, 2, 3].map(Some));
    }

    #[test]
    fn test_directed_sssd_enumeration_crls_22_3() {
        let graph = directed_crls_22_3();
        let parts: Vec<_> = SssdEnumerator::new(&graph, 1).collect();
        assert_eq!(parts.len(), graph.num_vertices().index());

        let mut distances = vec![None; graph.num_vertices().index()];
        for (u, v, d) in parts {
            assert_eq!(u, 1);
            distances[v.index()] = d;
        }
        assert_eq!(
            distances,
            [
                None,
                Some(0),
                Some(2),
                Some(3),
                None,
                Some(1),
                Some(2),
                Some(3)
            ]
        );
    }
}
