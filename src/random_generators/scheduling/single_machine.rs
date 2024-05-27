use crate::{
    data_structures::{
        graphs::DirectedAdjacencyArraysGraph,
        scheduling_problems::{Job, SchedulingInstance, SingleMachine},
    },
    experiments::ExperimentGenerator,
    random_generators::{graphs::DAG, numbers::Rng},
};

use num::{rational::Ratio, ToPrimitive, Zero};
use num_bigint::BigInt;

/// A single-machine scheduling instance with DAG precedences.
///
/// This corresponds to problems of the type 1|prec| in standardized scheduling notation.
///
/// Each processing time is chosen uniformly at random from the integer interval `1..=99`.
/// The DAG is chosen uniformly at random in the G(n,p) model with the given edge_probability.
pub struct WithPrecedences<'a> {
    pub rng: &'a mut dyn Rng,
    pub jobs: u32,
    pub edge_probability: f64,
}

impl
    ExperimentGenerator<
        SchedulingInstance<SingleMachine, i32, (), (), DirectedAdjacencyArraysGraph<u32>>,
    > for WithPrecedences<'_>
{
    fn path() -> String {
        String::from("./data/scheduling/single_machine/with_prec/")
    }

    fn file_name(&self) -> String {
        format!(
            "{}_{}_{}",
            self.jobs,
            self.edge_probability,
            self.rng.state_id(),
        )
    }

    fn generate(
        &mut self,
    ) -> SchedulingInstance<SingleMachine, i32, (), (), DirectedAdjacencyArraysGraph<u32>> {
        let mut job_data: Vec<Job<i32>> = (0..self.jobs)
            .map(|id| Job::for_num_operations(id, 1))
            .collect();

        for j in &mut job_data {
            j.operations[0] = self.rng.next_i32(1..=99);
        }

        let precedences = DAG {
            rng: self.rng,
            num_vertices: self.jobs,
            edge_probability: self.edge_probability,
        }
        .generate();

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
pub struct WithReleaseTimes<'a> {
    pub rng: &'a mut dyn Rng,
    pub jobs: u32,
    pub release_spread: f64,
}

impl ExperimentGenerator<SchedulingInstance<SingleMachine, i32, (), i32>> for WithReleaseTimes<'_> {
    fn path() -> String {
        String::from("./data/scheduling/single_machine/with_release_times/")
    }

    fn file_name(&self) -> String {
        format!(
            "{}_{}_{}",
            self.jobs,
            self.release_spread,
            self.rng.state_id(),
        )
    }

    fn generate(&mut self) -> SchedulingInstance<SingleMachine, i32, (), i32> {
        let mut job_data: Vec<Job<i32, (), i32>> = (0..self.jobs)
            .map(|id| Job::for_num_operations(id, 1))
            .collect();

        for j in &mut job_data {
            j.operations[0] = self.rng.next_i32(1..=99);
        }

        // Even though the individual operation lengths are at most 99, the sum could be large due to the number of jobs.
        // We saturate the max_release_time at `i32::MAX`.
        let total_time = job_data
            .iter()
            .fold(BigInt::zero(), |sum, j| sum + j.operations[0]);
        let spread =
            Ratio::from_float(self.release_spread).expect("Release time spread must be rational.");
        let spread_total_time = (Ratio::from(total_time) * spread).floor();
        let max_release_time: i32 = spread_total_time.to_i32().unwrap_or(i32::MAX);

        for j in &mut job_data {
            j.release_time = self.rng.next_i32(0..=max_release_time);
        }

        SchedulingInstance {
            environment: SingleMachine,
            jobs: job_data,
            precedences: (),
        }
    }
}
