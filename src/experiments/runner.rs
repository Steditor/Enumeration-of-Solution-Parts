use std::{path::Path, slice, time::Instant};

use rand::{seq::SliceRandom, RngCore};
use serde::{de::DeserializeOwned, Serialize};

use crate::io::{self, IOError};

use super::{
    aggregation::{Aggregation, StreamingAggregation},
    sets::ExperimentOptions,
    CachableInstanceGenerator, CouldNotComputeError, EnumerationAlgorithm, EnumerationMeasurement,
    ExperimentAlgorithm, InstanceGenerator, Quality, ResultMetric, StatisticsCollector,
    StatisticsOutput, TotalTimeAlgorithm, TotalTimeMeasurement,
};

pub fn run_experiment<Generator, Input, Partial, Output, Collector, Statistics, Metric, Q>(
    generator: &mut Generator,
    options: &mut ExperimentOptions,
    number_of_runs: u32,
    algorithms: &[ExperimentAlgorithm<Input, Partial, Output>],
) -> Result<(), io::IOError>
where
    Generator: InstanceGenerator<Input>,
    Collector: StatisticsCollector<Input, Statistics>,
    Statistics: StatisticsOutput,
    Metric: ResultMetric<Input, Partial, Output, Q>,
    Q: Quality,
{
    let seed = options.seed_generator.next_u64();
    let mut instance_path = Generator::path();
    instance_path.push_str(&generator.file_name());
    instance_path.push_str(&format!("_{}", seed));
    let instance = generator.generate(seed);

    if options.collect_statistics {
        collect_statistics_for_instance::<Collector, _, _>(&instance, &instance_path)?;
    }

    if options.run_algorithms {
        run_experiment_for_instance::<_, _, _, Metric, _>(
            &instance,
            &instance_path,
            number_of_runs,
            algorithms,
        )?;
    }

    Ok(())
}

pub fn run_cachable_experiment<
    Generator,
    Input,
    Partial,
    Output,
    Collector,
    Statistics,
    Metric,
    Q,
>(
    generator: &mut Generator,
    options: &mut ExperimentOptions,
    number_of_runs: u32,
    algorithms: &[ExperimentAlgorithm<Input, Partial, Output>],
) -> Result<(), io::IOError>
where
    Input: DeserializeOwned + Serialize,
    Generator: CachableInstanceGenerator<Input>,
    Collector: StatisticsCollector<Input, Statistics>,
    Statistics: StatisticsOutput,
    Metric: ResultMetric<Input, Partial, Output, Q>,
    Q: Quality,
{
    run_cachable_experiment_with_input_transform::<_, _, _, _, _, _, Collector, _, Metric, _>(
        generator,
        options,
        |x| x,
        number_of_runs,
        algorithms,
    )
}

pub fn run_cachable_experiment_with_input_transform<
    Generator,
    CachableInput,
    Transformer,
    Input,
    Partial,
    Output,
    Collector,
    Statistics,
    Metric,
    Q,
>(
    generator: &mut Generator,
    options: &mut ExperimentOptions,
    transformer: Transformer,
    number_of_runs: u32,
    algorithms: &[ExperimentAlgorithm<Input, Partial, Output>],
) -> Result<(), io::IOError>
where
    Generator: CachableInstanceGenerator<CachableInput>,
    CachableInput: DeserializeOwned + Serialize,
    Transformer: Fn(CachableInput) -> Input,
    Collector: StatisticsCollector<CachableInput, Statistics>,
    Statistics: StatisticsOutput,
    Metric: ResultMetric<Input, Partial, Output, Q>,
    Q: Quality,
{
    let seed = options.seed_generator.next_u64();
    let mut instance_path = Generator::path();
    instance_path.push_str(&generator.file_name());
    instance_path.push_str(&format!("_{}", seed));
    let instance = if options.cache_instances {
        generator.generate_with_cache(seed)?
    } else {
        generator.generate(seed)
    };

    if options.collect_statistics {
        collect_statistics_for_instance::<Collector, _, _>(&instance, &instance_path)?;
    }

    if options.run_algorithms {
        run_experiment_for_instance::<_, _, _, Metric, _>(
            &transformer(instance),
            &instance_path,
            number_of_runs,
            algorithms,
        )?;
    }
    Ok(())
}

pub fn collect_statistics_for_instance<Collector, Input, Statistics>(
    instance: &Input,
    instance_path: impl AsRef<str>,
) -> Result<(), IOError>
where
    Collector: StatisticsCollector<Input, Statistics>,
    Statistics: StatisticsOutput,
{
    let instance_path = instance_path.as_ref();
    log::info!("Collect statistics for {instance_path}.");

    let statistics = match Collector::collect_statistics(instance) {
        Some(s) => s,
        None => return Ok(()), // Not everyone has to collect statistics
    };
    let result_file_name = format!("{instance_path}.statistics.csv");
    io::csv::write_to_file(
        Path::new(&result_file_name),
        slice::from_ref(&statistics),
        io::csv::WriteMode::Replace,
        io::csv::HeaderMode::Auto,
    )
}

