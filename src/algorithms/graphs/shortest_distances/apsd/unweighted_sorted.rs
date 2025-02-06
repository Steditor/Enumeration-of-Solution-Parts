use crate::{
    data_structures::{
        graphs::{EdgeData, Graph},
        Index,
    },
    experiments::ExperimentAlgorithm,
};

use super::AlgorithmType;

pub const fn algorithm_enum_bfs<G, I, ED>() -> AlgorithmType<G, I>
where
    G: Graph<I, ED>,
    I: Index,
    ED: EdgeData,
{
    ExperimentAlgorithm::EnumerationAlgorithm("apsd-enum-bfs-sorted", |graph| {
        enumerate::prepare_enumeration(graph)
    })
}

mod enumerate {
    use std::{collections::VecDeque, iter::Peekable};

    use crate::{
        algorithms::graphs::shortest_distances::{
            sssd::unweighted::SssdEnumerator, ShortestDistancePartial,
        },
        data_structures::{
            graphs::{EdgeData, Graph},
            Index,
        },
        experiments::PreparedEnumerationAlgorithm,
    };

    pub fn prepare_enumeration<G, I, ED>(
        graph: &G,
    ) -> PreparedEnumerationAlgorithm<ShortestDistancePartial<I, I>>
    where
        G: Graph<I, ED> + ?Sized,
        I: Index,
        ED: EdgeData,
    {
        let trivial_iterator = Box::new(graph.vertices().map(|u| (u, u, Some(I::zero()))));

        let extension_iterator = ParallelBfs::new(graph);

        Box::new(trivial_iterator.chain(extension_iterator))
    }

    struct ParallelBfs<'a, G, I, ED>
    where
        G: Graph<I, ED> + ?Sized,
        I: Index,
        ED: EdgeData,
    {
        graph: &'a G,
        bfs_queue: Option<VecDeque<Peekable<SssdEnumerator<'a, G, I, ED>>>>,
        current_hop_distance: I,
        finalization_queue: PreparedEnumerationAlgorithm<'a, ShortestDistancePartial<I, I>>,
    }

    impl<'a, G, I, ED> ParallelBfs<'a, G, I, ED>
    where
        G: Graph<I, ED> + ?Sized,
        I: Index,
        ED: EdgeData,
    {
        fn new(graph: &'a G) -> Self {
            // no finished searches yet
            let finalization_queue = Box::new(std::iter::empty());
            Self {
                graph,
                bfs_queue: None, // delay the initialization of the BFSs
                current_hop_distance: I::zero(),
                finalization_queue,
            }
        }

        fn initialize_searches(&mut self) {
            let mut bfs_queue: VecDeque<_> = self
                .graph
                .vertices()
                .map(|source| SssdEnumerator::new(self.graph, source).peekable())
                .collect();
            for search in bfs_queue.iter_mut() {
                search.next(); // skip self-distance that was already emitted for each search
            }

            let queue_len = bfs_queue.len();
            self.bfs_queue = Some(bfs_queue);

            // Cycle once through all searches and move the ones without remaining 'reachable' parts to the finalization queue
            for _ in 0..queue_len {
                // This makes use of the fact that all 0-distances are already gone, but we still have a current_hop_distance of 0.
                // → The head-search is either moved to the back of the queue or to the finalization queue
                self.advance_head_search();
            }

            // Now we dealt with all initial setup; next solutions have hop distance 1
            self.current_hop_distance = I::one();
        }

        fn advance_head_search(&mut self) {
            let bfs_queue = self
                .bfs_queue
                .as_mut()
                .expect("This method must not be called before the searches are initialized");
            let mut head_search = match bfs_queue.pop_front() {
                Some(bfs) => bfs,
                None => return, // no remaining head search to advance
            };
            match head_search.peek() {
                // next solution part has the same hop distance as the current level?
                Some((_, _, Some(d))) if *d == self.current_hop_distance => {
                    bfs_queue.push_front(head_search) // continue here in next round!
                }
                // next solution part has larger hop distance than the current?
                Some((_, _, Some(_))) => {
                    bfs_queue.push_back(head_search); // deal with other bfs instances first
                }
                // next solution part is to an unreachable vertex?
                Some((_, _, None)) => {
                    // these kinds of solutions are emitted from the finalization queue, so chain there!
                    // we need to move finalization_queue out of self to be able to change it
                    let old_queue = std::mem::replace(
                        &mut self.finalization_queue,
                        Box::new(std::iter::empty()),
                    );
                    self.finalization_queue = Box::new(old_queue.chain(head_search));
                }
                // there is no next solution part?
                _ => (), // discard that search, there are no more solution parts
            }
        }
    }

    impl<G, I, ED> Iterator for ParallelBfs<'_, G, I, ED>
    where
        G: Graph<I, ED> + ?Sized,
        I: Index,
        ED: EdgeData,
    {
        type Item = ShortestDistancePartial<I, I>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.bfs_queue.is_none() {
                self.initialize_searches();
            }
            let bfs_queue = self.bfs_queue.as_mut().expect("We did initialize above.");

            let solution_part = match bfs_queue.front_mut() {
                Some(bfs) => bfs.next(),
                None => return self.finalization_queue.next(),
            };

            // This solution part has a finite but higher hop distance than the previous?
            if let Some((_, _, Some(d))) = solution_part {
                if d > self.current_hop_distance {
                    // Then we made it through all searches and begin the next round
                    self.current_hop_distance += I::one();
                }
            }

            // decide where to put head_search now
            self.advance_head_search();

            solution_part
        }
    }
}

#[cfg(test)]
mod test {
    use crate::algorithms::graphs::shortest_distances::apsd::tests::{
        check_enumeration_result, directed_sample, directed_sample_solution, undirected_sample,
        undirected_sample_solution,
    };

    use super::*;

    #[test]
    fn test_undirected_enumeration() {
        let graph = undirected_sample();
        let solution_parts: Vec<_> = enumerate::prepare_enumeration(&graph).collect();
        check_enumeration_result(&solution_parts, &undirected_sample_solution(), false, true);
    }

    #[test]
    fn test_directed_enumeration() {
        let graph = directed_sample();
        let solution_parts: Vec<_> = enumerate::prepare_enumeration(&graph).collect();
        check_enumeration_result(&solution_parts, &directed_sample_solution(), false, true);
    }
}
