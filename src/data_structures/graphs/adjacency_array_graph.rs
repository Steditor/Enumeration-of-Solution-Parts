use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::{
    Adjacency, DirectedEdgeListGraph, DirectedGraph, Direction, Edge, EdgeData, Graph, Index,
    UndirectedGraph,
};

/// A directed graph stored as array of adjacencies.
///
/// All out-adjacencies are stored in a single array, sorted by the source vertex.
/// For each source vertex we store the offset of the first adjacency in the combined array.
/// The end is derived by the offset of the next vertex or the end of the adjacency array.
///
/// Edges can have edge data of type `ED` (e.g. edge weights) attached.
///
/// The data structure is essentially identical to the "standard representation" presented in \[1\] and implemented similar to [the graph_builder crate](https://docs.rs/graph_builder/latest/src/graph_builder/index.rs.html).
/// \[1\] F. Kammer and A. Sajenko, “Linear-Time In-Place DFS and BFS on the Word RAM,” in Algorithms and Complexity, P. Heggernes, Ed., in Lecture Notes in Computer Science. Cham: Springer International Publishing, 2019, pp. 286–298. doi: [10.1007/978-3-030-17402-6_24](https://doi.org/10.1007/978-3-030-17402-6_24).
#[derive(Serialize, Deserialize, Debug)]
pub struct DirectedAdjacencyArrayGraph<I: Index, ED: EdgeData = ()> {
    offsets: Box<[I]>,
    adjacencies: Box<[(I, ED)]>,
}

impl<I: Index, ED: EdgeData> Display for DirectedAdjacencyArrayGraph<I, ED> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "DirectedAdjacencyArrayGraph(n={}, m={}) [",
            self.num_vertices(),
            self.num_edges()
        )?;
        for v in self.vertices() {
            write!(f, "\t{} →", v)?;
            for a in self.adjacencies(v, Direction::OUT) {
                write!(f, " {:?},", a)?;
            }
            writeln!(f)?;
        }
        write!(f, "]")
    }
}

impl<I: Index, ED: EdgeData> Graph<I, ED> for DirectedAdjacencyArrayGraph<I, ED> {
    fn num_vertices(&self) -> I {
        I::new(self.offsets.len())
    }

    fn num_edges(&self) -> I {
        I::new(self.adjacencies.len())
    }

    fn edges(&self) -> Box<dyn Iterator<Item = (I, I, ED)> + '_> {
        Box::new(EdgeIterator::new(self))
    }

    fn degree(&self, v: I, dir: Direction) -> I {
        match dir {
            Direction::OUT => {
                let (start_inclusive, end_exclusive) = self.bounds(v);
                end_exclusive - start_inclusive
            }
            Direction::IN => I::new(self.adjacencies.iter().filter(|a| a.sink() == v).count()),
        }
    }

    fn degrees(&self, dir: Direction) -> Box<[I]> {
        match dir {
            Direction::OUT => self
                .vertices()
                .map(|v| self.degree(v, Direction::OUT))
                .collect(),
            Direction::IN => {
                let mut degrees = vec![I::zero(); self.num_vertices().index()].into_boxed_slice();
                for adjacency in self.adjacencies.iter() {
                    degrees[adjacency.sink().index()] += I::one();
                }
                degrees
            }
        }
    }

    fn neighbors(&self, v: I, dir: Direction) -> Box<dyn Iterator<Item = I> + '_> {
        Box::new(self.adjacencies(v, dir).map(|a| a.sink()))
    }

    fn adjacencies(&self, v: I, dir: Direction) -> Box<dyn Iterator<Item = (I, ED)> + '_> {
        match dir {
            Direction::OUT => self.out_adjacencies(v),
            Direction::IN => self.in_adjacencies(v),
        }
    }

    fn new_with_edge_data(num_vertices: I, edges: &[(I, I, ED)]) -> Self {
        Self::from(&DirectedEdgeListGraph::new_with_edge_data(
            num_vertices,
            edges,
        ))
    }

    fn new(num_vertices: I, edges: &[(I, I)]) -> Self {
        Self::from(&DirectedEdgeListGraph::new(num_vertices, edges))
    }
}