pub fn run_experiment_for_instance<Input, Partial, Output, Metric, Q>(
    instance: &Input,
    instance_path: impl AsRef<str>,
    number_of_runs: u32,
    algorithms: &[ExperimentAlgorithm<Input, Partial, Output>],
) -> Result<(), io::IOError>
where
    Metric: ResultMetric<Input, Partial, Output, Q>,
    Q: Quality,
{
    let instance_path = instance_path.as_ref();
    log::info!("Run experiments for {}.", instance_path);

    let mut algorithms: Vec<_> = algorithms.iter().collect();
    let mut rng = rand::thread_rng();

    for run in 1..=number_of_runs {
        algorithms.shuffle(&mut rng);

        for algorithm in algorithms.iter() {
            match algorithm {
                ExperimentAlgorithm::TotalTimeAlgorithm(name, total_time_algorithm) => {
                    let measurement = match run_total_time_algorithm::<_, _, _, Metric, _>(
                        instance,
                        total_time_algorithm,
                    ) {
                        Ok(m) => m,
                        Err(why) => {
                            log::error!("Algorithm {name} did not yield a result: {why}");
                            continue;
                        }
                    };
                    let result_file_name = format!("{}.{}.csv", instance_path, name);
                    io::csv::append_to_file(
                        Path::new(&result_file_name),
                        slice::from_ref(&measurement),
                    )?;
                }
                ExperimentAlgorithm::EnumerationAlgorithm(name, enumeration_algorithm) => {
                    let measurement = run_enumeration_algorithm::<_, _, _, Metric, _>(
                        instance,
                        enumeration_algorithm,
                    );
                    let result_file_name = format!("{}.{}.csv", instance_path, name);
                    io::csv::append_to_file(
                        Path::new(&result_file_name),
                        slice::from_ref(&measurement),
                    )?;
                }
            }
        }

        log::info!("{:2}/{:2}", run, number_of_runs);
    }

    log::info!("Finished experiments for {}.", instance_path);

    Ok(())
}

fn run_total_time_algorithm<Input, Partial, Output, Metric, Q>(
    input: &Input,
    algorithm: &TotalTimeAlgorithm<Input, Output>,
) -> Result<TotalTimeMeasurement<Q>, CouldNotComputeError>
where
    Metric: ResultMetric<Input, Partial, Output, Q>,
    Q: Quality,
{
    let start = Instant::now();
    let output = algorithm(input)?;
    // overflow for ~584 years -> not relevant for us
    let total_time = start.elapsed().as_nanos() as u64;

    Ok(TotalTimeMeasurement {
        total_time,
        quality: Metric::output_quality(input, &output),
    })
}

fn run_enumeration_algorithm<Input, Partial, Output, Metric, Q>(
    input: &Input,
    algorithm: &EnumerationAlgorithm<Input, Partial>,
) -> EnumerationMeasurement<Q>
where
    Metric: ResultMetric<Input, Partial, Output, Q>,
    Q: Quality,
{
    let mut first_output = 0;
    let mut delay_aggregation = StreamingAggregation::default();
    let mut delay_inc_max = 0.0;

    let start = Instant::now();
    let enumeration_iterator = algorithm(input);

    // overflow for ~584 years -> not relevant for us
    let preprocessing = start.elapsed().as_nanos() as u64;

    let enumeration_start = Instant::now();

    let mut partials = Vec::new();
    let mut delay_start = Instant::now();
    for partial in enumeration_iterator {
        // overflow for ~584 years -> not relevant for us
        let delay = delay_start.elapsed().as_nanos() as u64;
        partials.push(partial);

        delay_aggregation.push(delay);

        if delay_aggregation.n == 1 {
            // overflow for ~584 years -> not relevant for us
            first_output = start.elapsed().as_nanos() as u64;
        }

        delay_inc_max = f64::max(
            delay_inc_max,
            // overflow not relevant for us; precision loss is acceptable
            enumeration_start.elapsed().as_nanos() as f64 / delay_aggregation.n as f64,
        );

        delay_start = Instant::now();
    }

    // todo: inc_max with n+1 to capture "post-processing"?

    // overflow for ~584 years -> not relevant for us
    let total_time = start.elapsed().as_nanos() as u64;

    EnumerationMeasurement {
        total_time,
        preprocessing,
        first_output,
        delays: delay_aggregation.n,
        delay_min: delay_aggregation.min,
        delay_max: delay_aggregation.max,
        delay_avg: delay_aggregation.avg,
        delay_inc_max,
        quality: Metric::partials_quality(input, &partials),
    }
}
