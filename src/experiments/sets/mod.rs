pub mod apsd;
pub mod apsd_artificial;
pub mod f2_cmax;
pub mod lazy_array;
pub mod mst;
pub mod p_cmax;
pub mod prec_cmax;
pub mod rj_cmax;
pub mod sssd;
pub mod sssd_artificial;

use std::path::Path;

use rand::RngCore;
use serde::de::DeserializeOwned;

use super::{
    aggregation::{self, extract_reference_quality, StoringAggregation, StreamingAggregation},
    ExperimentAlgorithm, Quality,
};

pub struct ExperimentOptions {
    pub max_size: Option<u32>,
    pub cache_instances: bool,
    pub seed_generator: Box<dyn RngCore>,
    pub collect_statistics: bool,
    pub run_algorithms: bool,
}

pub struct AggregationOptions {
    pub offline: bool,
    pub reference: Option<String>,
}

pub struct ExperimentSet {
    pub run: fn(options: &mut ExperimentOptions),
    pub aggregate: fn(options: &AggregationOptions),
}

fn aggregate<Input, Partial, Output, Q>(
    folder: impl AsRef<Path>,
    algorithms: &[ExperimentAlgorithm<Input, Partial, Output>],
    options: &AggregationOptions,
    reference: Option<&ExperimentAlgorithm<Input, Partial, Output>>,
) where
    Q: Quality + DeserializeOwned,
{
    if let Some(reference_algorithm) = reference {
        extract_reference_quality::<_, _, _, Q>(&folder, reference_algorithm).unwrap();
    }

    for algorithm in algorithms {
        if options.offline {
            aggregation::aggregate::<StoringAggregation, _, _, _, Q>(&folder, algorithm).unwrap();
        } else {
            aggregation::aggregate::<StreamingAggregation, _, _, _, Q>(&folder, algorithm).unwrap();
        }
    }
}