impl<I: Index, ED: EdgeData> DirectedAdjacencyArrayGraph<I, ED> {
    #[inline]
    fn bounds(&self, v: I) -> (I, I) {
        let start_inclusive = self.offsets[v.index()];
        let end_exclusive = match self.offsets.get(v.index() + 1) {
            Some(x) => *x,
            None => self.num_edges(),
        };

        (start_inclusive, end_exclusive)
    }

    fn out_adjacencies(&self, v: I) -> Box<dyn Iterator<Item = (I, ED)> + '_> {
        let (start_inclusive, end_exclusive) = self.bounds(v);
        Box::new(
            self.adjacencies[start_inclusive.index()..end_exclusive.index()]
                .iter()
                .copied(),
        )
    }

    fn in_adjacencies(&self, v: I) -> Box<dyn Iterator<Item = (I, ED)> + '_> {
        Box::new(
            EdgeIterator::new(self)
                .filter(move |e| e.sink() == v)
                .map(|e| (e.source(), e.data())),
        )
    }

    pub fn from_edges(el_graph: &DirectedEdgeListGraph<I, ED>, dir: Direction) -> Self {
        // compute offsets
        let degrees = el_graph.degrees(dir);
        let mut offsets = degrees_to_offsets(degrees);
        // collect edges
        let mut adjacencies: Box<[(I, ED)]> =
            vec![Default::default(); el_graph.num_edges().index()].into_boxed_slice();
        for edge in el_graph.edges() {
            let vertex = dir.vertex(&edge);
            let other = dir.other(&edge);

            adjacencies[offsets[vertex.index()].index()] = (other, edge.data());
            offsets[vertex.index()] += I::one();
        }

        // reset offsets
        offsets.rotate_right(1);
        offsets[0] = I::zero();

        DirectedAdjacencyArrayGraph {
            offsets,
            adjacencies,
        }
    }
}

fn degrees_to_offsets<I: Index>(mut degrees: Box<[I]>) -> Box<[I]> {
    let mut current_offset: I = I::zero();
    for entry in degrees.iter_mut() {
        let degree = *entry;
        *entry = current_offset;
        current_offset += degree;
    }
    degrees
}

impl<I: Index, ED: EdgeData> DirectedGraph<I, ED> for DirectedAdjacencyArrayGraph<I, ED> {}

impl<I: Index, ED: EdgeData> From<&DirectedEdgeListGraph<I, ED>>
    for DirectedAdjacencyArrayGraph<I, ED>
{
    fn from(el_graph: &DirectedEdgeListGraph<I, ED>) -> Self {
        DirectedAdjacencyArrayGraph::from_edges(el_graph, Direction::OUT)
    }
}

/// Lending iterator for edges of an adjacency array
struct EdgeIterator<'a, I: Index, ED: EdgeData> {
    graph: &'a DirectedAdjacencyArrayGraph<I, ED>,
    /// current vertex
    vertex: I,
    /// next offset
    offset: I,
}

impl<'a, I: Index, ED: EdgeData> EdgeIterator<'a, I, ED> {
    pub fn new(graph: &'a DirectedAdjacencyArrayGraph<I, ED>) -> Self {
        Self {
            graph,
            vertex: I::zero(),
            offset: I::zero(),
        }
    }
}

impl<I: Index, ED: EdgeData> Iterator for EdgeIterator<'_, I, ED> {
    type Item = (I, I, ED);

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.graph.num_edges() {
            return None;
        }

        // determine edge
        let a = &self.graph.adjacencies[self.offset.index()];
        while self.offset >= self.graph.bounds(self.vertex).1 {
            self.vertex += I::one();
        }

        // advance offset for next round
        self.offset += I::one();

        Some((self.vertex, a.sink(), a.data()))
    }
}

/// An undirected graph stored as array of adjacencies.
///
/// Internally this uses [DirectedAdjacencyArrayGraph] and stores each undirected edge
/// once per direction. Most methods delegate to this internal representation,
/// but the [Graph::num_edges] method is reimplemented to count each edge
/// only once.
#[derive(Serialize, Deserialize, Debug)]
pub struct UndirectedAdjacencyArrayGraph<I: Index, ED: EdgeData = ()> {
    graph: DirectedAdjacencyArrayGraph<I, ED>,
}

