use std::{collections::VecDeque, marker::PhantomData, ops::ControlFlow};

use crate::data_structures::{
    graphs::{Direction, EdgeData, Graph, UndirectedGraph},
    Index,
};

/// Discovery state of vertices as presented in CRLS: Introduction to Algorithms
#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
    /// undiscovered
    White,
    /// discovered, not finished
    Gray,
    /// finished
    Black,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BfsEvent<I: Index> {
    Discovered(I),
    Finished(I),
    TreeEdge(I, I),
}

pub struct IBFS<'a, G, I, ED>
where
    G: Graph<I, ED> + ?Sized,
    I: Index,
    ED: EdgeData,
{
    colors: Vec<Color>,
    last_discovered: Option<I>,
    bfs_visit_queue: VecDeque<(I, Box<dyn Iterator<Item = I> + 'a>)>,
    _phantom: PhantomData<(ED, G)>,
}

impl<'a, G, I, ED> IBFS<'a, G, I, ED>
where
    G: Graph<I, ED> + ?Sized,
    I: Index,
    ED: EdgeData,
{
    pub fn new(graph: &G, source: I) -> Self {
        Self {
            colors: vec![Color::White; graph.num_vertices().index()],
            last_discovered: Some(source),
            bfs_visit_queue: VecDeque::new(),
            _phantom: PhantomData,
        }
    }

    pub fn next(&mut self, graph: &'a G) -> Option<BfsEvent<I>> {
        // is there a half-processed discovery?
        if let Some(v) = self.last_discovered.take() {
            // this is the source or we've already sent out the TreeEdge event
            self.colors[v.index()] = Color::Gray;
            self.bfs_visit_queue
                .push_back((v, graph.neighbors(v, Direction::OUT)));
            return Some(BfsEvent::Discovered(v));
        }

        // nothing left to do?
        if self.bfs_visit_queue.is_empty() {
            return None; // we've visited all vertices reachable from source
        }

        let (u, neighbors) = self
            .bfs_visit_queue
            .front_mut()
            .expect("queue can't be empty");
        let u = *u;

        // resume the iterator over all neighbors of u
        for v in neighbors.by_ref() {
            if self.colors[v.index()] == Color::White {
                // we've found a tree edge!
                // we only do half the work here: TreeEdge now, Discovery in the next call
                self.last_discovered = Some(v);
                return Some(BfsEvent::TreeEdge(u, v));
            }
        }

        // still here? => done with all neighbors
        self.colors[u.index()] = Color::Black;
        self.bfs_visit_queue.pop_front();
        Some(BfsEvent::Finished(u))
    }
}

pub fn bfs<G, I, ED, B>(
    graph: &G,
    source: I,
    visitor: &mut impl FnMut(BfsEvent<I>) -> ControlFlow<B>,
) -> ControlFlow<B>
where
    G: Graph<I, ED> + ?Sized,
    I: Index,
    ED: EdgeData,
{
    let mut colors = vec![Color::White; graph.num_vertices().index()];
    let mut q = VecDeque::new();

    colors[source.index()] = Color::Gray;
    visitor(BfsEvent::Discovered(source))?;
    q.push_back(source);

    while !q.is_empty() {
        let u = q.pop_front().expect("queue cannot be empty");
        for v in graph.neighbors(u, Direction::OUT) {
            if colors[v.index()] == Color::White {
                visitor(BfsEvent::TreeEdge(u, v))?;
                q.push_back(v);
                colors[v.index()] = Color::Gray;
                visitor(BfsEvent::Discovered(v))?;
            }
        }
        colors[u.index()] = Color::Black;
        visitor(BfsEvent::Finished(u))?;
    }

    ControlFlow::Continue(())
}

pub fn is_connected<I: Index, ED: EdgeData>(graph: &impl UndirectedGraph<I, ED>) -> bool {
    let mut num_discovered = I::zero();
    bfs(graph, I::zero(), &mut |e| {
        if let BfsEvent::Discovered(_) = e {
            num_discovered += I::one();
        }
        ControlFlow::<()>::Continue(())
    });
    num_discovered == graph.num_vertices()
}

#[cfg(test)]
mod test {
    use crate::data_structures::graphs::UndirectedAdjacencyArrayGraph;

    use super::{BfsEvent::Discovered, BfsEvent::Finished, BfsEvent::TreeEdge, *};

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

    #[test]
    fn test_enumeration_crls_22_3() {
        let graph = UndirectedAdjacencyArrayGraph::<u32>::new(8, &CRLS_22_3_EDGES);
        let mut bfs = IBFS::new(&graph, 1);

        let mut events = Vec::new();
        while let Some(e) = bfs.next(&graph) {
            events.push(e);
        }

        assert_eq!(
            events,
            [
                Discovered(1),
                TreeEdge(1, 5),
                Discovered(5),
                TreeEdge(1, 0),
                Discovered(0),
                Finished(1),
                TreeEdge(5, 2),
                Discovered(2),
                TreeEdge(5, 6),
                Discovered(6),
                Finished(5),
                TreeEdge(0, 4),
                Discovered(4),
                Finished(0),
                TreeEdge(2, 3),
                Discovered(3),
                Finished(2),
                TreeEdge(6, 7),
                Discovered(7),
                Finished(6),
                Finished(4),
                Finished(3),
                Finished(7),
            ]
        );
    }

    #[test]
    fn test_loop_crls_22_3() {
        let graph = UndirectedAdjacencyArrayGraph::<u32>::new(8, &CRLS_22_3_EDGES);

        let mut events = Vec::new();

        bfs(&graph, 1, &mut |e| {
            events.push(e);
            ControlFlow::<()>::Continue(())
        });

        assert_eq!(
            events,
            [
                Discovered(1),
                TreeEdge(1, 5),
                Discovered(5),
                TreeEdge(1, 0),
                Discovered(0),
                Finished(1),
                TreeEdge(5, 2),
                Discovered(2),
                TreeEdge(5, 6),
                Discovered(6),
                Finished(5),
                TreeEdge(0, 4),
                Discovered(4),
                Finished(0),
                TreeEdge(2, 3),
                Discovered(3),
                Finished(2),
                TreeEdge(6, 7),
                Discovered(7),
                Finished(6),
                Finished(4),
                Finished(3),
                Finished(7),
            ]
        );
    }
}
