pub mod f2_cmax;
pub mod prec_cmax;
pub mod rj_cmax;

use serde::{de::DeserializeOwned, Serialize};

use super::{aggregator, ExperimentAlgorithm, ExperimentGenerator};

#[derive(Debug, Clone, Copy)]
pub struct ExperimentOptions {
    pub max_size: Option<u32>,
    pub cache_instances: bool,
}

pub struct ExperimentSet {
    pub run: fn(options: ExperimentOptions),
    pub aggregate: fn(),
}

fn aggregate<Generator, Input, Partial, Output>(
    algorithms: &[ExperimentAlgorithm<Input, Partial, Output>],
) where
    Input: Serialize + DeserializeOwned,
    Generator: ExperimentGenerator<Input>,
{
    for algorithm in algorithms {
        aggregator::aggregate::<Generator, _, _, _>(algorithm).unwrap();
    }
}
