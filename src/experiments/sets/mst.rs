use rand::distributions::Uniform;

use crate::{
    algorithms::graphs::spanning_tree::undirected_weighted,
    data_generators::graphs::UndirectedConnected,
    experiments::{runner, InstanceGenerator},
};

use super::{AggregationOptions, ExperimentOptions, ExperimentSet};

pub fn experiment_set() -> ExperimentSet {
    ExperimentSet { run, aggregate }
}

const ALGORITHMS: [undirected_weighted::AlgorithmType; 8] = [
    undirected_weighted::ENUMERATE_WITH_BORUVKA,
    undirected_weighted::ENUMERATE_WITH_KRUSKAL,
    undirected_weighted::ENUMERATE_WITH_PRIM,
    undirected_weighted::BORUVKA,
    undirected_weighted::KRUSKAL_IQS,
    undirected_weighted::KRUSKAL_PDQ,
    undirected_weighted::PRIM,
    undirected_weighted::INCREMENTAL_PRIM,
];

fn run(options: &mut ExperimentOptions) {
    let graph_sizes = [
        100, 200, 300, 400, 500, 600, 700, 800, 900, 1_000, 2_000, 3_000, 4_000, 5_000, 6_000,
        7_000, 8_000, 9_000, 10_000, 20_000, 30_000, 40_000, 50_000, 60_000, 70_000, 80_000,
        90_000, 100_000, 200_000,
    ];
    let instances_per_size = 10;
    let runs_per_instance = 5;
    let limit_expected_edges = u32::MAX as f64 * 0.75;
    let max_size = options.max_size;

    for num_vertices in graph_sizes
        .into_iter()
        .filter(|&size| max_size.is_none_or(|max| size <= max))
    {
        // expected order of edges: n^2 with different constants
        let mut edge_generation_parameters = vec![
            (1.0 / 4.0, "0.25"),
            (1.0 / 8.0, "0.125"),
            (1.0 / 16.0, "0.0625"),
            (1.0 / 32.0, "0.03125"),
        ];
        // expected order: n^1.5 = n sqrt(n)
        edge_generation_parameters.push((
            (num_vertices as f64).sqrt() / num_vertices as f64,
            "n.sqrt(n)",
        ));
        // expected order: n^1.25 = n sqrt(sqrt(n))
        edge_generation_parameters.push((
            (num_vertices as f64).powf(0.25) / num_vertices as f64,
            "n.sqrt(sqrt(n))",
        ));

        for (edge_probability, parameter_label) in edge_generation_parameters {
            let expected_edges = num_vertices as f64 * num_vertices as f64 * edge_probability;
            if expected_edges < num_vertices as f64 || expected_edges > limit_expected_edges {
                continue;
            }

            let edge_data_generator = Uniform::new(0, expected_edges.floor() as u32 / 4);

            log::info!(
                "Run MST algorithm for {} jobs and parameter {} (edge probability {}).",
                num_vertices,
                parameter_label,
                edge_probability,
            );
            for i in 1..=instances_per_size {
                log::info!(
                    "Solve instance {:2}/{:2} with {} vertices and parameter {} (edge probability {}).",
                    i,
                    instances_per_size,
                    num_vertices,
                    parameter_label,
                    edge_probability,
                );
                let mut generator = UndirectedConnected::new(
                    num_vertices,
                    edge_probability,
                    edge_data_generator,
                    parameter_label.to_string(),
                );

                runner::run_cachable_experiment::<_, _, _, _, (), _, (), _>(
                    &mut generator,
                    options,
                    runs_per_instance,
                    &ALGORITHMS,
                )
                .unwrap();
            }
        }
    }
}

fn aggregate(options: &AggregationOptions) {
    let folder = UndirectedConnected::<u32, u32, Uniform<_>>::path();
    super::aggregate::<_, _, _, ()>(&folder, &ALGORITHMS, options, None)
}
