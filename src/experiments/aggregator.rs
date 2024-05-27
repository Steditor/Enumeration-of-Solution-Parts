use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use num::{traits::AsPrimitive, Bounded, Float};
use serde::{de::DeserializeOwned, Deserialize, Serialize, Serializer};

use crate::{
    experiments::{EnumerationMeasurement, TotalTimeMeasurement},
    io::{self, IOError},
};

use super::{ExperimentAlgorithm, ExperimentGenerator};

// There doesn't seem to be a unified way of computing the min/max of
// two numbers (not even with the num package). So we add this ourselves.
pub trait MinMax {
    fn max(self, other: Self) -> Self;
    fn min(self, other: Self) -> Self;
}

macro_rules! impl_forward_min_max {
    ($target:ident, $($t:ty)*) => ($(
        impl MinMax for $t {
            #[inline]
            fn max(self, other: Self) -> Self {
                $target::max(self, other)
            }
            #[inline]
            fn min(self, other: Self) -> Self {
                $target::min(self, other)
            }
        }
    )*)
}
impl_forward_min_max!(Ord, u8 i8 u16 i16 u32 i32 u64 i64 u128 i128);
impl_forward_min_max!(Float, f32 f64);

// Aggregatable data can be averaged as f64, has a maximum and minimum value
// and provides max/min functions to find the larger/smaller of two values
pub trait Aggregatable: Copy + AsPrimitive<f64> + Bounded + MinMax {}
impl<T: Copy + AsPrimitive<f64> + Bounded + MinMax> Aggregatable for T {}

/// Aggregate data points by count, minimum, maximum and average
#[derive(Serialize, Deserialize)]
pub struct Aggregation<T: Aggregatable = u32> {
    /// The number of aggregated data points
    pub n: u32,
    /// The smallest observed data points
    pub min: T,
    /// The largest observed data points
    pub max: T,
    /// The average of all observed data points
    pub avg: f64,
}

impl<T: Aggregatable> Aggregation<T> {
    /// Create an empty aggregation
    ///
    /// "Empty" here means no measurements (`n=0`), placeholder values for min (`T::MAX`), max (`T::MIN`) and avg (`0.0`).
    pub fn new() -> Self {
        Self {
            n: 0,
            min: T::max_value(),
            max: T::min_value(),
            avg: 0.0,
        }
    }

    /// Push a new data point to the aggregation
    pub fn push(&mut self, value: T) {
        self.n += 1;

        self.max = self.max.max(value);
        self.min = self.min.min(value);

        if self.n == 1 {
            self.avg = value.as_();
        } else {
            self.avg += (value.as_() - self.avg) / self.n as f64;
        };
    }

    fn serialize_to_avg<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.avg)
    }
}

impl<T: Aggregatable> Default for Aggregation<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Default)]
pub struct TotalTimeAggregation {
    /// The instance size
    pub size: u32,
    /// The total computation time in ns
    #[serde(serialize_with = "Aggregation::serialize_to_avg")]
    pub total_time: Aggregation<u64>,
}

#[derive(Serialize, Default)]
pub struct EnumerationAggregation {
    /// The instance size
    pub size: u32,
    /// The total time in ns
    #[serde(serialize_with = "Aggregation::serialize_to_avg")]
    pub total_time: Aggregation<u64>,
    /// The preprocessing time in ns
    #[serde(serialize_with = "Aggregation::serialize_to_avg")]
    pub preprocessing: Aggregation<u64>,
    /// The time-to-first-output in ns
    #[serde(serialize_with = "Aggregation::serialize_to_avg")]
    pub first_output: Aggregation<u64>,
    /// The minimum delay time time in ns
    #[serde(serialize_with = "Aggregation::serialize_to_avg")]
    pub delay_min: Aggregation<u64>,
    /// The maximum delay time in ns
    #[serde(serialize_with = "Aggregation::serialize_to_avg")]
    pub delay_max: Aggregation<u64>,
    /// The preprocessing time in ns
    #[serde(serialize_with = "Aggregation::serialize_to_avg")]
    pub delay_avg: Aggregation<f64>,
}

/// Helper struct to parse a measurement path.
///
/// A measurement path has the form './data/{type}/{subtype}/{size}_{parameter1[-parameter2[...]]}_{RNG state id}.{algo}.csv'.
struct MeasurementFilePath {
    /// The complete path
    full_path: PathBuf,
    /// the instance size
    size: u32,
    /// the parameters
    parameters: String,
}

impl MeasurementFilePath {
    fn try_new(path: &Path, algorithm_name: &str) -> Option<Self> {
        let full_path = PathBuf::from(path);

        let mut editable_path = full_path.clone();
        editable_path
            .extension()
            .and_then(OsStr::to_str)
            .filter(|&e| e == "csv")?; // ensure extension is csv
        editable_path.set_extension(""); // remove 'csv' extension

        editable_path
            .extension()
            .and_then(OsStr::to_str)
            .filter(|&a| a == algorithm_name)?; // ensure algorithm is correct

        let mut parts = editable_path.file_stem()?.to_str()?.split('_');
        // first part of stem is the size
        let size = parts.next()?.parse::<u32>().ok()?;
        // second part of stem are the parameters
        let parameters = String::from(parts.next()?);
        // third part of stem is the RNG state id that we ignore in aggregation

        Some(Self {
            full_path,
            size,
            parameters,
        })
    }
}

