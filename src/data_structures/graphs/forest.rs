use std::ops::{Deref, DerefMut};

use super::{Adjacency, Direction, Edge, EdgeData, Graph, Index};

/// An in-forest represented by parent links.
///
/// Every vertex `v` either stores `None` or a parent adjacency `(p, d)`,
/// where `p` is the parent vertex and `d` is some edge data.
///
/// Note that, as the data structure is able to encode graphs with cycles,
/// users have to make sure to only provide acyclic graphs.
#[derive(Debug)]
pub struct Forest<I: Index, ED: EdgeData = ()>(Box<[Option<(I, ED)>]>);

impl<I: Index, ED: EdgeData> Deref for Forest<I, ED> {
    type Target = Box<[Option<(I, ED)>]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Index, ED: EdgeData> DerefMut for Forest<I, ED> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<I: Index, ED: EdgeData> Graph<I, ED> for Forest<I, ED> {
    fn num_vertices(&self) -> I {
        I::new(self.len())
    }

    fn num_edges(&self) -> I {
        I::new(self.iter().flatten().count())
    }

    fn edges(&self) -> Box<dyn Iterator<Item = (I, I, ED)> + '_> {
        Box::new(
            self.iter()
                .enumerate()
                .filter_map(|(v, a)| a.map(|a| (v, a)))
                .map(|(v, a)| (I::new(v), a.sink(), a.data())),
        )
    }

    fn adjacencies(&self, v: I, dir: Direction) -> Box<dyn Iterator<Item = (I, ED)> + '_> {
        match dir {
            Direction::OUT => Box::new(self[v.index()..v.index() + 1].iter().flatten().copied()),
            Direction::IN => Box::new(
                self.edges()
                    .filter(move |e| e.sink() == v)
                    .map(|e| (e.source(), e.data())),
            ),
        }
    }

    fn neighbors(&self, v: I, dir: Direction) -> Box<dyn Iterator<Item = I> + '_> {
        Box::new(self.adjacencies(v, dir).map(|a| a.sink()))
    }

    fn degree(&self, v: I, dir: Direction) -> I {
        match dir {
            Direction::OUT => {
                if self[v.index()].is_some() {
                    I::one()
                } else {
                    I::zero()
                }
            }
            Direction::IN => I::new(self.adjacencies(v, Direction::IN).count()),
        }
    }

    /// Create a new forest with the parent-links given as list of edges.
    ///
    /// Note that the parent-links have to be a consistent in-tree,
    /// otherwise the resulting forest might only capture some of the edges.
    fn new_with_edge_data(num_vertices: I, edges: &[(I, I, ED)]) -> Self
    where
        Self: Sized,
    {
        let mut forest = Self::new_isolated_vertices(num_vertices);

        for e in edges {
            forest[e.source().index()] = Some((e.sink(), e.data()));
        }

        forest
    }

    /// Create a new forest with the parent-links given as list of edges.
    ///
    /// Note that the parent-links have to be a consistent in-tree,
    /// otherwise the resulting forest might only capture some of the edges.
    ///
    /// Edge data is set to the default for `ED` for all edges.
    fn new(num_vertices: I, edges: &[(I, I)]) -> Self
    where
        Self: Sized,
    {
        let mut forest = Self::new_isolated_vertices(num_vertices);

        for e in edges {
            forest[e.source().index()] = Some((e.sink(), ED::default()));
        }

        forest
    }
}

impl<I: Index, ED: EdgeData> Forest<I, ED> {
    pub fn new_isolated_vertices(num_vertices: I) -> Self {
        Self(vec![Option::None; num_vertices.index()].into())
    }
}

#[cfg(test)]
mod test {
    use crate::helpers::assert_same_elements;

    use super::*;

    /// The MST edges of CLRS 3rd edition Figure 23.4
    fn edges() -> Box<[(u32, u32, u8)]> {
        Box::new([
            (8, 2, 2),
            (5, 2, 4),
            (3, 2, 7),
            (4, 3, 9),
            (6, 5, 2),
            (7, 6, 1),
            (0, 7, 8),
            (1, 0, 4),
        ])
    }

    fn build() -> Forest<u32, u8> {
        Forest::new_with_edge_data(9, &edges())
    }

