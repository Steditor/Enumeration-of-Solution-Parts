use serde::Serialize;

use super::{Aggregatable, Aggregation};

/// Aggregate data points by count, minimum, maximum and average without storing all data
#[derive(Serialize, Debug)]
pub struct StreamingAggregation {
    /// The number of aggregated data points
    pub n: usize,
    /// The smallest observed data points
    pub min: f64,
    /// The largest observed data points
    pub max: f64,
    /// The average of all observed data points
    pub avg: f64,
}

impl Default for StreamingAggregation {
    /// Create an empty aggregation
    ///
    /// "Empty" here means no measurements (`n=0`),
    /// placeholder values for min (`f64::MAX`), max (`f64::MIN`) and avg (`0.0`).
    fn default() -> Self {
        Self {
            n: 0,
            min: f64::MAX,
            max: f64::MIN,
            avg: 0.0,
        }
    }
}

impl Aggregation for StreamingAggregation {
    fn get_headers() -> Vec<String> {
        ["n", "min", "max", "avg"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Push a new data point to the aggregation
    fn push<T: Aggregatable>(&mut self, value: T) {
        self.n += 1;

        self.max = self.max.max(value.to_aggregatable());
        self.min = self.min.min(value.to_aggregatable());

        if self.n == 1 {
            self.avg = value.to_aggregatable();
        } else {
            self.avg += (value.to_aggregatable() - self.avg) / self.n as f64;
        };
    }

    fn avg(&mut self) -> f64 {
        self.avg
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_measurement_aggregation() {
        let mut aggregation = StreamingAggregation::default();

        [1, 7, 6, 3, 4, 9, 0, 5, 8, 2]
            .iter()
            .for_each(|x| aggregation.push(*x));

        assert_eq!(aggregation.n, 10);
        assert_eq!(aggregation.min, 0.0);
        assert_eq!(aggregation.max, 9.0);
        assert_eq!(aggregation.avg, 4.5);
    }
}
