use serde::{de::DeserializeOwned, Serialize};

use crate::{
    data_structures::graphs::{DirectedAdjacencyArraysGraph, DirectedEdgeListGraph, Index},
    experiments::ExperimentGenerator,
    random_generators::{numbers::Rng, permutations::Permutation},
};

/// A random directed acyclic graph.
///
/// This generates a random DAG in the uniform G(n,p) random graph model by
/// giving each "forward" edge the same probability of existing in the graph.
pub struct DAG<'a, I: Index + Serialize + DeserializeOwned> {
    pub rng: &'a mut dyn Rng,
    pub num_vertices: I,
    pub edge_probability: f64,
}

impl<I: Index + Serialize + DeserializeOwned> DAG<'_, I> {
    /// Generate the experiment instance.
    ///
    /// # Panics
    ///
    /// Panics if `edge_probability` is not between 0 and 1 (inclusive).
    fn generate_edge_list_graph(&mut self) -> DirectedEdgeListGraph<I> {
        assert!((0.0..=1.0).contains(&self.edge_probability));

        let vertex_permutation =
            Permutation::permutation(self.rng, I::new(0).range(self.num_vertices));

        let mut edges: Vec<(I, I)> = Vec::new();
        for i in I::new(0).range(self.num_vertices - I::new(1)) {
            for j in (i + I::new(1)).range(self.num_vertices) {
                if self.rng.next_double() < self.edge_probability {
                    edges.push((vertex_permutation[i.index()], vertex_permutation[j.index()]));
                }
            }
        }

        Permutation::shuffle(self.rng, &mut edges);

        DirectedEdgeListGraph::new(self.num_vertices, edges.into_boxed_slice())
    }
}

impl<I: Index + Serialize + DeserializeOwned> ExperimentGenerator<DirectedAdjacencyArraysGraph<I>>
    for DAG<'_, I>
{
    fn path() -> String {
        String::from("./data/graphs/dags/")
    }

    fn file_name(&self) -> String {
        format!(
            "{}_{}_{}",
            self.num_vertices,
            self.edge_probability,
            self.rng.state_id()
        )
    }

    /// Generate the experiment instance.
    ///
    /// # Panics
    ///
    /// Panics if `edge_probability` is not between 0 and 1 (inclusive).
    fn generate(&mut self) -> DirectedAdjacencyArraysGraph<I> {
        DirectedAdjacencyArraysGraph::from(&self.generate_edge_list_graph())
    }
}
