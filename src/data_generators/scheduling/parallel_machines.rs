use rand::SeedableRng;
use rand_pcg::Pcg64;

use crate::{
    data_generators::FixedRngDistribution,
    data_structures::scheduling_problems::{Job, ParallelMachines, SchedulingInstance},
    experiments::InstanceGenerator,
};

/// A scheduling instance for parallel machines without any precedences.
///
/// This corresponds to problems of the type P|| in standardized scheduling notation.
///
/// Each processing time is chosen uniformly at random from the integer interval `1..=100`.
pub struct Plain<'a> {
    pub machines: u32,
    pub jobs: u32,
    pub distribution: &'a dyn FixedRngDistribution<u32, Pcg64>,
}

impl InstanceGenerator<SchedulingInstance<ParallelMachines, u32>> for Plain<'_> {
    fn path() -> String {
        String::from("./data/scheduling/parallel_machines/plain/")
    }

    fn file_name(&self) -> String {
        format!("{}_{}", self.jobs, self.machines)
    }

    fn generate(&self, seed: u64) -> SchedulingInstance<ParallelMachines, u32> {
        let mut rng = Pcg64::seed_from_u64(seed);

        let mut job_data: Vec<Job<u32>> = (0..self.jobs)
            .map(|id| Job::for_num_operations(id, 1))
            .collect();

        for j in &mut job_data {
            j.operations[0] = self.distribution.sample(&mut rng);
        }

        SchedulingInstance {
            environment: ParallelMachines {
                machines: self.machines,
            },
            jobs: job_data,
            precedences: (),
        }
    }
}
