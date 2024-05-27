use std::{path::Path, slice, time::Instant};

use serde::{de::DeserializeOwned, Serialize};

use crate::io;

use rand::seq::SliceRandom;

use super::{
    aggregator::Aggregation, sets::ExperimentOptions, EnumerationAlgorithm, EnumerationMeasurement,
    ExperimentAlgorithm, ExperimentGenerator, TotalTimeAlgorithm, TotalTimeMeasurement,
};

pub fn run_experiment<Generator, Input, Partial, Output>(
    generator: &mut Generator,
    options: ExperimentOptions,
    number_of_runs: u32,
    algorithms: &[ExperimentAlgorithm<Input, Partial, Output>],
) -> Result<(), io::IOError>
where
    Input: DeserializeOwned + Serialize,
    Generator: ExperimentGenerator<Input>,
{
    let mut instance_path = Generator::path();
    instance_path.push_str(&generator.file_name());
    let instance = if options.cache_instances {
        generator.generate_with_cache()?
    } else {
        generator.generate()
    };

    log::info!("Run experiments for {}.", instance_path);

    let mut algorithms: Vec<_> = algorithms.iter().collect();
    let mut rng = rand::thread_rng();

    for run in 1..=number_of_runs {
        algorithms.shuffle(&mut rng);

        for algorithm in algorithms.iter() {
            match algorithm {
                ExperimentAlgorithm::TotalTimeAlgorithm(name, total_time_algorithm) => {
                    let measurement = run_total_time_algorithm(&instance, total_time_algorithm);
                    let result_file_name = format!("{}.{}.csv", instance_path, name);
                    io::append_csv_to_file(
                        Path::new(&result_file_name),
                        slice::from_ref(&measurement),
                    )?;
                }
                ExperimentAlgorithm::EnumerationAlgorithm(name, enumeration_algorithm) => {
                    let measurement = run_enumeration_algorithm(&instance, enumeration_algorithm);
                    let result_file_name = format!("{}.{}.csv", instance_path, name);
                    io::append_csv_to_file(
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

fn run_total_time_algorithm<Input, Output>(
    input: &Input,
    algorithm: &TotalTimeAlgorithm<Input, Output>,
) -> TotalTimeMeasurement
where
    Input: DeserializeOwned + Serialize,
{
    let start = Instant::now();
    algorithm(input);
    // overflow for ~584 years -> not relevant for us
    let total_time = start.elapsed().as_nanos() as u64;

    TotalTimeMeasurement { total_time }
}

fn run_enumeration_algorithm<Input, Partial>(
    input: &Input,
    algorithm: &EnumerationAlgorithm<Input, Partial>,
) -> EnumerationMeasurement
where
    Input: DeserializeOwned + Serialize,
{
    let mut first_output = 0;
    let mut delay_aggregation = Aggregation::new();

    let start = Instant::now();
    let enumeration_iterator = algorithm(input);

    // overflow for ~584 years -> not relevant for us
    let preprocessing = start.elapsed().as_nanos() as u64;

    let mut delay_start = Instant::now();
    for _ in enumeration_iterator {
        // overflow for ~584 years -> not relevant for us
        let delay = delay_start.elapsed().as_nanos() as u64;

        delay_aggregation.push(delay);

        if delay_aggregation.n == 1 {
            // overflow for ~584 years -> not relevant for us
            first_output = start.elapsed().as_nanos() as u64;
        }

        delay_start = Instant::now();
    }

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
    }
}
