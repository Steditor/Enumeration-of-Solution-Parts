use std::marker::PhantomData;

use binary_heap_plus::BinaryHeap;
use compare::{Compare, Extract, Rev};

use crate::{
    data_structures::{
        graphs::{Adjacency, Direction, Edge, EdgeData, Forest, UndirectedGraph},
        Index,
    },
    experiments::{ExperimentAlgorithm, PreparedEnumerationAlgorithm},
};

use super::{AlgorithmType, MstAlgorithm, MstPartial};

pub const PRIM: AlgorithmType =
    ExperimentAlgorithm::TotalTimeAlgorithm("total-prim", |graph| Ok(Prim::mst_for(graph)));

pub const INCREMENTAL_PRIM: AlgorithmType =
    ExperimentAlgorithm::EnumerationAlgorithm("incremental-prim", IncrementalPrim::enumerator_for);

/// Discovery state of vertices
#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
    /// undiscovered
    White,
    /// discovered, a possible crossing edge is in the queue
    Gray,
    /// finished, attached to the growing tree
    Black,
}

/// Prim's MST algorithm
///
/// The implementation uses [binary_heap_plus] as priority queue.
pub struct Prim {}

impl<I: Index, ED: EdgeData> MstAlgorithm<I, ED> for Prim {
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
        let mut tree: Forest<I, ED> = Forest::new_isolated_vertices(graph.num_vertices());
        let mut colors = vec![Color::White; graph.num_vertices().index()];

        // BinaryHeap is a max heap, we want a min-heap that sorts by the given comparator.
        // As we need to borrow the comparator below, we do not use [Compare::rev] here.
        // We don't have a decrease-key operation and add target vertices multiple times instead.
        let mut target_queue =
            BinaryHeap::with_capacity_by(graph.num_vertices().index(), |e1, e2| {
                comparator.compare(e2, e1)
            });

        // start at vertex 0 with fictitious self-edge
        let start = I::zero();
        target_queue.push((start, start, ED::default()));
        colors[start.index()] = Color::Gray;

        while let Some(e) = target_queue.pop() {
            let u = e.sink();
            if colors[u.index()] == Color::Black {
                // this vertex is already attached
                // the entry in the priority queue was 'deprecated' by a later 'decrease-key'
                continue;
            }
            colors[u.index()] = Color::Black;

            for a in graph.adjacencies(u, Direction::OUT) {
                let v = a.sink();
                let w = a.data();

                // crossing edge to new vertex or preferred to previously known one?
                if colors[v.index()] == Color::White
                    || (colors[v.index()] == Color::Gray
                        && tree[v.index()].is_some_and(|(old_u, old_w)| {
                            comparator.compares_lt(&(u, v, w), &(old_u, v, old_w))
                        }))
                {
                    tree[v.index()] = Some((u, w));
                    target_queue.push((u, v, w));
                    colors[v.index()] = Color::Gray;
                }
            }
        }

        tree
    }
}

pub struct IncrementalPrim<I: Index, ED: EdgeData> {
    _phantom: PhantomData<(I, ED)>,
}

impl<I: Index, ED: EdgeData> IncrementalPrim<I, ED> {
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

struct MstEnumerator<'a, G, I, ED, C>
where
    G: UndirectedGraph<I, ED>,
    I: Index,
    ED: EdgeData,
    C: Compare<(I, I, ED)>,
{
    graph: &'a G,
    colors: Vec<Color>,
    tree: Forest<I, ED>,
    comparator: C,
    target_queue: BinaryHeap<(I, I, ED), Rev<C>>,
}

impl<'a, G, I, ED, C> MstEnumerator<'a, G, I, ED, C>
where
    G: UndirectedGraph<I, ED>,
    I: Index,
    ED: EdgeData,
    C: Compare<(I, I, ED)> + Copy,
{
    pub fn with_comparator(graph: &'a G, comparator: C) -> Self {
        let tree: Forest<I, ED> = Forest::new_isolated_vertices(graph.num_vertices());
        let mut colors = vec![Color::White; graph.num_vertices().index()];

        // BinaryHeap is a max heap, we want a min-heap that sorts by the given comparator.
        // As we need to borrow the comparator below, we do not use [Compare::rev] here.
        // We don't have a decrease-key operation and add target vertices multiple times instead.
        let mut target_queue = BinaryHeap::from_vec_cmp(
            Vec::with_capacity(graph.num_vertices().index()),
            comparator.rev(),
        );

        // start at vertex 0 with fictitious self-edge
        let start = I::zero();
        target_queue.push((start, start, ED::default()));
        colors[start.index()] = Color::Gray;

        Self {
            graph,
            colors,
            tree,
            comparator,
            target_queue,
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
        while let Some(e) = self.target_queue.pop() {
            let u = e.sink();
            if self.colors[u.index()] == Color::Black {
                // this vertex is already attached
                // the entry in the priority queue was 'deprecated' by a later 'decrease-key'
                continue;
            }
            self.colors[u.index()] = Color::Black;

            for a in self.graph.adjacencies(u, Direction::OUT) {
                let v = a.sink();
                let w = a.data();

                // crossing edge to new vertex or preferred to previously known one?
                if self.colors[v.index()] == Color::White
                    || (self.colors[v.index()] == Color::Gray
                        && self.tree[v.index()].is_some_and(|(old_u, old_w)| {
                            self.comparator.compares_lt(&(u, v, w), &(old_u, v, old_w))
                        }))
                {
                    self.tree[v.index()] = Some((u, w));
                    self.target_queue.push((u, v, w));
                    self.colors[v.index()] = Color::Gray;
                }
            }

            if e.sink() != e.source() {
                return Some((e.sink(), e.source(), e.data()));
            }
        }

        None
    }
}
