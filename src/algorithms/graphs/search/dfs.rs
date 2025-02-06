use std::{marker::PhantomData, ops::ControlFlow};

use crate::data_structures::{
    graphs::{Adjacency, Direction, EdgeData, Forest, Graph},
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
pub enum DfsEvent<I: Index, ED: EdgeData> {
    Discovered(I),
    Finished(I),
    BackEdge(I, I, ED),
    TreeEdge(I, I, ED),
}

/// A vertex with a (partially consumed) iterator over its adjacencies
type DfsVisitAdjacencyIterator<'a, I, ED> = (I, Box<dyn Iterator<Item = (I, ED)> + 'a>);

/// Incremental depth first search
pub struct IDFS<'a, I, ED>
where
    I: Index,
    ED: EdgeData,
{
    colors: Vec<Color>,
    dfs_visit_stack: Vec<DfsVisitAdjacencyIterator<'a, I, ED>>,
    dfs_loop: Box<dyn Iterator<Item = I>>,
    _phantom: PhantomData<ED>,
}

impl<'a, I: Index, ED: EdgeData> IDFS<'a, I, ED> {
    pub fn new(num_vertices: I) -> Self {
        Self {
            colors: vec![Color::White; num_vertices.index()],
            dfs_visit_stack: Vec::new(),
            dfs_loop: Box::new(I::zero().range(num_vertices)),
            _phantom: PhantomData,
        }
    }

    pub fn next(&mut self, graph: &'a impl Graph<I, ED>) -> Option<DfsEvent<I, ED>> {
        // no current DFS-Visit stack to work through?
        if self.dfs_visit_stack.is_empty() {
            // find the next start vertex for dfs-visit
            for u in self.dfs_loop.by_ref() {
                if self.colors[u.index()] == Color::White {
                    self.dfs_visit_stack
                        .push((u, graph.adjacencies(u, Direction::OUT)));
                    break;
                }
            }
        }

        // nothing left to do?
        if self.dfs_visit_stack.is_empty() {
            return None; // we've visited all vertices
        }

        let (u, adjacencies) = self
            .dfs_visit_stack
            .last_mut()
            .expect("stack can't be empty");
        let u = *u;

        match self.colors[u.index()] {
            Color::White => {
                // (start dfs-visit)
                self.colors[u.index()] = Color::Gray;
                Some(DfsEvent::Discovered(u))
            }
            Color::Gray => {
                // (resume dfs-visit)
                // resume the iterator over all adjacencies of u
                for a in adjacencies.by_ref() {
                    let v = a.sink();
                    match self.colors[v.index()] {
                        Color::White => {
                            // tree edge
                            self.dfs_visit_stack
                                .push((v, graph.adjacencies(v, Direction::OUT)));
                            return Some(DfsEvent::TreeEdge(u, v, a.data()));
                        }
                        Color::Gray => {
                            // back edge
                            return Some(DfsEvent::BackEdge(u, v, a.data()));
                        }
                        Color::Black => (), // ignore forward and cross edges
                    }
                }
                // still here? => done with all adjacencies
                // (end dfs-visit)
                self.colors[u.index()] = Color::Black;
                self.dfs_visit_stack.pop();
                Some(DfsEvent::Finished(u))
            }
            Color::Black => {
                panic!("There should never be a black (finished) vertex on the stack.")
            }
        }
    }
}

/// A recursive DFS implementation as presented in CRLS: Introduction to Algorithms
pub fn dfs<I: Index, ED: EdgeData, B>(
    graph: &impl Graph<I, ED>,
    visitor: &mut impl FnMut(DfsEvent<I, ED>) -> ControlFlow<B>,
) -> ControlFlow<B> {
    let mut colors = vec![Color::White; graph.num_vertices().index()];

    for u in graph.vertices() {
        if colors[u.index()] == Color::White {
            dfs_visit(graph, u, visitor, &mut colors)?;
        }
    }
    ControlFlow::Continue(())
}

