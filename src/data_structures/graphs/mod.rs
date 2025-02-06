mod adjacency_array_graph;
mod edge;
mod edge_list_graph;
mod forest;
mod in_out_adjacency_arrays_graph;

#[cfg(test)]
mod tests;

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::experiments::aggregation::{Aggregatable, Aggregation, StreamingAggregation};
use crate::experiments::StatisticsCollector;

pub use self::adjacency_array_graph::{DirectedAdjacencyArrayGraph, UndirectedAdjacencyArrayGraph};
pub use self::edge::{Adjacency, Direction, Edge, EdgeData, EdgeWeight};
pub use self::edge_list_graph::{DirectedEdgeListGraph, UndirectedEdgeListGraph};
pub use self::forest::Forest;
pub use self::in_out_adjacency_arrays_graph::InOutAdjacencyArraysGraph;

use super::Index;

/// A graph
pub trait Graph<I: Index, ED: EdgeData = ()> {
    /// Returns the number of vertices of the graph.
    fn num_vertices(&self) -> I;

    /// Returns an iterator for the vertices of the graph.
    fn vertices(&self) -> I::IndexIterator {
        I::zero().range(self.num_vertices())
    }

    /// Returns the number of edges of the graph.
    ///
    /// In case of an undirected graph this should be the number of distinct
    /// edges, meaning each adjacent pair of unordered vertices is included
    /// exactly once in the count.
    fn num_edges(&self) -> I;

    /// Returns all edges of the graph.
    ///
    /// In case of an undirected graph this iterator should visit each edge
    /// between two different vertices twice; once per direction. Note that
    /// loops (edges from a vertex to itself) are to be included only once.
    fn edges(&self) -> Box<dyn Iterator<Item = (I, I, ED)> + '_>;

    /// Returns the degree of the specified vertex.
    ///
    /// Depending on `dir` this will return the out- or the in-degree of `v`.
    /// For undirected graphs, `dir` is irrelevant.
    fn degree(&self, v: I, dir: Direction) -> I {
        I::new(self.edges().filter(|edge| dir.vertex(edge) == v).count())
    }

    /// Returns the degrees of all vertices
    ///
    /// Depending on `dir` this will return the out- or the in-degrees.
    /// For undirected graphs, `dir` is irrelevant.
    fn degrees(&self, dir: Direction) -> Box<[I]> {
        let mut degrees = vec![I::zero(); self.num_vertices().index()].into_boxed_slice();
        for edge in self.edges() {
            degrees[dir.vertex(&edge).index()] += I::one();
        }
        degrees
    }

    /// Returns the neighbors of the specified vertex.
    ///
    /// Depending on `dir` this will return the successors (out-neighbors)
    /// or the predecessors (in-neighbors) of `v`.
    /// For undirected graphs the set of neighbors is the same regardless of the direction,
    /// but the order in which the neighbors are produced might differ.
    fn neighbors(&self, v: I, dir: Direction) -> Box<dyn Iterator<Item = I> + '_> {
        Box::new(
            self.edges()
                .filter(move |e| dir.vertex(e) == v)
                .map(move |e| dir.other(&e)),
        )
    }

    /// Returns the neighbors of the specified vertex along with potential edge data.
    ///
    /// Depending on `dir` this will return adjacencies to successors (out-neighbors)
    /// or predecessors (in-neighbors) of `v`.
    /// For undirected graphs the set of adjacencies is the same regardless of the direction,
    /// but the order in which the adjacencies are produced might differ.
    fn adjacencies(&self, v: I, dir: Direction) -> Box<dyn Iterator<Item = (I, ED)> + '_> {
        Box::new(
            self.edges()
                .filter(move |e| dir.vertex(e) == v)
                .map(move |e| (dir.other(&e), e.data())),
        )
    }

    /// Creates a new graph with the given edges with associated data
    ///
    /// This function assumes that each (un)directed edge is present in the edge list exactly once.
    /// No duplicate checks are made.
    fn new_with_edge_data(num_vertices: I, edges: &[(I, I, ED)]) -> Self
    where
        Self: Sized;

    /// Creates a new graph with the given edges
    ///
    /// This function assumes that each (un)directed edge is present in the edge list exactly once.
    /// No duplicate checks are made.
    ///
    /// Edge data is set to the default for `ED` for all edges.
    fn new(num_vertices: I, edges: &[(I, I)]) -> Self
    where
        Self: Sized;
}

/// Marker trait for directed graphs
pub trait DirectedGraph<I: Index, ED: EdgeData = ()>: Graph<I, ED> {}

/// Marker trait for undirected graphs
pub trait UndirectedGraph<I: Index, ED: EdgeData = ()>: Graph<I, ED> {}

#[derive(Serialize, Deserialize)]
pub struct GraphStatistics {
    num_vertices: usize,
    num_edges: usize,
    deg_in_min: f64,
    deg_in_avg: f64,
    deg_in_max: f64,
    deg_out_min: f64,
    deg_out_avg: f64,
    deg_out_max: f64,
}

pub struct GraphStatisticsCollector<G, I, ED>
where
    G: Graph<I, ED>,
    I: Index + Aggregatable,
    ED: EdgeData,
{
    _phantom: PhantomData<(G, I, ED)>,
}

impl<G, I, ED> StatisticsCollector<G, GraphStatistics> for GraphStatisticsCollector<G, I, ED>
where
    G: Graph<I, ED>,
    I: Index + Aggregatable,
    ED: EdgeData,
{
    fn collect_statistics(graph: &G) -> Option<GraphStatistics> {
        let mut in_degrees = StreamingAggregation::default();
        graph
            .degrees(Direction::IN)
            .iter()
            .for_each(|degree| in_degrees.push(*degree));
        let mut out_degrees = StreamingAggregation::default();
        graph
            .degrees(Direction::OUT)
            .iter()
            .for_each(|degree| out_degrees.push(*degree));

        Some(GraphStatistics {
            num_vertices: graph.num_vertices().index(),
            num_edges: graph.num_edges().index(),
            deg_in_min: in_degrees.min,
            deg_in_avg: in_degrees.avg,
            deg_in_max: in_degrees.max,
            deg_out_min: out_degrees.min,
            deg_out_avg: out_degrees.avg,
            deg_out_max: out_degrees.max,
        })
    }
}
