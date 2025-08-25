use num::ToPrimitive;
use rand::distributions::Uniform;

use crate::{
    algorithms::sorting::{
        incremental_heap_sort::ENUMERATE_WITH_IHS, incremental_quick_sort::ENUMERATE_WITH_IQS,
        AlgorithmType,
    },
    data_generators::sorting,
    experiments::{
        runner,
        sets::{AggregationOptions, ExperimentOptions, ExperimentSet},
        ExperimentAlgorithm, InstanceGenerator,
    },
};

pub fn experiment_set() -> ExperimentSet {
    ExperimentSet { run, aggregate }
}

const ALGORITHMS: [AlgorithmType; 3] = [
    ENUMERATE_WITH_IQS,
    ENUMERATE_WITH_IHS,
    ExperimentAlgorithm::TotalTimeAlgorithm("total-rustsort", |vec| {
        let mut vec = vec.to_vec();
        vec.sort_unstable();
        Ok(vec)
    }),
];

fn run(options: &mut ExperimentOptions) {
    let array_sizes: Vec<_> = (1..=7)
        .flat_map(|m| (1..=9).map(move |x| x * 10_usize.pow(m)))
        .collect();

    let distributions = [("uniform", Uniform::new(0, u32::MAX))];

    let instances_per_size = 10;
    let runs_per_instance = 5;
    let max_size = options.max_size;

    for array_size in array_sizes
        .into_iter()
        .filter(|&size| max_size.is_none_or(|max| size <= max.to_usize().unwrap()))
    {
        for (distribution_name, distribution) in distributions {
            log::info!("Sort {array_size} elements with distribution {distribution_name}.");
            for i in 1..=instances_per_size {
                log::info!(
                    "Solve instance {i:2}/{instances_per_size:2} with {array_size} elements."
                );
                let mut generator = sorting::DistributedElements::new(
                    array_size,
                    distribution,
                    distribution_name.to_string(),
                );

                runner::run_cachable_experiment::<_, _, _, _, (), _, (), _>(
                    &mut generator,
                    options,
                    runs_per_instance,
                    &ALGORITHMS,
                )
                .unwrap()
            }
        }
    }
}

fn aggregate(options: &AggregationOptions) {
    let folder = sorting::DistributedElements::<u32, Uniform<u32>>::path();
    super::aggregate::<_, _, _, ()>(&folder, &ALGORITHMS, options, None)
}
