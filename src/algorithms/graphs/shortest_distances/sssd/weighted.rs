use binary_heap_plus::{BinaryHeap, MinComparator};
use num::Unsigned;

use crate::{
    algorithms::graphs::shortest_distances::ShortestDistancePartial,
    data_structures::{
        graphs::{Direction, EdgeWeight, Graph},
        Index,
    },
    experiments::ExperimentAlgorithm,
};

use super::AlgorithmType;

pub const fn algorithm_enum_dijkstra<G, I, EW>() -> AlgorithmType<G, I, EW>
where
    G: Graph<I, EW>,
    I: Index,
    EW: EdgeWeight + Unsigned,
{
    ExperimentAlgorithm::EnumerationAlgorithm("enum-dijkstra", |(graph, source)| {
        Box::new(SssdEnumerator::new(graph, *source))
    })
}

/// Enumerate weighted SSSD via incremental Dijkstra
pub enum SssdEnumerator<'a, G, I, EW>
where
    G: Graph<I, EW> + ?Sized,
    I: Index,
    EW: EdgeWeight + Unsigned,
{
    CreditAccumulationPhase {
        graph: &'a G,
        source: I,
        distances: Vec<Option<EW>>,
        priority_queue: BinaryHeap<(EW, I), MinComparator>,
    },
    // No real extension phase; once dijkstra finishes, all distances are known, just not necessarily emitted.
    OutputFinalizationPhase {
        source: I,
        distances: Vec<Option<EW>>,
        iterator: I::IndexIterator,
    },
    Undefined {},
}

impl<G, I, EW> Default for SssdEnumerator<'_, G, I, EW>
where
    G: Graph<I, EW> + ?Sized,
    I: Index,
    EW: EdgeWeight + Unsigned,
{
    fn default() -> Self {
        Self::Undefined {}
    }
}

impl<'a, G, I, EW> SssdEnumerator<'a, G, I, EW>
where
    G: Graph<I, EW> + ?Sized,
    I: Index,
    EW: EdgeWeight + Unsigned,
{
    pub fn new(graph: &'a G, source: I) -> Self {
        let mut distances = vec![None; graph.num_vertices().index()];
        let mut priority_queue = BinaryHeap::new_min();

        distances[source.index()] = Some(EW::zero());
        priority_queue.push((EW::zero(), source));

        Self::CreditAccumulationPhase {
            graph,
            source,
            distances,
            priority_queue,
        }
    }
}

impl<G, I, EW> Iterator for SssdEnumerator<'_, G, I, EW>
where
    G: Graph<I, EW> + ?Sized,
    I: Index,
    EW: EdgeWeight + Unsigned,
{
    type Item = ShortestDistancePartial<I, EW>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Self::CreditAccumulationPhase {
            graph,
            source,
            distances,
            priority_queue,
        } = self
        {
            while let Some((d, u)) = priority_queue.pop() {
                // SAFETY: only elements with set distance are ever put in the priority queue
                unsafe {
                    // This entry in the priority queue was 'deprecated' by a later 'decrease-key'
                    if d > distances[u.index()].unwrap_unchecked() {
                        continue;
                    }
                }

                for (v, w) in graph.adjacencies(u, Direction::OUT) {
                    let new_d = d + w;
                    if distances[v.index()].is_none_or(|old_d| new_d < old_d) {
                        distances[v.index()] = Some(new_d);
                        priority_queue.push((new_d, v));
                    }
                }

                // u is finished now
                return Some((*source, u, Some(d)));
            }
            // still here? Dijkstra is done; prepare the next phase!
            if let Self::CreditAccumulationPhase {
                graph,
                source,
                distances,
                ..
            } = std::mem::take(self)
            {
                let iterator = I::zero().range(graph.num_vertices());
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
                match distances[v.index()] {
                    Some(_) => continue,
                    None => return Some((*source, v, None)),
                }
            }
            return None;
        }

        panic!("Iterating on an undefined state is not supported")
    }
}

pub const fn algorithm_dijkstra<G, I, EW>() -> AlgorithmType<G, I, EW>
where
    G: Graph<I, EW>,
    I: Index,
    EW: EdgeWeight + Unsigned,
{
    ExperimentAlgorithm::TotalTimeAlgorithm("dijkstra", |(graph, source)| {
        Ok(dijkstra(graph, *source))
    })
}

