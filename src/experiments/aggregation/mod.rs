mod aggregatable;
mod enumeration;
mod storing_aggregation;
mod streaming_aggregation;
mod total_time;

pub use storing_aggregation::StoringAggregation;
pub use streaming_aggregation::StreamingAggregation;

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub use aggregatable::Aggregatable;
use serde::{de::DeserializeOwned, Serialize};

use crate::io::{self, IOError};

use super::{
    EnumerationMeasurement, ExperimentAlgorithm, HasQuality, Quality, QualityMeasurement,
    TotalTimeMeasurement,
};

pub trait Aggregation: Default + Serialize + std::fmt::Debug {
    fn get_headers() -> Vec<String>;

    fn push<T: Aggregatable>(&mut self, value: T);
    fn aggregate(&mut self) {}
    fn avg(&mut self) -> f64;
}

/// Helper struct to parse a measurement path.
///
/// A measurement path has the form './data/{type}/{subtype}/{size}_{parameter1[-parameter2[...]]}_{RNG state id}.{algo}.csv'.
#[derive(Clone, Debug)]
struct MeasurementFilePath {
    /// The complete path
    full_path: PathBuf,
    /// the instance size
    size: u32,
    /// the parameters
    parameters: String,
}

static REFERENCE_ALGORITHM_NAME: &str = "reference-quality";

impl MeasurementFilePath {
    fn try_new(path: impl AsRef<Path>, algorithm_name: &str) -> Option<Self> {
        let full_path = PathBuf::from(path.as_ref());

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
        // but there has to be exactly one more part
        if parts.count() != 1 {
            return None;
        }

        Some(Self {
            full_path,
            size,
            parameters,
        })
    }

    fn to_algorithm(&self, algorithm_name: &str) -> Self {
        let mut full_path = self.full_path.clone();
        full_path.set_extension(""); // remove csv
        full_path.set_extension(format!("{algorithm_name}.csv")); // replace algorithm name and re-add csv
        Self::try_new(&full_path, algorithm_name)
            .expect("Replacing the algorithm in a valid MFP should always work.")
    }
}

pub fn aggregate<A, Input, Partial, Output, Q>(
    folder: impl AsRef<Path>,
    algorithm: &ExperimentAlgorithm<Input, Partial, Output>,
) -> Result<(), IOError>
where
    A: Aggregation,
    Q: Quality + DeserializeOwned,
{
    let folder = folder.as_ref();

    let mut files: Vec<_> = match folder.read_dir() {
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
            .filter_map(|f| MeasurementFilePath::try_new(f.path(), algorithm.name())),
    }
    .collect();

    // Group files by parameter
    files.sort_unstable_by(|a, b| a.parameters.cmp(&b.parameters));
    let file_sets = files.chunk_by(|a, b| a.parameters == b.parameters);

    match algorithm {
        ExperimentAlgorithm::EnumerationAlgorithm(algorithm_name, _) => {
            for file_set in file_sets {
                enumeration::aggregate::<A, Q>(file_set, folder, algorithm_name)?
            }
            Ok(())
        }
        ExperimentAlgorithm::TotalTimeAlgorithm(algorithm_name, _) => {
            for file_set in file_sets {
                total_time::aggregate::<A, Q>(file_set, folder, algorithm_name)?
            }
            Ok(())
        }
    }
}

pub fn extract_reference_quality<Input, Partial, Output, Q>(
    folder: impl AsRef<Path>,
    algorithm: &ExperimentAlgorithm<Input, Partial, Output>,
) -> Result<(), IOError>
where
    Q: Quality + DeserializeOwned,
{
    let folder = folder.as_ref();
    let files: Vec<_> = match folder.read_dir() {
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
            .filter_map(|f| MeasurementFilePath::try_new(f.path(), algorithm.name())),
    }
    .collect();

    let read_file = |f: &MeasurementFilePath| match algorithm {
        ExperimentAlgorithm::EnumerationAlgorithm(..) => {
            read_quality_from_file::<Q, EnumerationMeasurement<Q>>(f)
        }
        ExperimentAlgorithm::TotalTimeAlgorithm(..) => {
            read_quality_from_file::<Q, TotalTimeMeasurement<Q>>(f)
        }
    };

    for f in files {
        let measurements = match read_file(&f) {
            Ok(v) => v,
            Err(why) => {
                log::info!("Could not read from {}: {}", f.full_path.display(), why);
                continue;
            }
        };

        let mut quality_aggregation = StreamingAggregation::default();
        for measurement in measurements {
            quality_aggregation.push(measurement.to_aggregatable());
        }
        quality_aggregation.aggregate();

        let values = vec![QualityMeasurement {
            quality: quality_aggregation.avg(),
        }];

        let path = f.to_algorithm(REFERENCE_ALGORITHM_NAME);
        io::csv::write_to_file(
            path.full_path.as_path(),
            &values,
            io::csv::WriteMode::Replace,
            io::csv::HeaderMode::Auto,
        )?;
    }

    Ok(())
}