pub fn aggregate<Generator, Input, Partial, Output>(
    algorithm: &ExperimentAlgorithm<Input, Partial, Output>,
) -> Result<(), IOError>
where
    Input: DeserializeOwned + Serialize,
    Generator: ExperimentGenerator<Input>,
{
    let folder = Generator::path();
    let folder = Path::new(&folder);

    let algorithm_name = match algorithm {
        ExperimentAlgorithm::EnumerationAlgorithm(name, _) => *name,
        ExperimentAlgorithm::TotalTimeAlgorithm(name, _) => *name,
    };

    let files = match folder.read_dir() {
        Err(why) => {
            return Err(IOError::CannotRead(
                folder.display().to_string(),
                why.to_string(),
            ))
        }
        Ok(files) => files
            // ignore directory entries that are errors
            .filter_map(Result::ok)
            // only look at files with proper file name including correct algorithm name
            .filter_map(|f| MeasurementFilePath::try_new(&f.path(), algorithm_name)),
    };

    match algorithm {
        ExperimentAlgorithm::EnumerationAlgorithm(algorithm_name, _) => {
            aggregate_enumeration_algorithm(files, folder, algorithm_name)
        }
        ExperimentAlgorithm::TotalTimeAlgorithm(algorithm_name, _) => {
            aggregate_total_time_algorithm(files, folder, algorithm_name)
        }
    }
}

fn aggregate_enumeration_algorithm(
    files: impl Iterator<Item = MeasurementFilePath>,
    folder: &Path,
    algorithm_name: &str,
) -> Result<(), IOError> {
    let mut aggregations_by_parameter = HashMap::new();
    files.for_each(|f| {
        let measurements = io::read_csv_from_file::<EnumerationMeasurement>(&f.full_path);
        match measurements {
            Err(why) => {
                log::info!("Could not read from {}: {}", f.full_path.display(), why)
            }
            Ok(measurements) => {
                let aggregation = aggregations_by_parameter
                    .entry(f.parameters)
                    .or_insert_with(HashMap::new)
                    .entry(f.size)
                    .or_insert_with(|| EnumerationAggregation {
                        size: f.size,
                        ..Default::default()
                    });
                for m in measurements {
                    aggregation.total_time.push(m.total_time);
                    aggregation.preprocessing.push(m.preprocessing);
                    aggregation.first_output.push(m.first_output);
                    aggregation.delay_min.push(m.delay_min);
                    aggregation.delay_max.push(m.delay_max);
                    aggregation.delay_avg.push(m.delay_avg);
                }
            }
        }
    });
    for (parameters, aggregations_by_size) in aggregations_by_parameter {
        let mut path = PathBuf::from(folder);
        path.push(format!("aggregated_{}.{}.csv", parameters, algorithm_name));
        let mut values = aggregations_by_size.values().collect::<Vec<_>>();
        values.sort_unstable_by_key(|v| v.size);
        io::append_csv_to_file(path.as_path(), &values)?;
    }
    Ok(())
}

fn aggregate_total_time_algorithm(
    files: impl Iterator<Item = MeasurementFilePath>,
    folder: &Path,
    algorithm_name: &str,
) -> Result<(), IOError> {
    let mut aggregations_by_parameter = HashMap::new();
    files.for_each(|f| {
        let measurements = io::read_csv_from_file::<TotalTimeMeasurement>(&f.full_path);
        match measurements {
            Err(why) => {
                log::info!("Could not read from {}: {}", f.full_path.display(), why)
            }
            Ok(measurements) => {
                let aggregation = aggregations_by_parameter
                    .entry(f.parameters)
                    .or_insert_with(HashMap::new)
                    .entry(f.size)
                    .or_insert_with(|| TotalTimeAggregation {
                        size: f.size,
                        ..Default::default()
                    });
                for m in measurements {
                    aggregation.total_time.push(m.total_time);
                }
            }
        }
    });
    for (parameters, aggregations_by_size) in aggregations_by_parameter {
        let mut path = PathBuf::from(folder);
        path.push(format!("aggregated_{}.{}.csv", parameters, algorithm_name));
        let mut values = aggregations_by_size.values().collect::<Vec<_>>();
        values.sort_unstable_by_key(|v| v.size);
        io::append_csv_to_file(path.as_path(), &values)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_measurement_aggregation() {
        let mut aggregation = Aggregation::new();

        vec![1, 7, 6, 3, 4, 9, 0, 5, 8, 2]
            .iter()
            .for_each(|x| aggregation.push(*x));

        assert_eq!(aggregation.n, 10);
        assert_eq!(aggregation.min, 0);
        assert_eq!(aggregation.max, 9);
        assert_eq!(aggregation.avg, 4.5);
    }
}