pub fn dijkstra<G, I, EW>(graph: &G, source: I) -> Vec<Option<EW>>
where
    G: Graph<I, EW> + ?Sized,
    I: Index,
    EW: EdgeWeight + Unsigned,
{
    let mut distances = vec![None; graph.num_vertices().index()];
    // We don't have a decrease-key operation and add target vertices multiple times instead.
    let mut priority_queue = BinaryHeap::new_min();

    distances[source.index()] = Some(EW::zero());
    priority_queue.push((EW::zero(), source));

    while let Some((d, u)) = priority_queue.pop() {
        // SAFETY: only elements with set distance are ever put in the priority queue
        unsafe {
            // This entry in the priority queue was 'deprecated' by a later 'decrease-key'
            if d > distances[u.index()].unwrap_unchecked() {
                continue;
            }
        }

        for (v, w) in graph.adjacencies(u, Direction::OUT) {
            let new_d = d + w;
            if distances[v.index()].is_none_or(|old_d| new_d < old_d) {
                distances[v.index()] = Some(new_d);
                priority_queue.push((new_d, v));
            }
        }
    }

    distances
}

#[cfg(test)]
mod test {
    use crate::data_structures::graphs::{
        DirectedAdjacencyArrayGraph, UndirectedAdjacencyArrayGraph,
    };

    use super::*;

    /// Dijkstra example in Figure 22.6 of CRLS 4th edition
    const CRLS_22_6_EDGES: [(u32, u32, u32); 10] = [
        (0, 1, 10),
        (0, 3, 5),
        (1, 2, 1),
        (1, 3, 2),
        (2, 4, 4),
        (3, 1, 3),
        (3, 2, 9),
        (3, 4, 2),
        (4, 0, 7),
        (4, 2, 6),
    ];

    /// Undirected adaptation of Dijkstra example from Figure 22.6 of CRLS 4th edition
    const CRLS_22_6_UNDIRECTED_EDGES: [(u32, u32, u32); 8] = [
        (0, 1, 10),
        (0, 3, 5),
        (1, 2, 1),
        (1, 3, 2),
        (2, 4, 4),
        (3, 2, 9),
        (3, 4, 2),
        (4, 0, 7),
    ];

    fn directed_crls_22_6() -> DirectedAdjacencyArrayGraph<u32, u32> {
        DirectedAdjacencyArrayGraph::new_with_edge_data(5, &CRLS_22_6_EDGES)
    }

    fn undirected_crls_22_6() -> UndirectedAdjacencyArrayGraph<u32, u32> {
        UndirectedAdjacencyArrayGraph::new_with_edge_data(5, &CRLS_22_6_UNDIRECTED_EDGES)
    }

    #[test]
    fn test_directed_sssd_crls_22_6() {
        let graph = directed_crls_22_6();
        let distances = dijkstra(&graph, 0);
        assert_eq!(distances, [0, 8, 9, 5, 7].map(Some));
    }

    #[test]
    fn test_undirected_sssd_crls_22_6() {
        let graph = undirected_crls_22_6();
        let distances = dijkstra(&graph, 0);
        assert_eq!(distances, [0, 7, 8, 5, 7].map(Some));
    }

    #[test]
    fn test_directed_sssd_enumeration_crls_22_6() {
        let graph = directed_crls_22_6();
        let parts: Vec<_> = SssdEnumerator::new(&graph, 0).collect();
        assert_eq!(parts.len(), graph.num_vertices().index());

        let mut distances = vec![None; graph.num_vertices().index()];
        for (u, v, d) in parts {
            assert_eq!(u, 0);
            distances[v.index()] = d;
        }
        assert_eq!(distances, [0, 8, 9, 5, 7].map(Some));
    }

    #[test]
    fn test_undirected_sssd_enumeration_crls_22_6() {
        let graph = undirected_crls_22_6();
        let parts: Vec<_> = SssdEnumerator::new(&graph, 0).collect();
        assert_eq!(parts.len(), graph.num_vertices().index());

        let mut distances = vec![None; graph.num_vertices().index()];
        for (u, v, d) in parts {
            assert_eq!(u, 0);
            distances[v.index()] = d;
        }
        assert_eq!(distances, [0, 7, 8, 5, 7].map(Some));
    }
}
