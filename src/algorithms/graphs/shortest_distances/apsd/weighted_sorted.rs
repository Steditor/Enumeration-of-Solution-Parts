use num::Unsigned;

use crate::{
    data_structures::{
        graphs::{EdgeWeight, Graph},
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
    ExperimentAlgorithm::EnumerationAlgorithm("apsd-enum-dijkstra-sorted", |graph| {
        enumerate::prepare_enumeration(graph)
    })
}

mod enumerate {
    use std::marker::PhantomData;

    use binary_heap_plus::BinaryHeap;
    use compare::Compare;
    use num::Unsigned;

    use crate::{
        algorithms::graphs::shortest_distances::{
            sssd::weighted::SssdEnumerator, ShortestDistancePartial,
        },
        data_structures::{
            graphs::{EdgeWeight, Graph},
            Index,
        },
        experiments::PreparedEnumerationAlgorithm,
    };

    pub fn prepare_enumeration<G, I, EW>(
        graph: &G,
    ) -> PreparedEnumerationAlgorithm<ShortestDistancePartial<I, EW>>
    where
        G: Graph<I, EW> + ?Sized,
        I: Index,
        EW: EdgeWeight + Unsigned,
    {
        let trivial_iterator = Box::new(graph.vertices().map(|u| (u, u, Some(EW::zero()))));

        let extension_iterator = ParallelDijkstra::new(graph);

        Box::new(trivial_iterator.chain(extension_iterator))
    }

    struct MinDijkstraComparator<G, I, EW>
    where
        G: Graph<I, EW> + ?Sized,
        I: Index,
        EW: EdgeWeight + Unsigned,
    {
        _phantom: PhantomData<(I, EW, G)>,
    }
    impl<G, I, EW> Default for MinDijkstraComparator<G, I, EW>
    where
        G: Graph<I, EW> + ?Sized,
        I: Index,
        EW: EdgeWeight + Unsigned,
    {
        fn default() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }

    type ParallelDijkstraHeapEntry<'a, G, I, EW> =
        (ShortestDistancePartial<I, EW>, SssdEnumerator<'a, G, I, EW>);

    impl<G, I, EW> Compare<ParallelDijkstraHeapEntry<'_, G, I, EW>> for MinDijkstraComparator<G, I, EW>
    where
        G: Graph<I, EW> + ?Sized,
        I: Index,
        EW: EdgeWeight + Unsigned,
    {
        fn compare(
            &self,
            ((_, _, l_dist), _): &(ShortestDistancePartial<I, EW>, SssdEnumerator<'_, G, I, EW>),
            ((_, _, r_dist), _): &(ShortestDistancePartial<I, EW>, SssdEnumerator<'_, G, I, EW>),
        ) -> std::cmp::Ordering {
            match (l_dist, r_dist) {
                (None, None) => std::cmp::Ordering::Equal,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Less,
                (Some(l), Some(r)) => l.cmp(r),
            }
            .reverse() // we need the comparison for a max heap
        }
    }

    type ParallelDijkstraHeap<'a, G, I, EW> =
        BinaryHeap<ParallelDijkstraHeapEntry<'a, G, I, EW>, MinDijkstraComparator<G, I, EW>>;

    struct ParallelDijkstra<'a, G, I, EW>
    where
        G: Graph<I, EW> + ?Sized,
        I: Index,
        EW: EdgeWeight + Unsigned,
    {
        graph: &'a G,
        dijkstra_queue: Option<ParallelDijkstraHeap<'a, G, I, EW>>,
    }

    impl<'a, G, I, EW> ParallelDijkstra<'a, G, I, EW>
    where
        G: Graph<I, EW> + ?Sized,
        I: Index,
        EW: EdgeWeight + Unsigned,
    {
        fn new(graph: &'a G) -> Self {
            Self {
                graph,
                dijkstra_queue: None,
            }
        }

        fn initialize_searches(&mut self) {
            let dijkstra_instances: Vec<_> = self
                .graph
                .vertices()
                .filter_map(|source| {
                    // prepare dijkstra for source vertex
                    let mut dijkstra = SssdEnumerator::new(self.graph, source);
                    // skip self-distance that was already emitted for each search:
                    dijkstra.next();
                    // prepare the next part which is the key for the priority queue:
                    let first_part = dijkstra.next();
                    // only consider this search if it yields a first part:
                    first_part.map(|part| (part, dijkstra))
                })
                .collect();

            // order the searches in a binary heap
            self.dijkstra_queue = Some(BinaryHeap::from_vec_cmp(
                dijkstra_instances,
                MinDijkstraComparator::default(),
            ));
        }
    }

    impl<G, I, EW> Iterator for ParallelDijkstra<'_, G, I, EW>
    where
        G: Graph<I, EW> + ?Sized,
        I: Index,
        EW: EdgeWeight + Unsigned,
    {
        type Item = ShortestDistancePartial<I, EW>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.dijkstra_queue.is_none() {
                self.initialize_searches();
            }
            let dijkstra_queue = self.dijkstra_queue.as_mut().expect("We initialized above.");

            let (next_part, mut dijkstra) = dijkstra_queue.pop()?;

            match dijkstra.next() {
                // if there is a next part, re-insert the search with the next part as key
                Some(peek_part) => dijkstra_queue.push((peek_part, dijkstra)),
                None => (), // this search is done → discard it
            }

            Some(next_part)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::algorithms::graphs::shortest_distances::apsd::tests::{
        check_enumeration_result, directed_nonnegative_crls_23_4,
        directed_nonnegative_crls_23_4_solution,
    };

    use super::*;

    #[test]
    fn test_directed_enumeration() {
        let graph = directed_nonnegative_crls_23_4();
        let solution_parts: Vec<_> = enumerate::prepare_enumeration(&graph).collect();
        check_enumeration_result(
            &solution_parts,
            &directed_nonnegative_crls_23_4_solution(),
            false,
            true,
        );
    }
}