fn read_quality_from_file<Q: Quality, HQ: HasQuality<Q> + DeserializeOwned>(
    f: &MeasurementFilePath,
) -> Result<Vec<Q>, IOError> {
    io::csv::read_from_file::<HQ>(&f.full_path).map(|v| v.iter().map(|m| m.quality()).collect())
}

fn get_reference_quality(f: &MeasurementFilePath) -> Option<f64> {
    let comparison_f = f.to_algorithm(REFERENCE_ALGORITHM_NAME);

    match io::csv::read_from_file::<QualityMeasurement<f64>>(&comparison_f.full_path) {
        Ok(comparison_measurements) => {
            let mut quality = StreamingAggregation::default();
            comparison_measurements
                .iter()
                .for_each(|m| quality.push(m.quality));
            Some(quality.avg)
        }
        Err(why) => {
            log::debug!(
                "Could not read reference quality from {}: {}",
                comparison_f.full_path.display(),
                why
            );
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_measurement_file_path_correct() {
        let mfp = MeasurementFilePath::try_new(
            Path::new("./data/type/42_parameter1-parameter2_123456.my-algorithm.csv"),
            "my-algorithm",
        );
        assert!(mfp.is_some());
        let mfp = mfp.unwrap();
        assert_eq!(
            mfp.full_path.as_os_str(),
            "./data/type/42_parameter1-parameter2_123456.my-algorithm.csv"
        );
        assert_eq!(mfp.size, 42);
        assert_eq!(mfp.parameters, "parameter1-parameter2");
    }

    #[test]
    fn test_measurement_file_path_wrong_extension() {
        let mfp = MeasurementFilePath::try_new(
            Path::new("./data/type/42_parameter1-parameter2_123456.my-algorithm.txt"),
            "my-algorithm",
        );
        assert!(mfp.is_none());
    }

    #[test]
    fn test_measurement_file_path_wrong_algorithm() {
        let mfp = MeasurementFilePath::try_new(
            Path::new("./data/type/42_parameter1-parameter2_123456.my-algorithm.csv"),
            "my-other-algorithm",
        );
        assert!(mfp.is_none());
    }

    #[test]
    fn test_measurement_file_path_illegal_size() {
        let mfp = MeasurementFilePath::try_new(
            Path::new("./data/type/big_parameter1-parameter2_123456.my-algorithm.csv"),
            "my-algorithm",
        );
        assert!(mfp.is_none());
    }

    #[test]
    fn test_measurement_file_path_too_few_parts() {
        let mfp = MeasurementFilePath::try_new(
            Path::new("./data/type/42_123456.my-algorithm.csv"),
            "my-algorithm",
        );
        assert!(mfp.is_none());
    }

    #[test]
    fn test_measurement_file_path_too_many_parts() {
        let mfp = MeasurementFilePath::try_new(
            Path::new("./data/type/42_parameter1-parameter2_123456_more.my-algorithm.csv"),
            "my-algorithm",
        );
        assert!(mfp.is_none());
    }

    #[test]
    fn test_measurement_file_path_replace_algorithm() {
        let mfp = MeasurementFilePath::try_new(
            Path::new("./data/type/42_parameter1-parameter2_123456.my-algorithm.csv"),
            "my-algorithm",
        )
        .map(|mfp| mfp.to_algorithm("other-algorithm"));

        assert!(mfp.is_some());
        let mfp = mfp.unwrap();
        assert_eq!(
            mfp.full_path.as_os_str(),
            "./data/type/42_parameter1-parameter2_123456.other-algorithm.csv"
        );
        assert_eq!(mfp.size, 42);
        assert_eq!(mfp.parameters, "parameter1-parameter2");
    }
}
