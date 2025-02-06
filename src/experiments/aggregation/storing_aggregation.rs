use serde::Serialize;

use super::{Aggregatable, Aggregation};

/// Aggregate data points by count, minimum, maximum and average without storing all data
#[derive(Serialize, Debug)]
pub struct StoringAggregation {
    /// The data points that are to be aggregated
    #[serde(skip)]
    data_points: Vec<f64>,
    #[serde(skip)]
    dirty: bool,
    /// The number of aggregated data points
    pub n: Option<usize>,
    /// The smallest observed data points
    pub min: Option<f64>,
    /// The largest observed data points
    pub max: Option<f64>,
    /// The average of all observed data points
    pub avg: Option<f64>,
    /// The median of all observed data points;
    /// the average of the upper and lower median in case of an even number of data points
    pub median: Option<f64>,
    /// The lower quartile (median of the lower half)
    pub lower_quartile: Option<f64>,
    /// The upper quartiile (median of the upper half)
    pub upper_quartile: Option<f64>,
    // TODO: 1.5 IQR whiskers
}

impl Default for StoringAggregation {
    fn default() -> Self {
        Self {
            data_points: Vec::new(),
            dirty: true,
            n: None,
            min: None,
            max: None,
            avg: None,
            median: None,
            lower_quartile: None,
            upper_quartile: None,
        }
    }
}

impl Aggregation for StoringAggregation {
    fn get_headers() -> Vec<String> {
        [
            "n",
            "min",
            "max",
            "avg",
            "median",
            "lower_quartile",
            "upper_quartile",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }
    /// Push a new data point to the aggregation
    fn push<T: Aggregatable>(&mut self, value: T) {
        #[allow(clippy::eq_op)]
        if value != value {
            // this is actually true for NaN
            return; // filter out NaN values
        }
        self.data_points.push(value.to_aggregatable());
        self.dirty = true;
    }

    fn aggregate(&mut self) {
        if !self.dirty {
            return;
        }
        let n = self.data_points.len();
        self.n = Some(n);

        // We filter out NaN values above, thus we can use partial_cmp also for floats
        self.data_points.sort_by(|a, b| a.partial_cmp(b).unwrap());

        self.min = self.data_points.first().copied();
        self.max = self.data_points.last().copied();

        self.avg = Some(self.data_points.iter().sum::<f64>() / n as f64);

        let data = &self.data_points;
        self.median = median(data);

        self.lower_quartile = median(&data[..n / 2]);
        if n % 2 == 1 {
            self.upper_quartile = median(&data[n / 2 + 1..]);
        } else {
            self.upper_quartile = median(&data[n / 2..]);
        }
        self.dirty = false;
    }

    fn avg(&mut self) -> f64 {
        self.aggregate();
        self.avg.unwrap_or(0.0)
    }
}

fn median<T: Aggregatable>(sorted_slice: &[T]) -> Option<f64> {
    let n = sorted_slice.len();
    if n == 0 {
        None
    } else if n % 2 == 1 {
        Some(sorted_slice[n / 2].to_aggregatable())
    } else {
        let upper_median = sorted_slice[n / 2].to_aggregatable();
        let lower_median = sorted_slice[n / 2 - 1].to_aggregatable();
        Some((upper_median + lower_median) / 2.0)
    }
}

#[cfg(test)]
mod test {
    use float_cmp::assert_approx_eq;

    use super::*;

    #[test]
    fn test_measurement_aggregation_odd() {
        let mut aggregation = StoringAggregation::default();

        [6, 7, 15, 36, 39, 40, 41, 42, 43, 47, 49]
            .iter()
            .for_each(|x| aggregation.push(*x));
        aggregation.aggregate();

        assert_eq!(aggregation.n.unwrap(), 11);
        assert_eq!(aggregation.min.unwrap(), 6.0);
        assert_eq!(aggregation.max.unwrap(), 49.0);
        assert_approx_eq!(f64, aggregation.avg.unwrap(), 33.18181, epsilon = 1e-5);
        assert_approx_eq!(f64, aggregation.lower_quartile.unwrap(), 15.0, ulps = 2);
        assert_approx_eq!(f64, aggregation.median.unwrap(), 40.0, ulps = 2);
        assert_approx_eq!(f64, aggregation.upper_quartile.unwrap(), 43.0, ulps = 2);
    }

    #[test]
    fn test_measurement_aggregation_even() {
        let mut aggregation = StoringAggregation::default();

        [7, 15, 36, 39, 40, 41]
            .iter()
            .for_each(|x| aggregation.push(*x));
        aggregation.aggregate();

        assert_eq!(aggregation.n.unwrap(), 6);
        assert_eq!(aggregation.min.unwrap(), 7.0);
        assert_eq!(aggregation.max.unwrap(), 41.0);
        assert_approx_eq!(f64, aggregation.avg.unwrap(), 29.66666, epsilon = 1e-5);
        assert_approx_eq!(f64, aggregation.lower_quartile.unwrap(), 15.0, ulps = 2);
        assert_approx_eq!(f64, aggregation.median.unwrap(), 37.5, ulps = 2);
        assert_approx_eq!(f64, aggregation.upper_quartile.unwrap(), 40.0, ulps = 2);
    }

    #[test]
    fn test_dirty_aggregation() {
        let mut aggregation = StoringAggregation::default();
        assert_eq!(aggregation.n, None);
        assert_eq!(aggregation.min, None);
        aggregation.aggregate();
        assert_eq!(aggregation.n, Some(0));
        assert_eq!(aggregation.min, None);
        aggregation.push(2);
        aggregation.aggregate();
        assert_eq!(aggregation.n, Some(1));
        assert_eq!(aggregation.min, Some(2.0));
        aggregation.push(1);
        assert_eq!(aggregation.n, Some(1));
        assert_eq!(aggregation.min, Some(2.0));
        aggregation.aggregate();
        assert_eq!(aggregation.n, Some(2));
        assert_eq!(aggregation.min, Some(1.0));
    }
}
