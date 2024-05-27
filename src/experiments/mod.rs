pub mod aggregator;
pub mod runner;
pub mod sets;

use std::path::Path;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::io;

pub type TotalTimeAlgorithm<Input, Output> = fn(&Input) -> Output;
pub type PreparedEnumerationAlgorithm<'a, Partial> = Box<dyn Iterator<Item = Partial> + 'a>;
pub type EnumerationAlgorithm<Input, Partial> =
    fn(&Input) -> PreparedEnumerationAlgorithm<'_, Partial>;

pub enum ExperimentAlgorithm<Input, Partial, Output> {
    TotalTimeAlgorithm(&'static str, TotalTimeAlgorithm<Input, Output>),
    EnumerationAlgorithm(&'static str, EnumerationAlgorithm<Input, Partial>),
}

pub trait ExperimentGenerator<T: DeserializeOwned + Serialize> {
    /// Returns the canonical path for instances produced by this generator.
    ///
    /// The path is expected to have the form './data/{type}/{subtype}/'.
    fn path() -> String;

    /// Returns the canonical file name for the next generated instance as string.
    ///
    /// The file name is expected to have the form '{size}_{parameter1[-parameter2[...]]}_{RNG state id}' and should not include a file extension.
    ///
    /// **Careful: The returned file name usually changes after an instance was generated due to a changed RNG state.**
    fn file_name(&self) -> String;

    fn generate(&mut self) -> T;

    fn generate_with_cache(&mut self) -> Result<T, io::IOError> {
        let mut experiment_path = Self::path();
        experiment_path.push_str(&self.file_name());
        experiment_path.push_str(".json");
        let file_path = Path::new(&experiment_path);

        match io::read_json_from_file(Path::new(&file_path)) {
            Err(why) => log::info!("Reading the instance from a file failed: {}", why),
            Ok(instance) => {
                log::info!("Successfully read instance from {}.", file_path.display());
                return Ok(instance);
            }
        }

        log::info!("Fall back to generating the instance.");
        let instance = self.generate();

        log::info!("Writing instance to {}.", file_path.display());
        io::write_json_to_file(file_path, &instance)?;

        Ok(instance)
    }
}

#[derive(Serialize, Deserialize)]
pub struct TotalTimeMeasurement {
    /// The total time in ns
    pub total_time: u64,
}

#[derive(Serialize, Deserialize)]
pub struct EnumerationMeasurement {
    /// The total time in ns
    pub total_time: u64,
    /// The preprocessing time in ns
    pub preprocessing: u64,
    /// The time-to-first-output in ns
    pub first_output: u64,
    /// The total number of delays
    pub delays: u32,
    /// The minimum delay in ns
    pub delay_min: u64,
    /// The maximum delay in ns
    pub delay_max: u64,
    /// The average delay in ns
    pub delay_avg: f64,
    /*
       We could also keep track of:
       - variance
       - all (?) or some random subset of delays
    */
}
