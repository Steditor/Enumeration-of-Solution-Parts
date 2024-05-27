//! Exact algorithms for 1|r_j|C_max
//!
//! Optimize makespan by scheduling in order of non-decreasing release times.

use crate::{
    algorithms::sorting::IQS,
    data_structures::scheduling_problems::{Job, SchedulingInstance, SingleMachine},
    experiments::{ExperimentAlgorithm, PreparedEnumerationAlgorithm},
};

use super::SchedulePartial;

type InstanceType = SchedulingInstance<SingleMachine, i32, (), i32>;
pub type AlgorithmType = ExperimentAlgorithm<InstanceType, SchedulePartial, Vec<SchedulePartial>>;

/// Enumeration algorithm for 1|r_j|C_max with IQS for incremental sorting
pub const ENUMERATE_WITH_IQS: AlgorithmType =
    ExperimentAlgorithm::EnumerationAlgorithm("enum-iqs", EnumerateWithIQS::algorithm);

struct EnumerateWithIQS<'a> {
    iqs: IQS<&'a Job<i32, (), i32>>,
    time: i64,
}

impl<'a> EnumerateWithIQS<'a> {
    pub fn new(input: &'a InstanceType) -> Self {
        let sortable_jobs: Vec<&Job<i32, (), i32>> = input.jobs.iter().collect();
        let iqs = IQS::with_comparator(&sortable_jobs, |j1, j2| {
            j1.release_time.cmp(&j2.release_time)
        });
        Self { iqs, time: 0 }
    }

    fn algorithm(input: &InstanceType) -> PreparedEnumerationAlgorithm<SchedulePartial> {
        Box::new(EnumerateWithIQS::new(input))
    }
}

impl Iterator for EnumerateWithIQS<'_> {
    type Item = SchedulePartial;

    fn next(&mut self) -> Option<Self::Item> {
        self.iqs.next().map(|j| {
            let start_time = self.time.max(i64::from(j.release_time));
            self.time = start_time + i64::from(j.operations[0]);
            SchedulePartial {
                job: j.id,
                time: start_time,
            }
        })
    }
}

/// Total time algorithm for 1|r_j|C_max with rust's sort_unstable_by_key
pub const SOLVE_WITH_UNSTABLE_SORT: AlgorithmType =
    ExperimentAlgorithm::TotalTimeAlgorithm("total-unstable-sort", rust_unstable_sort);

fn rust_unstable_sort(input: &InstanceType) -> Vec<SchedulePartial> {
    let mut sortable_jobs: Vec<&Job<i32, (), i32>> = input.jobs.iter().collect();
    sortable_jobs.sort_unstable_by_key(|j| j.release_time);

    let mut schedule = Vec::with_capacity(sortable_jobs.len());
    let mut time = 0;
    for j in sortable_jobs {
        let start_time = time.max(i64::from(j.release_time));
        time = start_time + i64::from(j.operations[0]);
        schedule.push(SchedulePartial {
            job: j.id,
            time: start_time,
        });
    }
    schedule
}

#[cfg(test)]
mod test {
    use super::*;

    // At the time of writing the tests, this is the instance generated with
    // the random generator for single machine with release times and the parameters:
    // Taillard LCG (seed 42), 5 jobs, 0.5 release_spread.
    // First entry is the job id, then come the processing time and the release time.
    const INSTANCE: [(u32, i32, i32); 5] = [
        (0, 1, 18),
        (1, 52, 93),
        (2, 73, 49),
        (3, 27, 50),
        (4, 38, 24),
    ];

    // The solution to the above instance.
    // First entry is the job id, second the start time on the first machine.
    const SOLUTION: [(u32, i64); 5] = [(0, 18), (4, 24), (2, 62), (3, 135), (1, 162)];

    #[test]
    fn test_rj_cmax_enumeration() {
        let instance = SchedulingInstance {
            environment: SingleMachine,
            jobs: INSTANCE
                .iter()
                .map(|j| Job {
                    id: j.0,
                    operations: vec![j.1],
                    deadline: (),
                    release_time: j.2,
                })
                .collect(),
            precedences: (),
        };

        let schedule: Vec<_> = EnumerateWithIQS::new(&instance).collect();

        assert_eq!(
            schedule,
            SOLUTION.map(|s| SchedulePartial {
                job: s.0,
                time: s.1,
            })
        );
    }
    #[test]
    fn test_rj_cmax_total_time() {
        let instance = SchedulingInstance {
            environment: SingleMachine,
            jobs: INSTANCE
                .iter()
                .map(|j| Job {
                    id: j.0,
                    operations: vec![j.1],
                    deadline: (),
                    release_time: j.2,
                })
                .collect(),
            precedences: (),
        };

        let schedule = rust_unstable_sort(&instance);

        assert_eq!(
            schedule,
            SOLUTION.map(|s| SchedulePartial {
                job: s.0,
                time: s.1,
            })
        );
    }
}