impl<I: Index, ED: EdgeData> Display for UndirectedAdjacencyArrayGraph<I, ED> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "UndirectedAdjacencyArrayGraph(n={}, m={}) [",
            self.num_vertices(),
            self.num_edges()
        )?;
        for v in self.vertices() {
            write!(f, "\t{} →", v)?;
            for a in self.adjacencies(v, Direction::OUT) {
                write!(f, " {:?},", a)?;
            }
            writeln!(f)?;
        }
        write!(f, "]")
    }
}

impl<I: Index, ED: EdgeData> Graph<I, ED> for UndirectedAdjacencyArrayGraph<I, ED> {
    fn num_vertices(&self) -> I {
        self.graph.num_vertices()
    }

    fn num_edges(&self) -> I {
        // edges between different vertices are stored twice
        I::new(
            self.graph
                .edges()
                .filter(|e| e.source() <= e.sink())
                .count(),
        )
    }

    fn edges(&self) -> Box<dyn Iterator<Item = (I, I, ED)> + '_> {
        self.graph.edges()
    }

    fn degree(&self, v: I, _: Direction) -> I {
        self.graph.degree(v, Direction::OUT) // graph is undirected
    }

    fn degrees(&self, _: Direction) -> Box<[I]> {
        self.graph.degrees(Direction::OUT) // graph is undirected
    }

    fn neighbors(&self, v: I, _: Direction) -> Box<dyn Iterator<Item = I> + '_> {
        self.graph.neighbors(v, Direction::OUT) // graph is undirected
    }

    fn adjacencies(&self, v: I, _: Direction) -> Box<dyn Iterator<Item = (I, ED)> + '_> {
        self.graph.adjacencies(v, Direction::OUT) // graph is undirected
    }

    fn new_with_edge_data(num_vertices: I, edges: &[(I, I, ED)]) -> Self {
        let transposed_edges = edges
            .iter()
            .filter(|e| e.source() != e.sink())
            .map(|e| (e.sink(), e.source(), e.data()));
        let all_edges: Vec<_> = edges.iter().copied().chain(transposed_edges).collect();
        Self {
            graph: DirectedAdjacencyArrayGraph::new_with_edge_data(num_vertices, &all_edges),
        }
    }

    fn new(num_vertices: I, edges: &[(I, I)]) -> Self {
        let transposed_edges = edges
            .iter()
            .filter(|e| e.source() != e.sink())
            .map(|e| (e.sink(), e.source()));
        let all_edges: Vec<_> = edges.iter().copied().chain(transposed_edges).collect();
        Self {
            graph: DirectedAdjacencyArrayGraph::new(num_vertices, &all_edges),
        }
    }
}

impl<I: Index, ED: EdgeData> UndirectedGraph<I, ED> for UndirectedAdjacencyArrayGraph<I, ED> {}

#[cfg(test)]
mod test {
    use crate::data_structures::graphs::tests::directed_weighted;

    use super::*;

    const OFFSETS: [u32; 6] = [0, 0, 0, 3, 5, 7];
    const TRANSPOSED_OFFSETS: [u32; 6] = [0, 0, 2, 2, 5, 7];

    const UNWEIGHTED_ADJACENCIES: [(u32, u8); 7] =
        [(1, 1), (4, 3), (3, 6), (4, 5), (3, 7), (1, 2), (3, 4)];
    const TRANSPOSED_UNWEIGHTED_ADJACENCIES: [(u32, u8); 7] =
        [(2, 1), (4, 2), (4, 4), (2, 6), (3, 7), (2, 3), (3, 5)];

    #[test]
    fn test_build() {
        let graph = DirectedAdjacencyArrayGraph::new_with_edge_data(6, &directed_weighted::edges());

        assert_eq!(graph.offsets, OFFSETS.into());
        assert_eq!(graph.adjacencies, UNWEIGHTED_ADJACENCIES.into());
    }

    #[test]
    fn test_build_transposed() {
        let graph = DirectedEdgeListGraph::new_with_edge_data(6, &directed_weighted::edges());
        let graph = DirectedAdjacencyArrayGraph::from_edges(&graph, Direction::IN);

        assert_eq!(graph.offsets, TRANSPOSED_OFFSETS.into());
        assert_eq!(graph.adjacencies, TRANSPOSED_UNWEIGHTED_ADJACENCIES.into());
    }
}
