use num::Unsigned;
use rand::distributions::{uniform::SampleUniform, Uniform};

use crate::{
    algorithms::graphs::shortest_distances::sssd::{
        unweighted, unweighted_lazy, weighted, AlgorithmType,
    },
    data_generators::graphs::Undirected,
    data_structures::{
        graphs::{
            EdgeData, EdgeWeight, Graph, GraphStatisticsCollector, UndirectedAdjacencyArrayGraph,
        },
        Index,
    },
    experiments::{runner, InstanceGenerator},
};

use super::{AggregationOptions, ExperimentOptions, ExperimentSet};

// Unweighted Graphs

const fn unweighted_algorithms<G, I, ED>() -> [AlgorithmType<G, I>; 3]
where
    G: Graph<I, ED>,
    I: Index,
    ED: EdgeData,
{
    [
        unweighted::algorithm_bfs(),
        unweighted::algorithm_enum_bfs(),
        unweighted_lazy::algorithm_enum_bfs_lazy(),
    ]
}

pub fn unweighted_experiment_set() -> ExperimentSet {
    ExperimentSet {
        run: |options| {
            run_sssd_on_worst_case(
                options,
                &unweighted_algorithms::<UndirectedAdjacencyArrayGraph<u32, u8>, _, _>(),
            );
            run_sssd_on_gnp(
                options,
                &unweighted_algorithms::<UndirectedAdjacencyArrayGraph<u32, u32>, _, _>(),
            );
        },
        aggregate: |options| {
            aggregate_sssd_worst_case(
                options,
                &unweighted_algorithms::<UndirectedAdjacencyArrayGraph<u32, u8>, _, _>(),
            );
            aggregate_sssd_gnp(
                options,
                &unweighted_algorithms::<UndirectedAdjacencyArrayGraph<u32, u32>, _, _>(),
            );
        },
    }
}

// Weighted Graphs

const fn weighted_algorithms<G, I, EW>() -> [AlgorithmType<G, I, EW>; 2]
where
    G: Graph<I, EW>,
    I: Index,
    EW: EdgeWeight + Unsigned,
{
    [
        weighted::algorithm_dijkstra(),
        weighted::algorithm_enum_dijkstra(),
    ]
}

pub fn weighted_experiment_set() -> ExperimentSet {
    ExperimentSet {
        run: |options| {
            run_sssd_on_worst_case(
                options,
                &weighted_algorithms::<UndirectedAdjacencyArrayGraph<u32, u8>, _, _>(),
            );
            run_sssd_on_gnp(
                options,
                &weighted_algorithms::<UndirectedAdjacencyArrayGraph<u32, u32>, _, _>(),
            );
        },
        aggregate: |options| {
            aggregate_sssd_worst_case(
                options,
                &weighted_algorithms::<UndirectedAdjacencyArrayGraph<u32, u8>, _, _>(),
            );
            aggregate_sssd_gnp(
                options,
                &weighted_algorithms::<UndirectedAdjacencyArrayGraph<u32, u32>, _, _>(),
            );
        },
    }
}

