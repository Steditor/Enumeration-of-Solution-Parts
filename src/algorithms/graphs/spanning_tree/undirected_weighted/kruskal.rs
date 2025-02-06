use compare::{Compare, Extract};

use crate::{
    algorithms::{graphs::search::dfs::dfs_forest, sorting::IQS},
    data_structures::{
        graphs::{Edge, EdgeData, Forest, Graph, UndirectedAdjacencyArrayGraph, UndirectedGraph},
        union_find::{DisjointSet, RankedUnionFind},
        Index,
    },
    experiments::ExperimentAlgorithm,
};

use super::{AlgorithmType, MstAlgorithm};

pub const KRUSKAL_IQS: AlgorithmType =
    ExperimentAlgorithm::TotalTimeAlgorithm("total-kruskal-iqs", |graph| {
        Ok(Kruskal::mst_for(graph))
    });

pub const KRUSKAL_PDQ: AlgorithmType =
    ExperimentAlgorithm::TotalTimeAlgorithm("total-kruskal-pdq", |graph| {
        Ok(Kruskal::rust_sort_mst_for(graph))
    });

/// Kruskal's MST algorithm
///
/// Internally we use incremental quick sort ([IQS]) to avoid having to sort all edges.
pub struct Kruskal {}

impl<I: Index, ED: EdgeData> MstAlgorithm<I, ED> for Kruskal {
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
        let mut tree_edges = Vec::with_capacity(graph.num_vertices().index() - 1);
        let mut components = RankedUnionFind::new_with_size(graph.num_vertices());

        let edges: Vec<_> = graph.edges().collect();
        let sorted_edges = IQS::with_comparator(&edges, comparator);
        let mut disjunct_sets = graph.num_vertices();

        for e in sorted_edges {
            let u = e.source();
            let v = e.sink();
            if !components.is_same(u, v) {
                components.union(u, v);
                tree_edges.push(e);
                disjunct_sets -= I::one();

                if disjunct_sets == I::one() {
                    break;
                }
            }
        }

        let tree_graph =
            UndirectedAdjacencyArrayGraph::new_with_edge_data(graph.num_vertices(), &tree_edges);
        dfs_forest(&tree_graph)
    }
}

impl Kruskal {
    pub fn rust_sort_mst_for<I: Index, ED: EdgeData + Ord>(
        graph: &impl UndirectedGraph<I, ED>,
    ) -> Forest<I, ED> {
        let mut tree_edges = Vec::with_capacity(graph.num_vertices().index() - 1);
        let mut components = RankedUnionFind::new_with_size(graph.num_vertices());

        let mut edges: Vec<_> = graph.edges().collect();
        edges.sort_unstable_by_key(|e| e.data());
        let mut disjunct_sets = graph.num_vertices();

        for e in edges {
            let u = e.source();
            let v = e.sink();
            if !components.is_same(u, v) {
                components.union(u, v);
                tree_edges.push(e);
                disjunct_sets -= I::one();

                if disjunct_sets == I::one() {
                    break;
                }
            }
        }

        let tree_graph =
            UndirectedAdjacencyArrayGraph::new_with_edge_data(graph.num_vertices(), &tree_edges);
        dfs_forest(&tree_graph)
    }
}
