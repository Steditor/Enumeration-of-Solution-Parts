use std::ops::ControlFlow;

use crate::data_structures::graphs::{DirectedGraph, Direction, Index};

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
pub enum DfsEvent<I: Index> {
    Discovered(I),
    Finished(I),
    BackEdge(I, I),
}

/// Incremental depth first search
pub struct IDFS<'a, I: Index> {
    colors: Vec<Color>,
    dfs_visit_stack: Vec<(I, Box<dyn Iterator<Item = I> + 'a>)>,
    dfs_loop: Box<dyn Iterator<Item = I>>,
}

impl<'a, I: Index> IDFS<'a, I> {
    pub fn new(num_vertices: I) -> Self {
        Self {
            colors: vec![Color::White; num_vertices.index()],
            dfs_visit_stack: Vec::new(),
            dfs_loop: Box::new(I::new(0).range(num_vertices)),
        }
    }

    pub fn next<G: DirectedGraph<I>>(&mut self, graph: &'a G) -> Option<DfsEvent<I>> {
        // no current DFS-Visit stack to work through?
        if self.dfs_visit_stack.is_empty() {
            // find the next start vertex for dfs-visit
            for u in self.dfs_loop.by_ref() {
                if self.colors[u.index()] == Color::White {
                    self.dfs_visit_stack
                        .push((u, graph.neighbors(u, Direction::OUT)));
                    break;
                }
            }
        }

        // nothing left to do?
        if self.dfs_visit_stack.is_empty() {
            return None; // we've visited all vertices
        }

        let (u, neighbors) = self
            .dfs_visit_stack
            .last_mut()
            .expect("stack can't be empty");
        let u = *u;

        // gray vertex on top? Try to find next white child
        // (resume dfs-visit)
        if self.colors[u.index()] == Color::Gray {
            for v in neighbors.by_ref() {
                match self.colors[v.index()] {
                    Color::White => {
                        // tree edge
                        self.dfs_visit_stack
                            .push((v, graph.neighbors(v, Direction::OUT)));
                        break;
                    }
                    Color::Gray => {
                        // back edge
                        return Some(DfsEvent::BackEdge(u, v));
                    }
                    Color::Black => (), // ignore forward and cross edges
                }
            }
        }
        let u = self.dfs_visit_stack.last().expect("stack can't be empty").0;

        match self.colors[u.index()] {
            Color::White => {
                // a white vertex neighbor u has just been discovered
                // (start dfs-visit)
                self.colors[u.index()] = Color::Gray;
                Some(DfsEvent::Discovered(u))
            }
            Color::Gray => {
                // still the same gray vertex as before; has no undiscovered neighbors
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
pub fn dfs<I: Index, G: DirectedGraph<I>, B>(
    graph: &G,
    visitor: &mut impl FnMut(DfsEvent<I>) -> ControlFlow<B>,
) -> ControlFlow<B> {
    let mut colors = vec![Color::White; graph.num_vertices().index()];

    for u in I::new(0).range(graph.num_vertices()) {
        if colors[u.index()] == Color::White {
            dfs_visit(graph, u, visitor, &mut colors)?;
        }
    }
    ControlFlow::Continue(())
}

fn dfs_visit<I: Index, G: DirectedGraph<I>, B>(
    graph: &G,
    u: I,
    visitor: &mut impl FnMut(DfsEvent<I>) -> ControlFlow<B>,
    colors: &mut Vec<Color>,
) -> ControlFlow<B> {
    colors[u.index()] = Color::Gray;
    visitor(DfsEvent::Discovered(u))?;

    for v in graph.neighbors(u, Direction::OUT) {
        match colors[v.index()] {
            Color::White => {
                // tree edge
                dfs_visit(graph, v, visitor, colors)?;
            }
            Color::Gray => {
                // back edge
                visitor(DfsEvent::BackEdge(u, v))?;
            }
            Color::Black => {
                // ignore forward and cross edges
            }
        }
    }
    colors[u.index()] = Color::Black;
    visitor(DfsEvent::Finished(u))
}

#[cfg(test)]
mod test {
    use crate::data_structures::graphs::{DirectedAdjacencyArraysGraph, DirectedEdgeListGraph};

    use super::{DfsEvent::BackEdge, DfsEvent::Discovered, DfsEvent::Finished, *};

    /// DFS example in Figure 20.4 of CRLS 4th edition
    const CRLS_20_4_EDGES: [(u32, u32); 8] = [
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
    fn test_enumeration_crls_20_4() {
        let graph = DirectedEdgeListGraph::new(6, Box::new(CRLS_20_4_EDGES));
        let graph = DirectedAdjacencyArraysGraph::from(&graph);
        let mut dfs = IDFS::new(graph.num_vertices());

        let mut events = Vec::new();
        while let Some(e) = dfs.next(&graph) {
            events.push(e);
        }

        assert_eq!(
            events,
            [
                Discovered(0),
                Discovered(1),
                Discovered(4),
                Discovered(3),
                BackEdge(3, 1),
                Finished(3),
                Finished(4),
                Finished(1),
                Finished(0),
                Discovered(2),
                Discovered(5),
                BackEdge(5, 5),
                Finished(5),
                Finished(2),
            ]
        );
    }

    #[test]
    fn test_recursive_crls_20_4() {
        let graph = DirectedEdgeListGraph::new(6, Box::new(CRLS_20_4_EDGES));
        let graph = DirectedAdjacencyArraysGraph::from(&graph);

        let mut events = Vec::new();

        dfs(&graph, &mut |e| {
            events.push(e);
            ControlFlow::<()>::Continue(())
        });

        assert_eq!(
            events,
            [
                Discovered(0),
                Discovered(1),
                Discovered(4),
                Discovered(3),
                BackEdge(3, 1),
                Finished(3),
                Finished(4),
                Finished(1),
                Finished(0),
                Discovered(2),
                Discovered(5),
                BackEdge(5, 5),
                Finished(5),
                Finished(2),
            ]
        );
    }
}
