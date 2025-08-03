use num::Unsigned;

use crate::{
    algorithms::graphs::shortest_distances::apsd::{
        unweighted, unweighted_no_self, unweighted_sorted, weighted, weighted_no_self,
        weighted_sorted, AlgorithmType,
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

const fn unweighted_algorithms<G, I>() -> [AlgorithmType<G, I>; 5]
where
    G: Graph<I>,
    I: Index,
{
    [
        unweighted::algorithm_bfs(),
        unweighted::algorithm_bfs_visitor(),
        unweighted::algorithm_enum_bfs(),
        unweighted_no_self::algorithm_enum_bfs(),
        unweighted_sorted::algorithm_enum_bfs(),
    ]
}

pub fn unweighted_experiment_set() -> ExperimentSet {
    ExperimentSet {
        run: |options| {
            run_apsd(
                options,
                &unweighted_algorithms::<UndirectedAdjacencyArrayGraph<u32>, _>(),
            )
        },
        aggregate: |options| {
            aggregate_apsd(
                options,
                &unweighted_algorithms::<UndirectedAdjacencyArrayGraph<u32>, _>(),
            )
        },
    }
}

// Weighted Graphs

const fn weighted_algorithms<G, I, EW>() -> [AlgorithmType<G, I, EW>; 5]
where
    G: Graph<I, EW>,
    I: Index,
    EW: EdgeWeight + Unsigned,
{
    [
        weighted::algorithm_dijkstra(),
        weighted::algorithm_floyd_warshall(),
        weighted::algorithm_enum_dijkstra(),
        weighted_no_self::algorithm_enum_dijkstra(),
        weighted_sorted::algorithm_enum_dijkstra(),
    ]
}

pub fn weighted_experiment_set() -> ExperimentSet {
    ExperimentSet {
        run: |options| {
            run_apsd(
                options,
                &weighted_algorithms::<UndirectedAdjacencyArrayGraph<u32, u32>, _, _>(),
            )
        },
        aggregate: |options| {
            aggregate_apsd(
                options,
                &weighted_algorithms::<UndirectedAdjacencyArrayGraph<u32, u32>, _, _>(),
            )
        },
    }
}

/// Run the given algorithms on OpenStreetMap graphs (unweighted or weighted, depending on the algorithms and `G`).
fn run_apsd<G, I, ED, D>(options: &mut ExperimentOptions, algorithms: &[AlgorithmType<G, I, D>])
where
    G: Graph<I, ED>,
    I: Index + Aggregatable,
    ED: EdgeData,
    OsmReader: GraphReader<G, I, ED, OsmReaderOptions>,
{
    let instances = GraphSetIterator::<OsmReader, G, I, ED, OsmReaderOptions>::new(
        OsmReader::get_file_paths(),
        true,
        OsmReaderOptions::new()
            .with_max_size(options.max_size)
            .require_tag("highway", "motorway")
            .require_tag("highway", "motorway_link")
            .merge_ways(true),
    );

    let runs_per_instance = 5;

    for GraphSetEntry { graph, path, .. } in instances {
        // path/size_parameters
        let instance_prefix = format!(
            "{}/{}_osm",
            path.parent()
                .expect("input files must have a proper parent folder")
                .display(),
            graph.num_vertices()
        );

        let instance_path = format!("{instance_prefix}_0");
        log::info!(
            "Solve instance with {} vertices and {} edges from osm file {}.",
            graph.num_vertices(),
            graph.num_edges(),
            path.display(),
        );

        if options.collect_statistics {
            runner::collect_statistics_for_instance::<GraphStatisticsCollector<_, _, _>, _, _>(
                &graph,
                &instance_path,
            )
            .unwrap();
        }

        if options.run_algorithms {
            runner::run_experiment_for_instance::<_, _, _, (), ()>(
                &graph,
                &instance_path,
                runs_per_instance,
                algorithms,
            )
            .unwrap();
        }
    }
}

fn aggregate_apsd<G, I, ED, D>(options: &AggregationOptions, algorithms: &[AlgorithmType<G, I, D>])
where
    G: Graph<I, ED>,
    I: Index,
    ED: EdgeData,
    OsmReader: GraphReader<G, I, ED, OsmReaderOptions>,
{
    for folder in OsmReader::get_folders() {
        super::aggregate::<_, _, _, ()>(&folder, algorithms, options, None);
    }
}
