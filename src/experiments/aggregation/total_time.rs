use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    experiments::{Quality, TotalTimeMeasurement},
    io::{self, IOError},
};

use super::{get_reference_quality, Aggregation, MeasurementFilePath};

#[derive(Default, Serialize, Debug)]
pub struct TotalTimeAggregation<A: Aggregation> {
    /// The instance size
    pub size: u32,
    /// The total computation time in ns
    pub total_time: A,
    /// The approximation ratio
    pub approximation_ratio: A,
}

impl<A: Aggregation> TotalTimeAggregation<A> {
    fn from_measurements<Q: Quality>(
        measurements: &[TotalTimeMeasurement<Q>],
        reference_quality: Option<f64>,
    ) -> Self {
        let mut tta = Self::default();
        measurements
            .iter()
            .for_each(|m| tta.push_measurement(m, reference_quality));
        tta.aggregate();
        tta
    }

    fn push_measurement<Q: Quality>(
        &mut self,
        measurement: &TotalTimeMeasurement<Q>,
        reference_quality: Option<f64>,
    ) {
        self.total_time.push(measurement.total_time);

        if let Some(rq) = reference_quality {
            self.approximation_ratio
                .push(measurement.quality.approximation_ratio_to(rq));
        }
    }

    fn push_aggregation(&mut self, aggregation: &mut Self) {
        self.total_time.push(aggregation.total_time.avg());
        self.approximation_ratio
            .push(aggregation.approximation_ratio.avg());
    }

    fn aggregate(&mut self) {
        self.total_time.aggregate();
        self.approximation_ratio.aggregate();
    }
}

pub fn aggregate<A: Aggregation, Q: Quality + DeserializeOwned>(
    files: &[MeasurementFilePath],
    folder: &Path,
    algorithm_name: &str,
) -> Result<(), IOError> {
    let mut aggregations_by_size: HashMap<u32, TotalTimeAggregation<A>> = HashMap::new();
    for f in files {
        let measurements = match io::csv::read_from_file::<TotalTimeMeasurement<Q>>(&f.full_path) {
            Ok(v) => v,
            Err(why) => {
                log::info!("Could not read from {}: {}", f.full_path.display(), why);
                continue;
            }
        };

        let reference_quality = get_reference_quality(f);

        // aggregate all measurements for this instance
        let mut instance_aggregation =
            TotalTimeAggregation::<A>::from_measurements(&measurements, reference_quality);

        // add the average of this instance's measurements to the aggregations for this input size
        let size_aggregation =
            aggregations_by_size
                .entry(f.size)
                .or_insert_with(|| TotalTimeAggregation {
                    size: f.size,
                    ..Default::default()
                });
        size_aggregation.push_aggregation(&mut instance_aggregation);
    }
    aggregations_by_size
        .iter_mut()
        .for_each(|(_, a)| a.aggregate());

    let mut path = PathBuf::from(folder);
    path.push(format!(
        "aggregated_{}.{}.csv",
        files[0].parameters, algorithm_name
    ));

    let mut values: Vec<_> = aggregations_by_size.values().collect();
    values.sort_unstable_by_key(|v| v.size);

    let mut headers = vec!["size".to_string()];
    let af_headers = A::get_headers();
    for field in ["total_time", "approximation_ratio"] {
        for header in &af_headers {
            headers.push(format!("{field}_{}", header.as_str()));
        }
    }
    io::csv::write_to_file(
        path.as_path(),
        &[headers],
        io::csv::WriteMode::Replace,
        io::csv::HeaderMode::None,
    )?;

    io::csv::write_to_file(
        path.as_path(),
        &values,
        io::csv::WriteMode::Append,
        io::csv::HeaderMode::None,
    )?;

    Ok(())
}
