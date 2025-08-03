use rand::distributions::Uniform;

use crate::{
    algorithms::scheduling::parallel_machines::cmax,
    data_generators::scheduling::parallel_machines,
    experiments::{runner, InstanceGenerator},
};

use super::{AggregationOptions, ExperimentOptions, ExperimentSet};

pub fn experiment_set() -> ExperimentSet {
    ExperimentSet { run, aggregate }
}

const ALGORITHMS: [cmax::AlgorithmType; 3] = [
    cmax::APPROXIMATE_WITH_LPT,
    cmax::ENUMERATE_WITH_LPT,
    cmax::ENUMERATE_WITH_LPT_COROUTINE,
];

fn run(options: &mut ExperimentOptions) {
    let job_numbers = (1..=8).flat_map(|m| (1..=9).map(move |x| x * 10_u32.pow(m)));
    let machine_numbers = (1..=9)
        .flat_map(|x| (0..=3).map(move |m| x * 10_u32.pow(m)))
        .skip(1); // 1 machine is boring

    let instances_per_size = 10;
    let runs_per_instance = 5;
    let max_size = options.max_size;

    for jobs in job_numbers.filter(|&size| max_size.is_none_or(|max| size <= max)) {
        for machines in machine_numbers.clone().filter(|m| m < &jobs) {
            log::info!("Run P||C_max approximation for {jobs} jobs on {machines} machines.");
            for i in 1..=instances_per_size {
                log::info!(
                    "Solve instance {i:2}/{instances_per_size:2} with {jobs} jobs on {machines} machines."
                );
                let mut generator = parallel_machines::Plain {
                    jobs,
                    machines,
                    distribution: &Uniform::new_inclusive(1, jobs / machines),
                };

                runner::run_cachable_experiment::<_, _, _, _, (), _, cmax::Metric, _>(
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
    let folder = parallel_machines::Plain::path();
    super::aggregate::<_, _, _, u64>(&folder, &ALGORITHMS, options, reference_algorithm)
}
