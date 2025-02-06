use std::marker::PhantomData;

use num::range;
use rand::{
    distributions::{Distribution, Uniform},
    seq::SliceRandom,
    Rng, SeedableRng,
};
use rand_pcg::Pcg64;

use crate::{
    data_generators::permutations::Permutation,
    data_structures::{
        graphs::{Edge, EdgeData, Graph, InOutAdjacencyArraysGraph},
        Index,
    },
    experiments::InstanceGenerator,
};

/// Generate DAG edges in the uniform G(n,p) model.
///
/// All edges point in the direction of the higher vertex id and are either sorted by source or by sink vertex id.
/// Thus, for a truly random graph, you should permute vertex ids and shuffle the edges.
///
/// This generator samples edges by skipping the expected waiting time
/// until the respective next edge is selected in the plain Bernoulli process,
/// as described in \[1\].
///
/// \[1\] V. Batagelj and U. Brandes, “Efficient generation of large random networks,” Phys. Rev. E, vol. 71, no. 3, p. 036113, Mar. 2005, doi: [10.1103/PhysRevE.71.036113](https://doi.org/10.1103/PhysRevE.71.036113).
pub fn generate_sorted_dag_edges<I: Index, ED: EdgeData>(
    num_vertices: I,
    edge_probability: f64,
    edge_data_generator: &impl Distribution<ED>,
    rng: &mut impl Rng,
) -> Vec<(I, I, ED)> {
    let n = num_vertices.index();
    if edge_probability <= 0.0 {
        Vec::new() // empty graph
    } else if edge_probability >= 1.0 {
        // generate all forward edges
        let mut edges: Vec<(I, I, ED)> = Vec::with_capacity(n * (n - 1) / 2);
        for i in range(0, n - 1) {
            for j in range(i + 1, n) {
                edges.push((I::new(i), I::new(j), edge_data_generator.sample(rng)));
            }
        }
        edges
    } else {
        // generate random forward-edges
        let mut edges: Vec<(I, I, ED)> = Vec::with_capacity(n * (n - 1) / 2);
        let lp = (1.0 - edge_probability).log2();
        let distribution: Uniform<f64> = Uniform::new(0.0, 1.0);
        let mut v = 1;
        // in the original version, w starts as -1. We start at 0 to avoid
        // a signed data type and adjust the calculation below accordingly
        let mut w = 0;
        while v < n {
            let lr = (1.0 - distribution.sample(rng)).log2();
            // The next statement has +1 in the original version; we added 1 before the loop
            // and will do so again at the end of the loop, just before the next iteration
            w += (lr / lp).floor() as usize;
            while w >= v && v < n {
                w -= v;
                v += 1;
            }
            if v < n {
                edges.push((I::new(w), I::new(v), edge_data_generator.sample(rng)));
            }
            // Postponed +1, which is at the beginning of the loop in the original version
            w += 1;
        }
        edges
    }
}

/// Generate DAG edges in the uniform G(n,p) model.
///
/// # Panics
///
/// Panics if `edge_probability` is not between 0 and 1 (inclusive).
pub fn generate_edges<I: Index, ED: EdgeData>(
    num_vertices: I,
    edge_probability: f64,
    edge_data_generator: &impl Distribution<ED>,
    rng: &mut impl Rng,
) -> Vec<(I, I, ED)> {
    let mut edges: Vec<(I, I, ED)> =
        generate_sorted_dag_edges(num_vertices, edge_probability, edge_data_generator, rng);

    let vertex_permutation = Permutation::permutation(rng, I::zero().range(num_vertices));

    // permute vertex ids
    for e in edges.iter_mut() {
        *e = (
            vertex_permutation[e.source().index()],
            vertex_permutation[e.sink().index()],
            e.data(),
        );
    }

    // shuffle order of edges
    edges.shuffle(rng);

    edges
}

/// A random directed acyclic graph.
///
/// This generates a random DAG in the uniform G(n,p) random graph model by
/// giving each "forward" edge the same probability of existing in the graph.
pub struct DAG<I, ED: EdgeData, D: Distribution<ED>> {
    num_vertices: I,
    edge_probability: f64,
    edge_data_generator: D,
    parameter_label: String,
    _phantom: PhantomData<ED>,
}

impl<I, ED: EdgeData, D: Distribution<ED>> DAG<I, ED, D> {
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
    InstanceGenerator<InOutAdjacencyArraysGraph<I, ED>> for DAG<I, ED, D>
{
    fn path() -> String {
        String::from("./data/graphs/dags/")
    }

    fn file_name(&self) -> String {
        format!("{}_{}", self.num_vertices, self.parameter_label)
    }

    /// Generate the experiment instance.
    ///
    /// # Panics
    ///
    /// Panics if `edge_probability` is not between 0 and 1 (inclusive).
    fn generate(&self, seed: u64) -> InOutAdjacencyArraysGraph<I, ED> {
        InOutAdjacencyArraysGraph::new_with_edge_data(
            self.num_vertices,
            &generate_edges(
                self.num_vertices,
                self.edge_probability,
                &self.edge_data_generator,
                &mut Pcg64::seed_from_u64(seed),
            ),
        )
    }
}