/// Run the given algorithms on worst-case-instance graphs as described in
/// "Shortest Distances as Enumeration Problem" Fig. 1
const WORST_CASE_PATH: &str = "./data/graphs/sssd-lower-bound/";
fn run_sssd_on_worst_case<D>(
    options: &mut ExperimentOptions,
    algorithms: &[AlgorithmType<UndirectedAdjacencyArrayGraph<u32, u8>, u32, D>],
) {
    let graph_sizes = [
        100, 200, 300, 400, 500, 600, 700, 800, 900, 1_000, 2_000, 3_000, 4_000, 5_000, 6_000,
        7_000, 8_000, 9_000, 10_000, 20_000, 30_000,
    ]
    .map(|k| (k, k * k + k)); // (k, n)

    let runs_per_instance = 5;
    let max_size = options.max_size;

    for (k, n) in graph_sizes
        .into_iter()
        .filter(|(_, n)| max_size.is_none_or(|max| *n <= max))
    {
        let m = (k * (k - 1) / 2 + k * k) as usize;
        let mut edges = Vec::with_capacity(m);

        // build an almost-k-clique between vertices {0, 1, ..., k-1} with one missing edge (k-2, k-1).
        (0..k - 2)
            .flat_map(|u| (u + 1..k).map(move |v| (u, v)))
            .for_each(|(u, v)| edges.push((u, v, 1)));

        // build a path from k-1 to k-2 that runs over all remaining vertices
        (k - 1..n)
            .chain(std::iter::once(k - 2))
            .map_windows(|[u, v]| (*u, *v))
            .for_each(|(u, v)| edges.push((u, v, 1)));

        let graph = UndirectedAdjacencyArrayGraph::<u32, u8>::new_with_edge_data(n, &edges);

        // path/size
        let instance_prefix = format!("{WORST_CASE_PATH}/{n}");

        if options.collect_statistics {
            runner::collect_statistics_for_instance::<GraphStatisticsCollector<_, _, _>, _, _>(
                &graph,
                format!("{instance_prefix}_all_0"),
            )
            .unwrap();
        }

        if options.run_algorithms {
            log::info!("Run SSSD algorithm for worst-case instance with clique size {k} / start at clique vertex.");
            // start vertex 0 is worst case: the whole clique will be processed first
            let mut instance = (graph, 0);
            runner::run_experiment_for_instance::<_, _, _, (), ()>(
                &instance,
                format!("{instance_prefix}_worst-case_0"),
                runs_per_instance,
                algorithms,
            )
            .unwrap();

            log::info!("Run SSSD algorithm for worst-case instance with clique size {k} / start at middle-path vertex.");
            // start vertex in the middle of the path is best case: the whole path will be processed first
            instance.1 = (k - 1) + (n - k) / 2;
            runner::run_experiment_for_instance::<_, _, _, (), ()>(
                &instance,
                format!("{instance_prefix}_best-case_0"),
                runs_per_instance,
                algorithms,
            )
            .unwrap();
        }
    }
}

fn aggregate_sssd_worst_case<G, I, ED, D>(
    options: &AggregationOptions,
    algorithms: &[AlgorithmType<G, I, D>],
) where
    G: Graph<I, ED>,
    I: Index,
    ED: EdgeData,
{
    super::aggregate::<_, _, _, ()>(WORST_CASE_PATH, algorithms, options, None);
}

/// Run the given algorithms on random graphs
fn run_sssd_on_gnp<D>(
    options: &mut ExperimentOptions,
    algorithms: &[AlgorithmType<UndirectedAdjacencyArrayGraph<u32, u32>, u32, D>],
) {
    let graph_sizes = [
        100, 200, 300, 400, 500, 600, 700, 800, 900, 1_000, 2_000, 3_000, 4_000, 5_000, 6_000,
        7_000, 8_000, 9_000, 10_000, 20_000, 30_000, 40_000, 50_000, 60_000, 70_000, 80_000,
        90_000, 100_000, 200_000,
    ];

    let instances_per_size = 10;
    let runs_per_instance = 5;
    let limit_expected_edges = u32::MAX as f64 * 0.5;
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
                "Run SSSD algorithm for {} vertices and parameter {} (edge probability {}).",
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
                let mut generator = Undirected::new(
                    num_vertices,
                    edge_probability,
                    edge_data_generator,
                    parameter_label.to_string(),
                );

                runner::run_cachable_experiment_with_input_transform::<
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    GraphStatisticsCollector<_, _, _>,
                    _,
                    (),
                    _,
                >(
                    &mut generator,
                    options,
                    |graph| (graph, 0),
                    runs_per_instance,
                    algorithms,
                )
                .unwrap();
            }
        }
    }
}

fn aggregate_sssd_gnp<G, I, ED, D>(
    options: &AggregationOptions,
    algorithms: &[AlgorithmType<G, I, D>],
) where
    G: Graph<I, ED>,
    I: Index + SampleUniform,
    ED: EdgeData,
{
    let folder = Undirected::<u32, u32, Uniform<_>>::path();
    super::aggregate::<_, _, _, ()>(&folder, algorithms, options, None);
}
