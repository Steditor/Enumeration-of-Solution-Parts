use crate::{
    algorithms::scheduling::single_machine::prec_cmax,
    experiments::runner,
    random_generators::{
        numbers::{Rng, TaillardLCG},
        scheduling::single_machine,
    },
};

use super::{ExperimentOptions, ExperimentSet};

pub fn experiment_set() -> ExperimentSet {
    ExperimentSet { run, aggregate }
}

const ALGORITHMS: [prec_cmax::AlgorithmType; 3] = [
    prec_cmax::ENUMERATE_WITH_TOPO_SORT,
    prec_cmax::SOLVE_WITH_IDFS_FINISH_TIME,
    prec_cmax::SOLVE_WITH_DFS_FINISH_TIME,
];

fn run(options: ExperimentOptions) {
    let job_numbers = [
        100, 200, 300, 400, 500, 600, 700, 800, 900, 1_000, 2_000, 3_000, 4_000, 5_000, 6_000,
        7_000, 8_000, 9_000, 10_000, 20_000, 30_000, 40_000, 50_000, 60_000, 70_000, 80_000,
        90_000, 100_000, 200_000, 300_000,
    ];
    let instances_per_size = 10;
    let runs_per_instance = 5;
    let limit_expected_edges = u32::MAX as f64 * 0.75;

    let mut seed_rng = TaillardLCG::from_seed(42);

    for jobs in job_numbers
        .into_iter()
        .filter(|&size| !options.max_size.is_some_and(|max| size > max))
    {
        // expected order of edges: n^2 with different constants
        let mut edge_probabilities = vec![1.0 / 4.0, 1.0 / 8.0, 1.0 / 16.0, 1.0 / 32.0];
        // expected order: n^1.5 = n sqrt(n)
        edge_probabilities.push((jobs as f64).sqrt() / jobs as f64);
        // expected order: n^1.25 = n sqrt(sqrt(n))
        edge_probabilities.push((jobs as f64).powf(0.25) / jobs as f64);
        // expected order: n log_2(n)
        edge_probabilities.push((jobs as f64).log2() / jobs as f64);
        // expected order: n log_10(n)
        edge_probabilities.push((jobs as f64).log10() / jobs as f64);
        // expected order: n
        edge_probabilities.push(1.0 / jobs as f64);

        for edge_probability in edge_probabilities {
            let expected_edges = jobs as f64 * jobs as f64 * edge_probability;
            if expected_edges < 1.0 || expected_edges > limit_expected_edges {
                continue;
            }

            log::info!(
                "Run 1|prec|C_max solver for {} jobs and edge probability {}.",
                jobs,
                edge_probability
            );
            for i in 1..=instances_per_size {
                log::info!(
                    "Solve instance {:2}/{:2} with {} jobs and edge probability {}.",
                    i,
                    instances_per_size,
                    jobs,
                    edge_probability
                );
                let mut instance_rng = TaillardLCG::from_seed(seed_rng.next_seed());
                let mut generator = single_machine::WithPrecedences {
                    rng: &mut instance_rng,
                    jobs,
                    edge_probability,
                };

                runner::run_experiment(&mut generator, options, runs_per_instance, &ALGORITHMS)
                    .unwrap();
            }
        }
    }
}

fn aggregate() {
    super::aggregate::<single_machine::WithPrecedences, _, _, _>(&ALGORITHMS)
}