    #[test]
    pub fn test_vertices() {
        let graph = build();
        assert_eq!(graph.num_vertices(), 9);
        assert_same_elements(graph.vertices(), [0, 1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    pub fn test_edges() {
        let graph = build();
        assert_eq!(graph.num_edges(), 8);
        assert_same_elements(graph.edges(), edges().iter().copied());
    }

    #[test]
    pub fn test_out_degree() {
        let graph = build();
        assert_eq!(graph.degree(0, Direction::OUT), 1);
        assert_eq!(graph.degree(1, Direction::OUT), 1);
        assert_eq!(graph.degree(2, Direction::OUT), 0);
        assert_eq!(graph.degree(3, Direction::OUT), 1);
        assert_eq!(graph.degree(4, Direction::OUT), 1);
        assert_eq!(graph.degree(5, Direction::OUT), 1);
        assert_eq!(graph.degree(6, Direction::OUT), 1);
        assert_eq!(graph.degree(7, Direction::OUT), 1);
        assert_eq!(graph.degree(8, Direction::OUT), 1);
    }

    #[test]
    pub fn test_out_neighbors() {
        let graph = build();
        assert_same_elements(graph.neighbors(0, Direction::OUT), [7]);
        assert_same_elements(graph.neighbors(1, Direction::OUT), [0]);
        assert_same_elements(graph.neighbors(2, Direction::OUT), []);
        assert_same_elements(graph.neighbors(3, Direction::OUT), [2]);
        assert_same_elements(graph.neighbors(4, Direction::OUT), [3]);
        assert_same_elements(graph.neighbors(5, Direction::OUT), [2]);
        assert_same_elements(graph.neighbors(6, Direction::OUT), [5]);
        assert_same_elements(graph.neighbors(7, Direction::OUT), [6]);
        assert_same_elements(graph.neighbors(8, Direction::OUT), [2]);
    }

    #[test]
    pub fn test_out_adjacencies() {
        let graph = build();
        assert_same_elements(graph.adjacencies(0, Direction::OUT), [(7, 8)]);
        assert_same_elements(graph.adjacencies(1, Direction::OUT), [(0, 4)]);
        assert_same_elements(graph.adjacencies(2, Direction::OUT), []);
        assert_same_elements(graph.adjacencies(3, Direction::OUT), [(2, 7)]);
        assert_same_elements(graph.adjacencies(4, Direction::OUT), [(3, 9)]);
        assert_same_elements(graph.adjacencies(5, Direction::OUT), [(2, 4)]);
        assert_same_elements(graph.adjacencies(6, Direction::OUT), [(5, 2)]);
        assert_same_elements(graph.adjacencies(7, Direction::OUT), [(6, 1)]);
        assert_same_elements(graph.adjacencies(8, Direction::OUT), [(2, 2)]);
    }

    #[test]
    pub fn test_in_degree() {
        let graph = build();
        assert_eq!(graph.degree(0, Direction::IN), 1);
        assert_eq!(graph.degree(1, Direction::IN), 0);
        assert_eq!(graph.degree(2, Direction::IN), 3);
        assert_eq!(graph.degree(3, Direction::IN), 1);
        assert_eq!(graph.degree(4, Direction::IN), 0);
        assert_eq!(graph.degree(5, Direction::IN), 1);
        assert_eq!(graph.degree(6, Direction::IN), 1);
        assert_eq!(graph.degree(7, Direction::IN), 1);
        assert_eq!(graph.degree(8, Direction::IN), 0);
    }

    #[test]
    pub fn test_in_neighbors() {
        let graph = build();
        assert_same_elements(graph.neighbors(0, Direction::IN), [1]);
        assert_same_elements(graph.neighbors(1, Direction::IN), []);
        assert_same_elements(graph.neighbors(2, Direction::IN), [3, 5, 8]);
        assert_same_elements(graph.neighbors(3, Direction::IN), [4]);
        assert_same_elements(graph.neighbors(4, Direction::IN), []);
        assert_same_elements(graph.neighbors(5, Direction::IN), [6]);
        assert_same_elements(graph.neighbors(6, Direction::IN), [7]);
        assert_same_elements(graph.neighbors(7, Direction::IN), [0]);
        assert_same_elements(graph.neighbors(8, Direction::IN), []);
    }

    #[test]
    pub fn test_in_adjacencies() {
        let graph = build();
        assert_same_elements(graph.adjacencies(0, Direction::IN), [(1, 4)]);
        assert_same_elements(graph.adjacencies(1, Direction::IN), []);
        assert_same_elements(
            graph.adjacencies(2, Direction::IN),
            [(3, 7), (5, 4), (8, 2)],
        );
        assert_same_elements(graph.adjacencies(3, Direction::IN), [(4, 9)]);
        assert_same_elements(graph.adjacencies(4, Direction::IN), []);
        assert_same_elements(graph.adjacencies(5, Direction::IN), [(6, 2)]);
        assert_same_elements(graph.adjacencies(6, Direction::IN), [(7, 1)]);
        assert_same_elements(graph.adjacencies(7, Direction::IN), [(0, 8)]);
        assert_same_elements(graph.adjacencies(8, Direction::IN), []);
    }
}
