use std::marker::PhantomData;

use rand::{prelude::Distribution, SeedableRng};
use rand_pcg::Pcg64;

use crate::experiments::InstanceGenerator;

pub type SortingInstance<T> = Vec<T>;

pub struct DistributedElements<T, D: Distribution<T>> {
    num_elements: usize,
    element_generator: D,
    parameter_label: String,
    _phantom: PhantomData<T>,
}

impl<T, D: Distribution<T>> DistributedElements<T, D> {
    pub fn new(num_elements: usize, element_generator: D, parameter_label: String) -> Self {
        Self {
            num_elements,
            element_generator,
            parameter_label,
            _phantom: PhantomData,
        }
    }
}

impl<T, D: Distribution<T>> InstanceGenerator<SortingInstance<T>> for DistributedElements<T, D> {
    fn path() -> String {
        String::from("./data/arrays/")
    }

    fn file_name(&self) -> String {
        format!("{}_{}", self.num_elements, self.parameter_label)
    }

    fn generate(&self, seed: u64) -> SortingInstance<T> {
        (&self.element_generator)
            .sample_iter(Pcg64::seed_from_u64(seed))
            .take(self.num_elements)
            .collect()
    }
}
