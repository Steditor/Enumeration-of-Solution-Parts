use crate::{
    data_structures::LazyArray,
    experiments::{runner, ExperimentAlgorithm, InstanceGenerator},
};

use super::{AggregationOptions, ExperimentOptions, ExperimentSet};

use num::ToPrimitive;
use rand::{
    distributions::{Distribution, Uniform},
    seq::SliceRandom,
    Rng, SeedableRng,
};
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};

pub fn experiment_set() -> ExperimentSet {
    ExperimentSet { run, aggregate }
}

type AlgorithmType = ExperimentAlgorithm<(usize, Vec<ArrayAccessOperation>), (), ()>;

const ALGORITHMS: [AlgorithmType; 4] = [
    ExperimentAlgorithm::TotalTimeAlgorithm("total-vec", |(array_size, operations)| {
        run_std_vec(*array_size, operations);
        Ok(())
    }),
    ExperimentAlgorithm::TotalTimeAlgorithm("total-lazy-array", |(array_size, operations)| {
        run_lazy_array(*array_size, operations);
        Ok(())
    }),
    ExperimentAlgorithm::TotalTimeAlgorithm("allocate-vec", |(array_size, operations)| {
        allocate_std_vec(*array_size, operations);
        Ok(())
    }),
    ExperimentAlgorithm::TotalTimeAlgorithm("allocate-lazy-array", |(array_size, operations)| {
        allocate_lazy_array(*array_size, operations);
        Ok(())
    }),
];

fn run(options: &mut ExperimentOptions) {
    let mut array_sizes: Vec<_> = (1..=7)
        .flat_map(|m| (1..=9).map(move |x| x * 10_usize.pow(m)))
        .collect();
    array_sizes.shuffle(&mut rand::thread_rng());

    let instances_per_size = 10;
    let runs_per_instance = 5;
    let max_size = options.max_size;

    for array_size in array_sizes
        .into_iter()
        .filter(|&size| max_size.is_none_or(|max| size <= max.to_usize().unwrap()))
    {
        let num_operations = 200_000_000;
        log::info!(
            "Perform {num_operations} get/set operations on arrays with length {array_size}."
        );

        {
            let mut pre_alloc = LazyArray::new(array_size);
            pre_alloc.set(0, 0);
            log::info!(
                "Pre-Allocation done, value {:?} in index 0.",
                pre_alloc.get(0)
            );
        }

        for i in 1..=instances_per_size {
            log::info!("Solve instance {i:2}/{instances_per_size:2} with {num_operations} operations on arrays of length {array_size}.");

            let mut generator =
                ArrayAccessGenerator::new(array_size, num_operations, format!("{num_operations}"));

            runner::run_cachable_experiment::<_, _, _, _, (), _, (), _>(
                &mut generator,
                options,
                runs_per_instance,
                &ALGORITHMS,
            )
            .unwrap();
        }
    }
}

fn aggregate(options: &AggregationOptions) {
    let folder = ArrayAccessGenerator::path();
    super::aggregate::<_, _, _, ()>(&folder, &ALGORITHMS, options, None)
}

fn run_std_vec(array_size: usize, operations: &[ArrayAccessOperation]) {
    let mut array = vec![None; array_size];

    for operation in operations {
        match operation {
            ArrayAccessOperation::Read { index } => {
                let _ = array.get(*index);
            }
            ArrayAccessOperation::Write { index, value } => {
                array[*index] = Some(*value);
            }
        }
    }
    log::trace!("{:?}", array.first()); // make sure the vector cannot be optimized away
}

fn allocate_std_vec(array_size: usize, operations: &[ArrayAccessOperation]) {
    let mut array = vec![None; array_size];

    // perform one operation to make sure that the vector has to be created
    if let Some(operation) = operations.first() {
        match operation {
            ArrayAccessOperation::Read { index } => {
                let _ = array.get(*index);
            }
            ArrayAccessOperation::Write { index, value } => {
                array[*index] = Some(*value);
            }
        }
    }

    log::trace!("{:?}", array.first()); // make sure the vector cannot be optimized away
}

fn run_lazy_array(array_size: usize, operations: &[ArrayAccessOperation]) {
    let mut array = LazyArray::new(array_size);

    for operation in operations {
        match operation {
            ArrayAccessOperation::Read { index } => {
                let _ = array.get(*index);
            }
            ArrayAccessOperation::Write { index, value } => {
                array.set(*index, *value);
            }
        }
    }
    log::trace!("{:?}", array.get(0)); // make sure the vector cannot be optimized away
}

fn allocate_lazy_array(array_size: usize, operations: &[ArrayAccessOperation]) {
    let mut array = LazyArray::new(array_size);

    // perform one operation to make sure that the vector has to be created
    if let Some(operation) = operations.first() {
        match operation {
            ArrayAccessOperation::Read { index } => {
                let _ = array.get(*index);
            }
            ArrayAccessOperation::Write { index, value } => {
                array.set(*index, *value);
            }
        }
    }

    log::trace!("{:?}", array.get(0)); // make sure the array cannot be optimized away
}

#[derive(Serialize, Deserialize)]
enum ArrayAccessOperation {
    Read { index: usize },
    Write { index: usize, value: u32 },
}

struct ArrayAccessGenerator {
    array_size: usize,
    num_operations: usize,
    parameter_label: String,
}

impl ArrayAccessGenerator {
    pub fn new(array_size: usize, num_operations: usize, parameter_label: String) -> Self {
        Self {
            array_size,
            num_operations,
            parameter_label,
        }
    }
}

impl InstanceGenerator<(usize, Vec<ArrayAccessOperation>)> for ArrayAccessGenerator {
    fn path() -> String {
        String::from("./data/array_access/")
    }

    fn file_name(&self) -> String {
        format!("{}_{}", self.array_size, self.parameter_label)
    }

    fn generate(&self, seed: u64) -> (usize, Vec<ArrayAccessOperation>) {
        let mut rng = Pcg64::seed_from_u64(seed);
        let mut operations = Vec::with_capacity(self.num_operations);

        let index_distribution = Uniform::new(0, self.array_size);
        let value_distribution = Uniform::new(0, u32::MAX);

        for _ in 0..self.num_operations {
            let index = index_distribution.sample(&mut rng);
            operations.push(match rng.gen_bool(0.5) {
                true => ArrayAccessOperation::Write {
                    index,
                    value: value_distribution.sample(&mut rng),
                },
                false => ArrayAccessOperation::Read { index },
            })
        }
        (self.array_size, operations)
    }
}
