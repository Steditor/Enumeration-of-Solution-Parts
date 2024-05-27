mod directed_adjacency_arrays_graph;
mod directed_edge_list_graph;
mod direction;
mod index;

pub use self::directed_adjacency_arrays_graph::DirectedAdjacencyArraysGraph;
pub use self::directed_edge_list_graph::DirectedEdgeListGraph;
pub use self::direction::Direction;
pub use self::index::Index;

/// A graph with directed edges.
pub trait DirectedGraph<I: Index> {
    /// Returns the number of vertices of the graph
    fn num_vertices(&self) -> I;
    /// Returns the number of edges of the graph
    fn num_edges(&self) -> I;

    /// Returns the degree of the specified vertex.
    ///
    /// Depending on `dir` this will return the out- or the in-degree of `v`.
    fn degree(&self, v: I, dir: Direction) -> I;

    /// Returns the neighbors of the specified vertex.
    ///
    /// Depending on `dir` this will return the successors (out-neighbors)
    /// or the predecessors (in-neighbors) of `v`.
    fn neighbors(&self, v: I, dir: Direction) -> Box<dyn Iterator<Item = I> + '_>;
}