fn dfs_visit<I: Index, ED: EdgeData, B>(
    graph: &impl Graph<I, ED>,
    u: I,
    visitor: &mut impl FnMut(DfsEvent<I, ED>) -> ControlFlow<B>,
    colors: &mut Vec<Color>,
) -> ControlFlow<B> {
    colors[u.index()] = Color::Gray;
    visitor(DfsEvent::Discovered(u))?;

    for a in graph.adjacencies(u, Direction::OUT) {
        let v = a.sink();
        match colors[v.index()] {
            Color::White => {
                // tree edge
                visitor(DfsEvent::TreeEdge(u, v, a.data()))?;
                dfs_visit(graph, v, visitor, colors)?;
            }
            Color::Gray => {
                // back edge
                visitor(DfsEvent::BackEdge(u, v, a.data()))?;
            }
            Color::Black => {
                // ignore forward and cross edges
            }
        }
    }
    colors[u.index()] = Color::Black;
    visitor(DfsEvent::Finished(u))
}

pub fn dfs_forest<I: Index, ED: EdgeData>(graph: &impl Graph<I, ED>) -> Forest<I, ED> {
    let mut forest = Forest::new_isolated_vertices(graph.num_vertices());

    dfs(graph, &mut |e| {
        if let DfsEvent::TreeEdge(u, v, d) = e {
            forest[v.index()] = Some((u, d));
        }
        ControlFlow::<()>::Continue(())
    });

    forest
}

#[cfg(test)]
mod test {
    use crate::{
        data_structures::graphs::DirectedAdjacencyArrayGraph, helpers::assert_same_elements,
    };

    use super::{
        DfsEvent::BackEdge, DfsEvent::Discovered, DfsEvent::Finished, DfsEvent::TreeEdge, *,
    };

    /// DFS example in Figure 22.4 of CRLS 3rd edition
    const CRLS_22_4_EDGES: [(u32, u32); 8] = [
        (0, 1),
        (0, 3),
        (1, 4),
        (2, 4),
        (2, 5),
        (3, 1),
        (4, 3),
        (5, 5),
    ];

    #[test]
    fn test_enumeration_crls_22_4() {
        let graph = DirectedAdjacencyArrayGraph::<u32>::new(6, &CRLS_22_4_EDGES);
        let mut dfs = IDFS::new(graph.num_vertices());

        let mut events = Vec::new();
        while let Some(e) = dfs.next(&graph) {
            events.push(e);
        }

        assert_eq!(
            events,
            [
                Discovered(0),
                TreeEdge(0, 1, ()),
                Discovered(1),
                TreeEdge(1, 4, ()),
                Discovered(4),
                TreeEdge(4, 3, ()),
                Discovered(3),
                BackEdge(3, 1, ()),
                Finished(3),
                Finished(4),
                Finished(1),
                Finished(0),
                Discovered(2),
                TreeEdge(2, 5, ()),
                Discovered(5),
                BackEdge(5, 5, ()),
                Finished(5),
                Finished(2),
            ]
        );
    }

    #[test]
    fn test_recursive_crls_22_4() {
        let graph = DirectedAdjacencyArrayGraph::<u32>::new(6, &CRLS_22_4_EDGES);

        let mut events = Vec::new();

        dfs(&graph, &mut |e| {
            events.push(e);
            ControlFlow::<()>::Continue(())
        });

        assert_eq!(
            events,
            [
                Discovered(0),
                TreeEdge(0, 1, ()),
                Discovered(1),
                TreeEdge(1, 4, ()),
                Discovered(4),
                TreeEdge(4, 3, ()),
                Discovered(3),
                BackEdge(3, 1, ()),
                Finished(3),
                Finished(4),
                Finished(1),
                Finished(0),
                Discovered(2),
                TreeEdge(2, 5, ()),
                Discovered(5),
                BackEdge(5, 5, ()),
                Finished(5),
                Finished(2),
            ]
        );
    }

    #[test]
    fn test_recursive_dfs_tree_22_4() {
        let graph: DirectedAdjacencyArrayGraph<u32> =
            DirectedAdjacencyArrayGraph::<u32>::new(6, &CRLS_22_4_EDGES);

        let tree = dfs_forest(&graph);

        assert_same_elements(
            tree.edges(),
            [(1, 0, ()), (4, 1, ()), (3, 4, ()), (5, 2, ())],
        );
    }
}
