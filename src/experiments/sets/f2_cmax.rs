use crate::{
    algorithms::scheduling::flow_shop::f2_cmax,
    data_generators::scheduling::flow_shop,
    experiments::{runner, InstanceGenerator},
};

use super::{AggregationOptions, ExperimentOptions, ExperimentSet};

pub fn experiment_set() -> ExperimentSet {
    ExperimentSet { run, aggregate }
}

const ALGORITHMS: [f2_cmax::AlgorithmType; 2] = [
    f2_cmax::ENUMERATE_WITH_IQS,
    f2_cmax::SOLVE_WITH_UNSTABLE_SORT,
];

fn run(options: &mut ExperimentOptions) {
    let job_numbers = [
        10_000,
        20_000,
        30_000,
        40_000,
        50_000,
        60_000,
        70_000,
        80_000,
        90_000,
        100_000,
        200_000,
        300_000,
        400_000,
        500_000,
        600_000,
        700_000,
        800_000,
        900_000,
        1_000_000,
        2_000_000,
        3_000_000,
        4_000_000,
        5_000_000,
        6_000_000,
        7_000_000,
        8_000_000,
        9_000_000,
        10_000_000,
        20_000_000,
        30_000_000,
        40_000_000,
        50_000_000,
        60_000_000,
        70_000_000,
        80_000_000,
        90_000_000,
        100_000_000,
        200_000_000,
        300_000_000,
        400_000_000,
        500_000_000,
        600_000_000,
        700_000_000,
        800_000_000,
        900_000_000,
        1_000_000_000,
        2_000_000_000,
        3_000_000_000,
    ];
    let instances_per_size = 10;
    let runs_per_instance = 5;
    let max_size = options.max_size;

    for jobs in job_numbers
        .into_iter()
        .filter(|&size| max_size.is_none_or(|max| size <= max))
    {
        log::info!("Run F2||C_max solver for {} jobs.", jobs);
        for i in 1..=instances_per_size {
            log::info!(
                "Solve instance {:2}/{:2} with {} jobs.",
                i,
                instances_per_size,
                jobs
            );
            let mut generator = flow_shop::Taillard { jobs, machines: 2 };

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

fn aggregate(options: &AggregationOptions) {
    let reference_algorithm = options
        .reference
        .as_ref()
        .and_then(|algo_name| ALGORITHMS.iter().find(|algo| algo.name() == algo_name));
    let folder = flow_shop::Taillard::path();
    super::aggregate::<_, _, _, ()>(&folder, &ALGORITHMS, options, reference_algorithm)
}
