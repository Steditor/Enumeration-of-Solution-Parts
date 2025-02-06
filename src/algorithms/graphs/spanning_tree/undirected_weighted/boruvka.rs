use std::cmp::{max, min, Ordering};

use compare::{Compare, Extract};

use crate::{
    algorithms::graphs::search::dfs::dfs_forest,
    data_structures::{
        graphs::{Edge, EdgeData, Forest, Graph, UndirectedAdjacencyArrayGraph, UndirectedGraph},
        union_find::{DisjointSet, UnionFind},
        Index,
    },
    experiments::ExperimentAlgorithm,
};

use super::{AlgorithmType, MstAlgorithm};

pub const BORUVKA: AlgorithmType =
    ExperimentAlgorithm::TotalTimeAlgorithm("total-boruvka", |graph| Ok(Boruvka::mst_for(graph)));

/// Borůvka's MST algorithm
pub struct Boruvka {}

impl<I: Index, ED: EdgeData> MstAlgorithm<I, ED> for Boruvka {
    fn mst_for(graph: &impl UndirectedGraph<I, ED>) -> Forest<I, ED>
    where
        ED: Ord,
    {
        Self::comparator_st_for(graph, Extract::new(|e: &(I, I, ED)| e.data()))
    }

    fn comparator_st_for<C: Compare<(I, I, ED)>>(
        graph: &impl UndirectedGraph<I, ED>,
        comparator: C,
    ) -> Forest<I, ED> {
        // add tie-breaking to the comparator
        let comparator = comparator
            .then(Extract::new(|e: &(I, I, ED)| min(e.source(), e.sink())))
            .then(Extract::new(|e: &(I, I, ED)| max(e.source(), e.sink())));

        let mut tree_edges: Vec<(I, I, ED)> = Vec::with_capacity(graph.num_vertices().index() - 1);
        let mut components = UnionFind::new_with_size(graph.num_vertices());

        let mut done = false;
        while !done {
            components.flatten();
            let mut new_edges: Vec<Option<(I, I, ED)>> =
                vec![Option::None; graph.num_vertices().index()];

            // find new crossing edges
            for e in graph.edges() {
                // we called `flatten()` before, so if the parents are the same, the components are.
                if components.is_same_parent(e.source(), e.sink()) {
                    continue;
                }

                let source_component = components.find(e.source());
                new_edges[source_component.index()] = match new_edges[source_component.index()] {
                    None => Some(e),
                    Some(old_e) => match comparator.compare(&e, &old_e) {
                        Ordering::Less | Ordering::Equal => Some(e),
                        Ordering::Greater => Some(old_e),
                    },
                };
            }

            // filter out duplicate edges
            for i in components.elements() {
                if let Some(edge_i) = new_edges[i.index()] {
                    let j = if components.find(edge_i.source()) == i {
                        components.find(edge_i.sink())
                    } else {
                        components.find(edge_i.source())
                    };
                    if new_edges[j.index()].is_some_and(|edge_j| edge_i.is_same_undirected(&edge_j))
                    {
                        new_edges[j.index()] = None;
                    }
                }
            }

            done = true;
            // add edges to the growing tree
            for e in new_edges.iter().flatten() {
                done = false;
                tree_edges.push(*e);
                components.union(e.source(), e.sink());
            }
        }

        let tree_graph =
            UndirectedAdjacencyArrayGraph::new_with_edge_data(graph.num_vertices(), &tree_edges);
        dfs_forest(&tree_graph)
    }
}
