use serde::{Deserialize, Serialize};

use super::{
    DirectedAdjacencyArrayGraph, DirectedEdgeListGraph, DirectedGraph, Direction, EdgeData, Graph,
    Index,
};

/// A graph stored as arrays of out- and in-adjacencies.
///
/// By storing both the graph and the transposed graph,
/// we trade space for faster access to in-degrees, in-adjacencies and in-neighbors.
///
/// See [AdjacencyArray] for implementation details.
#[derive(Serialize, Deserialize, Debug)]
pub struct InOutAdjacencyArraysGraph<I: Index, ED: EdgeData = ()> {
    adjacencies: DirectedAdjacencyArrayGraph<I, ED>,
    transposed_adjacencies: DirectedAdjacencyArrayGraph<I, ED>,
}

impl<I: Index, ED: EdgeData> Graph<I, ED> for InOutAdjacencyArraysGraph<I, ED> {
    fn num_vertices(&self) -> I {
        self.adjacencies.num_vertices()
    }

    fn num_edges(&self) -> I {
        self.adjacencies.num_edges()
    }

    fn edges(&self) -> Box<dyn Iterator<Item = (I, I, ED)> + '_> {
        self.adjacencies.edges()
    }

    fn degree(&self, v: I, dir: Direction) -> I {
        match dir {
            Direction::OUT => self.adjacencies.degree(v, Direction::OUT),
            Direction::IN => self.transposed_adjacencies.degree(v, Direction::OUT),
        }
    }

    fn degrees(&self, dir: Direction) -> Box<[I]> {
        match dir {
            Direction::OUT => self.adjacencies.degrees(Direction::OUT),
            Direction::IN => self.transposed_adjacencies.degrees(Direction::OUT),
        }
    }

    fn neighbors(&self, v: I, dir: Direction) -> Box<dyn Iterator<Item = I> + '_> {
        match dir {
            Direction::OUT => self.adjacencies.neighbors(v, Direction::OUT),
            Direction::IN => self.transposed_adjacencies.neighbors(v, Direction::OUT),
        }
    }

    fn adjacencies(&self, v: I, dir: Direction) -> Box<dyn Iterator<Item = (I, ED)> + '_> {
        match dir {
            Direction::OUT => self.adjacencies.adjacencies(v, Direction::OUT),
            Direction::IN => self.transposed_adjacencies.adjacencies(v, Direction::OUT),
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

impl<I: Index, ED: EdgeData> DirectedGraph<I, ED> for InOutAdjacencyArraysGraph<I, ED> {}

impl<I: Index, ED: EdgeData> From<&DirectedEdgeListGraph<I, ED>>
    for InOutAdjacencyArraysGraph<I, ED>
{
    fn from(el_graph: &DirectedEdgeListGraph<I, ED>) -> Self {
        Self {
            adjacencies: DirectedAdjacencyArrayGraph::from_edges(el_graph, Direction::OUT),
            transposed_adjacencies: DirectedAdjacencyArrayGraph::from_edges(
                el_graph,
                Direction::IN,
            ),
        }
    }
}
