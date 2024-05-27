use serde::{Deserialize, Serialize};

use super::{directed_edge_list_graph::DirectedEdgeListGraph, DirectedGraph, Direction, Index};

/// A directed graph stored as out- and in-adjacency arrays.
///
/// The data structure is essentially identical to the "standard representation" presented in \[1\] and implemented similar to [the graph_builder crate](https://docs.rs/graph_builder/latest/src/graph_builder/index.rs.html):
/// All out-adjacencies are stored in a single array, sorted by the source vertex.
/// For each source vertex we store the offset of the first adjacency in the combined array. The end is derived by the offset of the next vertex or the end of the adjacency array.
/// The same is stored for in-adjacencies.
///
/// \[1\] F. Kammer and A. Sajenko, “Linear-Time In-Place DFS and BFS on the Word RAM,” in Algorithms and Complexity, P. Heggernes, Ed., in Lecture Notes in Computer Science. Cham: Springer International Publishing, 2019, pp. 286–298. doi: [10.1007/978-3-030-17402-6_24](https://doi.org/10.1007/978-3-030-17402-6_24).
#[derive(Serialize, Deserialize, Debug)]
pub struct DirectedAdjacencyArraysGraph<I: Index> {
    out_offsets: Box<[I]>,
    out_adjacencies: Box<[I]>,
    in_offsets: Box<[I]>,
    in_adjacencies: Box<[I]>,
}

impl<I: Index> DirectedGraph<I> for DirectedAdjacencyArraysGraph<I> {
    fn num_vertices(&self) -> I {
        I::new(self.out_offsets.len())
    }

    fn num_edges(&self) -> I {
        I::new(self.out_adjacencies.len())
    }

    fn degree(&self, v: I, dir: Direction) -> I {
        let (start_inclusive, end_exclusive) = self.bounds(v, dir);
        end_exclusive - start_inclusive
    }

    fn neighbors(&self, v: I, dir: Direction) -> Box<dyn Iterator<Item = I> + '_> {
        let (start_inclusive, end_exclusive) = self.bounds(v, dir);
        Box::new(
            self.adjacencies(dir)[start_inclusive.index()..end_exclusive.index()]
                .iter()
                .copied(),
        )
    }
}

impl<I: Index> DirectedAdjacencyArraysGraph<I> {
    #[inline]
    fn offsets(&self, dir: Direction) -> &[I] {
        match dir {
            Direction::OUT => &self.out_offsets,
            Direction::IN => &self.in_offsets,
        }
    }

    #[inline]
    fn adjacencies(&self, dir: Direction) -> &[I] {
        match dir {
            Direction::OUT => &self.out_adjacencies,
            Direction::IN => &self.in_adjacencies,
        }
    }

    #[inline]
    fn bounds(&self, v: I, dir: Direction) -> (I, I) {
        let offsets = self.offsets(dir);
        let start_inclusive = offsets[v.index()];
        let end_exclusive = match offsets.get(v.index() + 1) {
            Some(x) => *x,
            None => self.num_edges(),
        };

        (start_inclusive, end_exclusive)
    }
}

impl<I: Index> From<&DirectedEdgeListGraph<I>> for DirectedAdjacencyArraysGraph<I> {
    fn from(el_graph: &DirectedEdgeListGraph<I>) -> Self {
        let out_aa = AdjacencyArray::from_edges(el_graph, Direction::OUT);
        let in_aa = AdjacencyArray::from_edges(el_graph, Direction::IN);

        Self {
            out_offsets: out_aa.offsets,
            out_adjacencies: out_aa.adjacencies,
            in_offsets: in_aa.offsets,
            in_adjacencies: in_aa.adjacencies,
        }
    }
}

struct AdjacencyArray<I: Index> {
    offsets: Box<[I]>,
    adjacencies: Box<[I]>,
}

impl<I: Index> AdjacencyArray<I> {
    fn from_edges(el_graph: &DirectedEdgeListGraph<I>, dir: Direction) -> Self {
        // compute offsets
        let degrees = el_graph.degrees(dir);
        let mut offsets = degrees_to_offsets(degrees);

        // collect edges
        let mut adjacencies = vec![I::new(0); el_graph.num_edges().index()].into_boxed_slice();
        for edge in el_graph.edges() {
            let vertex = dir.vertex(edge);
            let other = dir.other(edge);

            adjacencies[offsets[vertex.index()].index()] = other;
            offsets[vertex.index()] += I::new(1);
        }

        // reset offsets
        offsets.rotate_right(1);
        offsets[0] = I::new(0);

        AdjacencyArray {
            offsets,
            adjacencies,
        }
    }
}

fn degrees_to_offsets<I: Index>(mut degrees: Box<[I]>) -> Box<[I]> {
    let mut current_offset: I = I::new(0);
    for entry in degrees.iter_mut() {
        let degree = *entry;
        *entry = current_offset;
        current_offset += degree;
    }
    degrees
}
