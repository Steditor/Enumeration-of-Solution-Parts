use std::marker::PhantomData;

use rand::{distributions::Distribution, Rng, SeedableRng};
use rand_pcg::Pcg64;

use crate::{
    algorithms::graphs::search::bfs::is_connected,
    data_structures::{
        graphs::{Edge, EdgeData, Graph, UndirectedAdjacencyArrayGraph},
        Index,
    },
    experiments::InstanceGenerator,
};

/// Generate undirected edges in the uniform G(n,p) model
///
/// # Panics
///
/// Panics if `edge_probability` is not between 0 and 1 (inclusive).
fn generate_edges<I: Index, ED: EdgeData>(
    num_vertices: I,
    edge_probability: f64,
    edge_data_generator: &impl Distribution<ED>,
    rng: &mut impl Rng,
) -> Vec<(I, I, ED)> {
    // Generate DAG as proxy for an undirected graph
    let mut edges =
        super::dag::generate_edges(num_vertices, edge_probability, edge_data_generator, rng);

    // Flip edge directions with probability 0.5
    for edge in edges.iter_mut() {
        if rng.gen_bool(0.5) {
            *edge = (edge.sink(), edge.source(), edge.data());
        }
    }
    edges
}

/// A random graph.
///
/// This generates a random Graph in the uniform G(n,p) random graph model by
/// giving each edge the same probability of existing in the graph.
pub struct Undirected<I: Index, ED: EdgeData, D: Distribution<ED>> {
    num_vertices: I,
    edge_probability: f64,
    edge_data_generator: D,
    parameter_label: String,
    _phantom: PhantomData<ED>,
}

impl<I: Index, ED: EdgeData, D: Distribution<ED>> Undirected<I, ED, D> {
    pub fn new(
        num_vertices: I,
        edge_probability: f64,
        edge_data_generator: D,
        parameter_label: String,
    ) -> Self {
        Self {
            num_vertices,
            edge_probability,
            edge_data_generator,
            parameter_label,
            _phantom: PhantomData,
        }
    }
}

impl<I: Index, ED: EdgeData, D: Distribution<ED>>
    InstanceGenerator<UndirectedAdjacencyArrayGraph<I, ED>> for Undirected<I, ED, D>
{
    fn path() -> String {
        String::from("./data/graphs/undirected/")
    }

    fn file_name(&self) -> String {
        format!("{}_{}", self.num_vertices, self.parameter_label)
    }

    /// Generate the experiment instance.
    ///
    /// # Panics
    ///
    /// Panics if `edge_probability` is not between 0 and 1 (inclusive).
    fn generate(&self, seed: u64) -> UndirectedAdjacencyArrayGraph<I, ED> {
        let mut rng = Pcg64::seed_from_u64(seed);
        let edges = generate_edges(
            self.num_vertices,
            self.edge_probability,
            &self.edge_data_generator,
            &mut rng,
        );
        UndirectedAdjacencyArrayGraph::new_with_edge_data(self.num_vertices, &edges)
    }
}

pub struct UndirectedConnected<I: Index, ED: EdgeData, D: Distribution<ED>> {
    num_vertices: I,
    edge_probability: f64,
    edge_data_generator: D,
    parameter_label: String,
    _phantom: PhantomData<ED>,
}

impl<I: Index, ED: EdgeData, D: Distribution<ED>> UndirectedConnected<I, ED, D> {
    pub fn new(
        num_vertices: I,
        edge_probability: f64,
        edge_data_generator: D,
        parameter_label: String,
    ) -> Self {
        Self {
            num_vertices,
            edge_probability,
            edge_data_generator,
            parameter_label,
            _phantom: PhantomData,
        }
    }
}

impl<I: Index, ED: EdgeData, D: Distribution<ED>>
    InstanceGenerator<UndirectedAdjacencyArrayGraph<I, ED>> for UndirectedConnected<I, ED, D>
{
    fn path() -> String {
        String::from("./data/graphs/undirected/connected/")
    }

    fn file_name(&self) -> String {
        format!("{}_{}", self.num_vertices, self.parameter_label)
    }

    /// Generate the experiment instance by rejection sampling of undirected graphs.
    ///
    /// Note that `edge_probability` should be sufficiently high to make it possible
    /// to sample a connected graph within few tries. `p > log (n) / n` should work fine.
    ///
    /// # Panics
    ///
    /// Panics if `edge_probability` is not between 0 and 1 (inclusive).
    fn generate(&self, seed: u64) -> UndirectedAdjacencyArrayGraph<I, ED> {
        let mut rng = Pcg64::seed_from_u64(seed);
        let mut generator = || {
            UndirectedAdjacencyArrayGraph::new_with_edge_data(
                self.num_vertices,
                &generate_edges(
                    self.num_vertices,
                    self.edge_probability,
                    &self.edge_data_generator,
                    &mut rng,
                ),
            )
        };
        let mut candidate = generator();
        while !is_connected(&candidate) {
            candidate = generator();
        }
        candidate
    }
}
