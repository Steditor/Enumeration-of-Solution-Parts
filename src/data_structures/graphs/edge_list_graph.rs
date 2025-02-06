use super::{
    edge::{Edge, EdgeData},
    DirectedGraph, Graph, Index, UndirectedGraph,
};

/// A directed graph stored as number of vertices and list of edges.
///
/// The vertices are implied to be named from (in terms of `I`) `0` to `num_vertices - 1` (inclusive),
/// and edges must not reference any other vertex name outside that range.
#[derive(Clone, Debug)]
pub struct DirectedEdgeListGraph<I: Index, ED: EdgeData = ()> {
    num_vertices: I,
    edges: Box<[(I, I, ED)]>,
}

impl<I: Index, ED: EdgeData> Graph<I, ED> for DirectedEdgeListGraph<I, ED> {
    fn num_vertices(&self) -> I {
        self.num_vertices
    }

    fn num_edges(&self) -> I {
        I::new(self.edges.len())
    }

    fn edges(&self) -> Box<dyn Iterator<Item = (I, I, ED)> + '_> {
        Box::new(self.edges.iter().copied())
    }

    fn new_with_edge_data(num_vertices: I, edges: &[(I, I, ED)]) -> Self {
        Self {
            num_vertices,
            edges: edges.into(),
        }
    }

    fn new(num_vertices: I, edges: &[(I, I)]) -> Self {
        Self {
            num_vertices,
            edges: edges
                .iter()
                .map(|e| (e.source(), e.sink(), ED::default()))
                .collect(),
        }
    }
}

impl<I: Index, ED: EdgeData> DirectedGraph<I, ED> for DirectedEdgeListGraph<I, ED> {}

/// An undirected graph stored as number of vertices and list of edges.
///
/// Internally this uses [DirectedEdgeListGraph] and stores each undirected edge
/// once as ordered tuple. Most methods delegate to this internal representation,
/// but [Graph::edges] and all dependend methods are reimplemented to return each
/// undirected (non-loop) edge once *per direction*.
#[derive(Clone, Debug)]
pub struct UndirectedEdgeListGraph<I: Index, ED: EdgeData = ()> {
    graph: DirectedEdgeListGraph<I, ED>,
}

impl<I: Index, ED: EdgeData> Graph<I, ED> for UndirectedEdgeListGraph<I, ED> {
    fn num_vertices(&self) -> I {
        self.graph.num_vertices()
    }

    fn num_edges(&self) -> I {
        self.graph.num_edges()
    }

    fn edges(&self) -> Box<dyn Iterator<Item = (I, I, ED)> + '_> {
        let transposed_edges = self
            .graph
            .edges()
            .filter(|e| e.source() != e.sink()) // loops should not be repeated
            .map(|e| (e.sink(), e.source(), e.data()));

        Box::new(self.graph.edges().chain(transposed_edges))
    }

    fn new_with_edge_data(num_vertices: I, edges: &[(I, I, ED)]) -> Self {
        Self {
            graph: DirectedEdgeListGraph::new_with_edge_data(num_vertices, edges),
        }
    }

    fn new(num_vertices: I, edges: &[(I, I)]) -> Self {
        Self {
            graph: DirectedEdgeListGraph::new(num_vertices, edges),
        }
    }
}

impl<I: Index, ED: EdgeData> UndirectedGraph<I, ED> for UndirectedEdgeListGraph<I, ED> {}
