use std::marker::PhantomData;

use compare::{Compare, Extract};

use crate::{
    algorithms::{
        graphs::{search::dfs::dfs_forest, spanning_tree::undirected_weighted::MstPartial},
        sorting::IQS,
    },
    data_structures::{
        graphs::{Edge, EdgeData, Forest, Graph, UndirectedAdjacencyArrayGraph, UndirectedGraph},
        union_find::{DisjointSet, RankedUnionFind},
        Index,
    },
    experiments::{ExperimentAlgorithm, PreparedEnumerationAlgorithm},
};

use super::{AlgorithmType, MstAlgorithm};

pub const KRUSKAL_IQS: AlgorithmType =
    ExperimentAlgorithm::TotalTimeAlgorithm("total-kruskal-iqs", |graph| {
        Ok(Kruskal::mst_for(graph))
    });

pub const KRUSKAL_RUSTSORT: AlgorithmType =
    ExperimentAlgorithm::TotalTimeAlgorithm("total-kruskal-rustsort", |graph| {
        Ok(Kruskal::rust_sort_mst_for(graph))
    });

pub const INCREMENTAL_KRUSKAL: AlgorithmType = ExperimentAlgorithm::EnumerationAlgorithm(
    "incremental-kruskal",
    IncrementalKruskal::enumerator_for,
);

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

pub struct IncrementalKruskal<I: Index, ED: EdgeData> {
    _phantom: PhantomData<(I, ED)>,
}

impl<I: Index, ED: EdgeData> IncrementalKruskal<I, ED> {
    pub fn enumerator_for(
        graph: &impl UndirectedGraph<I, ED>,
    ) -> PreparedEnumerationAlgorithm<MstPartial<I, ED>>
    where
        ED: Ord,
    {
        Self::comparator_enumerator_for(graph, Extract::new(|e: &(I, I, ED)| e.data()))
    }

    pub fn comparator_enumerator_for<C: Compare<(I, I, ED)> + Copy + 'static>(
        graph: &impl UndirectedGraph<I, ED>,
        comparator: C,
    ) -> PreparedEnumerationAlgorithm<MstPartial<I, ED>> {
        Box::new(MstEnumerator::with_comparator(graph, comparator))
    }
}

struct MstEnumerator<I, ED, C>
where
    I: Index,
    ED: EdgeData,
    C: Compare<(I, I, ED)>,
{
    components: RankedUnionFind<I>,
    sorted_edges: IQS<(I, I, ED), C>,
    disjunct_sets: I,
}

impl<I, ED, C> MstEnumerator<I, ED, C>
where
    I: Index,
    ED: EdgeData,
    C: Compare<(I, I, ED)>,
{
    pub fn with_comparator(graph: &impl UndirectedGraph<I, ED>, comparator: C) -> Self {
        let edges: Vec<_> = graph.edges().collect();
        Self {
            components: RankedUnionFind::new_with_size(graph.num_vertices()),
            sorted_edges: IQS::with_comparator(&edges, comparator),
            disjunct_sets: graph.num_vertices(),
        }
    }
}

impl<I, ED, C> Iterator for MstEnumerator<I, ED, C>
where
    I: Index,
    ED: EdgeData,
    C: Compare<(I, I, ED)>,
{
    type Item = MstPartial<I, ED>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.disjunct_sets == I::one() {
            return None;
        }
        for e in self.sorted_edges.by_ref() {
            let u = e.source();
            let v = e.sink();
            if !self.components.is_same(u, v) {
                self.components.union(u, v);
                self.disjunct_sets -= I::one();
                return Some(e);
            }
        }
        // Should never get here, but the compiler wants
        // this path to return something as well
        None
    }
}
