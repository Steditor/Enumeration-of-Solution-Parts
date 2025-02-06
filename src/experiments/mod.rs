pub mod aggregation;
pub mod runner;
pub mod sets;

use num::cast::AsPrimitive;
use std::{
    fmt::{Debug, Display},
    path::Path,
};

use aggregation::Aggregatable;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::io;

#[derive(Debug)]
pub struct CouldNotComputeError {
    pub reason: String,
}
impl Display for CouldNotComputeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.reason)
    }
}

pub type TotalTimeAlgorithm<Input, Output> = fn(&Input) -> Result<Output, CouldNotComputeError>;
pub type PreparedEnumerationAlgorithm<'a, Partial> = Box<dyn Iterator<Item = Partial> + 'a>;
pub type EnumerationAlgorithm<Input, Partial> = fn(&Input) -> PreparedEnumerationAlgorithm<Partial>;

pub enum ExperimentAlgorithm<Input: ?Sized, Partial, Output> {
    TotalTimeAlgorithm(&'static str, TotalTimeAlgorithm<Input, Output>),
    EnumerationAlgorithm(&'static str, EnumerationAlgorithm<Input, Partial>),
}

impl<Input, Partial, Output> ExperimentAlgorithm<Input, Partial, Output> {
    fn name(&self) -> &str {
        match self {
            ExperimentAlgorithm::TotalTimeAlgorithm(name, _) => name,
            ExperimentAlgorithm::EnumerationAlgorithm(name, _) => name,
        }
    }
}

pub trait Quality: Serialize + Default + Aggregatable + Debug {
    fn approximation_ratio_to(self, _: f64) -> f64 {
        f64::NAN
    }
}

macro_rules! impl_forward_div {
    ($($t:ty)*) => ($(
        impl Quality for $t {
            #[inline]
            fn approximation_ratio_to(self: Self, other: f64) -> f64 {
                AsPrimitive::<f64>::as_(self) / other
            }
        }
    )*)
}
impl_forward_div!(u8 i8 u16 i16 u32 i32 f32 u64 i64 f64 u128 i128);

impl Quality for () {}

pub trait ResultMetric<Input, Partial, Output, Q>
where
    Q: Quality,
{
    fn output_quality(input: &Input, output: &Output) -> Q;
    fn partials_quality(input: &Input, partials: &[Partial]) -> Q;
}

impl<Input, Partial, Output> ResultMetric<Input, Partial, Output, ()> for () {
    fn output_quality(_: &Input, _: &Output) {}
    fn partials_quality(_: &Input, _: &[Partial]) {}
}

pub trait InstanceGenerator<T> {
    /// Returns the canonical path for instances produced by this generator.
    ///
    /// The path is expected to have the form './data/{type}/{subtype}/'.
    fn path() -> String;

    /// Returns the canonical file name for a generated instance.
    ///
    /// The file name is expected to have the form '{size}_{parameter1[-parameter2[...]]}'.
    fn file_name(&self) -> String;

    fn generate(&self, seed: u64) -> T;
}

pub trait CachableInstanceGenerator<T: DeserializeOwned + Serialize>: InstanceGenerator<T> {
    fn generate_with_cache(&self, seed: u64) -> Result<T, io::IOError> {
        let mut experiment_path = Self::path();
        experiment_path.push_str(&self.file_name());
        experiment_path.push_str(&format!("_{}", seed));
        experiment_path.push_str(".json");
        let file_path = Path::new(&experiment_path);

        match io::json::read_json_from_file(file_path) {
            Err(why) => log::info!("Reading the instance from a file failed: {}", why),
            Ok(instance) => {
                log::info!("Successfully read instance from {}.", file_path.display());
                return Ok(instance);
            }
        }

        log::info!("Fall back to generating the instance.");
        let instance = self.generate(seed);

        log::info!("Writing instance to {}.", file_path.display());
        io::json::write_json_to_file(file_path, &instance)?;

        Ok(instance)
    }
}
impl<T, IG: InstanceGenerator<T>> CachableInstanceGenerator<T> for IG where
    T: DeserializeOwned + Serialize
{
}

pub trait InstanceTransformator<Input, Output> {
    fn new(instance: Input) -> Self;
}

#[derive(Serialize, Deserialize)]
pub struct QualityMeasurement<Q: Quality> {
    /// The quality of the output
    pub quality: Q,
}

#[derive(Serialize, Deserialize)]
pub struct TotalTimeMeasurement<Q: Quality> {
    /// The total time in ns
    pub total_time: u64,
    /// The quality of the output
    pub quality: Q,
}

#[derive(Serialize, Deserialize)]
pub struct EnumerationMeasurement<Q: Quality> {
    /// The total time in ns
    pub total_time: u64,
    /// The preprocessing time in ns
    pub preprocessing: u64,
    /// The time-to-first-output in ns
    pub first_output: u64,
    /// The total number of delays
    pub delays: usize,
    /// The minimum delay in ns
    pub delay_min: f64,
    /// The maximum delay in ns
    pub delay_max: f64,
    /// The average delay in ns
    pub delay_avg: f64,
    /// The maximum incremental delay in ns
    pub delay_inc_max: f64,
    /*
       We could also keep track of:
       - variance
       - all (?) or some random subset of delays
    */
    /// The quality of the output
    pub quality: Q,
}

pub trait HasQuality<Q: Quality> {
    fn quality(&self) -> Q;
}

macro_rules! impl_forward_hasQuality {
    ($($t:ty)*) => ($(
        impl<Q: Quality> HasQuality<Q> for $t {
            fn quality(&self) -> Q {
                self.quality
            }
        }
    )*)
}
impl_forward_hasQuality!(QualityMeasurement<Q> TotalTimeMeasurement<Q> EnumerationMeasurement<Q>);

pub trait StatisticsOutput: Serialize + DeserializeOwned {}
impl<T> StatisticsOutput for T where T: Serialize + DeserializeOwned {}

pub trait StatisticsCollector<Input, Statistics>
where
    Statistics: StatisticsOutput,
{
    fn collect_statistics(instance: &Input) -> Option<Statistics>;
}

impl<T> StatisticsCollector<T, ()> for () {
    fn collect_statistics(_: &T) -> Option<()> {
        None
    }
}
