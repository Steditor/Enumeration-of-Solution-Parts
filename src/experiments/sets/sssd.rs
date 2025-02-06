use num::Unsigned;
use rand::{
    distributions::{uniform::SampleUniform, Uniform},
    prelude::Distribution,
    SeedableRng,
};
use rand_pcg::Pcg64;

use crate::{
    algorithms::graphs::shortest_distances::sssd::{
        unweighted, unweighted_lazy, weighted, AlgorithmType,
    },
    data_sets::{
        osm::{OsmReader, OsmReaderOptions},
        GraphReader, GraphSetEntry, GraphSetIterator,
    },
    data_structures::{
        graphs::{
            EdgeData, EdgeWeight, Graph, GraphStatisticsCollector, UndirectedAdjacencyArrayGraph,
        },
        Index,
    },
    experiments::{aggregation::Aggregatable, runner},
};

use super::{AggregationOptions, ExperimentOptions, ExperimentSet};

// Unweighted Graphs

const fn unweighted_algorithms<G, I>() -> [AlgorithmType<G, I>; 3]
where
    G: Graph<I>,
    I: Index,
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
            run_sssd(
                options,
                &unweighted_algorithms::<UndirectedAdjacencyArrayGraph<u32>, _>(),
            )
        },
        aggregate: |options| {
            aggregate_sssd(
                options,
                &unweighted_algorithms::<UndirectedAdjacencyArrayGraph<u32>, _>(),
            )
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
            run_sssd(
                options,
                &weighted_algorithms::<UndirectedAdjacencyArrayGraph<u32, u32>, _, _>(),
            )
        },
        aggregate: |options| {
            aggregate_sssd(
                options,
                &weighted_algorithms::<UndirectedAdjacencyArrayGraph<u32, u32>, _, _>(),
            )
        },
    }
}

/// Run the given algorithms on OpenStreetMap graphs (unweighted or weighted, depending on the algorithms and `G`).
fn run_sssd<G, I, ED, D>(options: &mut ExperimentOptions, algorithms: &[AlgorithmType<G, I, D>])
where
    G: Graph<I, ED>,
    I: Index + SampleUniform + Aggregatable,
    ED: EdgeData,
    OsmReader: GraphReader<G, I, ED, OsmReaderOptions>,
{
    let instances = GraphSetIterator::<OsmReader, G, I, ED, OsmReaderOptions>::new(
        OsmReader::get_file_paths(),
        true,
        OsmReaderOptions::new()
            .with_max_size(options.max_size)
            .merge_ways(true),
    );

    let instances_per_graph = 10;
    let runs_per_instance = 5;

    for GraphSetEntry {
        mut graph, path, ..
    } in instances
    {
        let node_distribution = Uniform::new(I::zero(), graph.num_vertices());

        // path/size_parameters
        let instance_prefix = format!(
            "{}/{}_osm",
            path.parent()
                .expect("input files must have a proper parent folder")
                .display(),
            graph.num_vertices()
        );

        if options.collect_statistics {
            runner::collect_statistics_for_instance::<GraphStatisticsCollector<_, _, _>, _, _>(
                &graph,
                format!("{instance_prefix}_x"),
            )
            .unwrap();
        }

        if options.run_algorithms {
            for i in 1..=instances_per_graph {
                let seed = options.seed_generator.next_u64();
                let instance_path = format!("{}_{}", instance_prefix, seed);
                log::info!(
                    "Solve instance {:2}/{:2} with {} vertices and {} edges from osm file {}.",
                    i,
                    instances_per_graph,
                    graph.num_vertices(),
                    graph.num_edges(),
                    path.display(),
                );

                let mut rng = Pcg64::seed_from_u64(seed);
                let instance = (graph, node_distribution.sample(&mut rng));
                runner::run_experiment_for_instance::<_, _, _, (), ()>(
                    &instance,
                    &instance_path,
                    runs_per_instance,
                    algorithms,
                )
                .unwrap();

                graph = instance.0; // reclaim ownership of input graph for next iteration
            }
        }
    }
}

fn aggregate_sssd<G, I, ED, D>(options: &AggregationOptions, algorithms: &[AlgorithmType<G, I, D>])
where
    G: Graph<I, ED>,
    I: Index + SampleUniform,
    ED: EdgeData,
    OsmReader: GraphReader<G, I, ED, OsmReaderOptions>,
{
    for folder in OsmReader::get_folders() {
        super::aggregate::<_, _, _, ()>(&folder, algorithms, options, None);
    }
}
