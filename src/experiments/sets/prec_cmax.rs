use crate::{
    algorithms::scheduling::single_machine::prec_cmax,
    data_generators::scheduling::single_machine,
    experiments::{runner, InstanceGenerator},
};

use super::{AggregationOptions, ExperimentOptions, ExperimentSet};

pub fn experiment_set() -> ExperimentSet {
    ExperimentSet { run, aggregate }
}

const ALGORITHMS: [prec_cmax::AlgorithmType; 3] = [
    prec_cmax::ENUMERATE_WITH_TOPO_SORT,
    prec_cmax::SOLVE_WITH_IDFS_FINISH_TIME,
    prec_cmax::SOLVE_WITH_DFS_FINISH_TIME,
];

fn run(options: &mut ExperimentOptions) {
    let job_numbers = [
        100, 200, 300, 400, 500, 600, 700, 800, 900, 1_000, 2_000, 3_000, 4_000, 5_000, 6_000,
        7_000, 8_000, 9_000, 10_000, 20_000, 30_000, 40_000, 50_000, 60_000, 70_000, 80_000,
        90_000, 100_000, 200_000,
    ];
    let instances_per_size = 10;
    let runs_per_instance = 5;
    let limit_expected_edges = u32::MAX as f64 * 0.75;
    let max_size = options.max_size;

    for jobs in job_numbers
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
        edge_generation_parameters.push(((jobs as f64).sqrt() / jobs as f64, "n.sqrt(n)"));
        // expected order: n^1.25 = n sqrt(sqrt(n))
        edge_generation_parameters
            .push(((jobs as f64).powf(0.25) / jobs as f64, "n.sqrt(sqrt(n))"));
        // expected order: n log_2(n)
        edge_generation_parameters.push(((jobs as f64).log2() / jobs as f64, "n.log(n)"));
        // expected order: n log_10(n)
        edge_generation_parameters.push(((jobs as f64).log10() / jobs as f64, "n.ld(n)"));
        // expected order: n
        edge_generation_parameters.push((1.0 / jobs as f64, "n"));

        for (edge_probability, parameter_label) in edge_generation_parameters {
            let expected_edges = jobs as f64 * jobs as f64 * edge_probability;
            if expected_edges < 1.0 || expected_edges > limit_expected_edges {
                continue;
            }

            log::info!(
                "Run 1|prec|C_max solver for {} jobs and parameter {} (edge probability {}).",
                jobs,
                parameter_label,
                edge_probability,
            );
            for i in 1..=instances_per_size {
                log::info!(
                    "Solve instance {:2}/{:2} with {} jobs and parameter {} (edge probability {}).",
                    i,
                    instances_per_size,
                    jobs,
                    parameter_label,
                    edge_probability,
                );
                let mut generator = single_machine::WithPrecedences {
                    jobs,
                    edge_probability,
                    parameter_label: parameter_label.to_string(),
                };

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
    let reference_algorithm = options
        .reference
        .as_ref()
        .and_then(|algo_name| ALGORITHMS.iter().find(|algo| algo.name() == algo_name));
    let folder = single_machine::WithPrecedences::path();
    super::aggregate::<_, _, _, ()>(&folder, &ALGORITHMS, options, reference_algorithm)
}
