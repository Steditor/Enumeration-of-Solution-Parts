use super::{DirectedGraph, Direction, Index};

/// A directed graph stored as number of vertices and list of edges.
#[derive(Clone, Debug)]
pub struct DirectedEdgeListGraph<I: Index> {
    num_vertices: I,
    edges: Box<[(I, I)]>,
}

impl<I: Index> DirectedGraph<I> for DirectedEdgeListGraph<I> {
    fn num_vertices(&self) -> I {
        self.num_vertices
    }

    fn num_edges(&self) -> I {
        I::new(self.edges.len())
    }

    fn degree(&self, v: I, dir: Direction) -> I {
        I::new(
            self.edges
                .iter()
                .filter(|edge| dir.vertex(edge) == v)
                .count(),
        )
    }

    fn neighbors(&self, v: I, dir: Direction) -> Box<dyn Iterator<Item = I> + '_> {
        Box::new(
            self.edges
                .iter()
                .filter(move |e| dir.vertex(e) == v)
                .map(|e| e.1),
        )
    }
}

impl<I: Index> DirectedEdgeListGraph<I> {
    pub fn new(num_vertices: I, edges: Box<[(I, I)]>) -> Self {
        Self {
            num_vertices,
            edges,
        }
    }

    pub fn degrees(&self, dir: Direction) -> Box<[I]> {
        let mut degrees = vec![I::new(0); self.num_vertices.index()].into_boxed_slice();
        for edge in self.edges.iter() {
            degrees[dir.vertex(edge).index()] += I::new(1);
        }
        degrees
    }

    pub fn edges(&self) -> &[(I, I)] {
        &self.edges
    }
}
