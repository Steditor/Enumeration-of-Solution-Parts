use crate::{
    data_generators::graphs::DAG,
    data_structures::{
        graphs::InOutAdjacencyArraysGraph,
        scheduling_problems::{Job, SchedulingInstance, SingleMachine},
    },
    experiments::InstanceGenerator,
};

use num::{rational::Ratio, ToPrimitive, Zero};
use num_bigint::BigInt;
use rand::distributions::Standard;

use super::taillard_lcg::TaillardLCG;

/// A single-machine scheduling instance with DAG precedences.
///
/// This corresponds to problems of the type 1|prec| in standardized scheduling notation.
///
/// Each processing time is chosen uniformly at random from the integer interval `1..=99`.
/// The DAG is chosen uniformly at random in the G(n,p) model with the given edge_probability.
pub struct WithPrecedences {
    pub jobs: u32,
    pub edge_probability: f64,
    pub parameter_label: String,
}

impl
    InstanceGenerator<
        SchedulingInstance<SingleMachine, u32, (), (), InOutAdjacencyArraysGraph<u32>>,
    > for WithPrecedences
{
    fn path() -> String {
        String::from("./data/scheduling/single_machine/with_prec/")
    }

    fn file_name(&self) -> String {
        format!("{}_{}", self.jobs, self.parameter_label)
    }

    fn generate(
        &self,
        seed: u64,
    ) -> SchedulingInstance<SingleMachine, u32, (), (), InOutAdjacencyArraysGraph<u32>> {
        let mut rng = TaillardLCG::from_seed(seed);

        let mut job_data: Vec<Job<u32>> = (0..self.jobs)
            .map(|id| Job::for_num_operations(id, 1))
            .collect();

        for j in &mut job_data {
            j.operations[0] = rng.next_i32(1..=99) as u32;
        }

        let precedences = DAG::new(
            self.jobs,
            self.edge_probability,
            Standard,
            self.parameter_label.clone(),
        )
        .generate(seed);

        SchedulingInstance {
            environment: SingleMachine,
            jobs: job_data,
            precedences,
        }
    }
}

/// A single-machine scheduling instance with release times.
///
/// This corresponds to problems of the type 1|r_j| in standardized scheduling notation.
///
/// Each processing time is chosen uniformly at random from the integer interval `1..=99`.
/// Let `T` be the total processing time (sum of all individual processing times).
/// Release times are chosen uniformly at random from the integer interval
/// `0..=min(floor(T * release_spread), i32::MAX)`.
/// By chosing `release_spread` appropriately one can thus generate instances where
/// jobs are usually lining up to be scheduled (`release_spread` < 1) or there are gaps
/// where no jobs are available (`release_spread` > 1). Note that the later can only happen,
/// if the sum of processing times is not too large, as the release time is stored as `i32` and
/// we thus cap the maximum release time at `i32::MAX`.
pub struct WithReleaseTimes {
    pub jobs: u32,
    pub release_spread: f64,
}

impl InstanceGenerator<SchedulingInstance<SingleMachine, u32, (), u32>> for WithReleaseTimes {
    fn path() -> String {
        String::from("./data/scheduling/single_machine/with_release_times/")
    }

    fn file_name(&self) -> String {
        format!("{}_{}", self.jobs, self.release_spread)
    }

    fn generate(&self, seed: u64) -> SchedulingInstance<SingleMachine, u32, (), u32> {
        let mut rng = TaillardLCG::from_seed(seed);

        let mut job_data: Vec<Job<u32, (), u32>> = (0..self.jobs)
            .map(|id| Job::for_num_operations(id, 1))
            .collect();

        for j in &mut job_data {
            j.operations[0] = rng.next_i32(1..=99) as u32;
        }

        // Even though the individual operation lengths are at most 99, the sum could be large due to the number of jobs.
        // We saturate the max_release_time at `i32::MAX`.
        let total_time = job_data
            .iter()
            .fold(BigInt::zero(), |sum, j| sum + j.operations[0]);
        let spread =
            Ratio::from_float(self.release_spread).expect("Release time spread must be rational.");
        let spread_total_time = (Ratio::from(total_time) * spread).floor();
        let max_release_time = spread_total_time.to_i32().unwrap_or(i32::MAX);

        for j in &mut job_data {
            j.release_time = rng.next_i32(0..=max_release_time) as u32;
        }

        SchedulingInstance {
            environment: SingleMachine,
            jobs: job_data,
            precedences: (),
        }
    }
}
