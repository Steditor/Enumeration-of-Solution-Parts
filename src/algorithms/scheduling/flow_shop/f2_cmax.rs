//! Exact algorithms for F2||C_max
//!
//! Optimize makespan by scheduling according to Johnson's algorithm \[1\]:
//!
//! - same order on both machines
//! - first execute jobs that have a shorter (or equal) processing time on machine 1, ordered by increasing processing time
//! - then execute jobs with shorter time on machine 2, ordered by decreasing processing time
//!
//! \[1\] S. M. Johnson, “Optimal two- and three-stage production schedules with setup times included,” Naval Research Logistics Quarterly, vol. 1, no. 1, pp. 61–68, 1954, doi: [10.1002/nav.3800010110](https://doi.org/10.1002/nav.3800010110).

use std::{cmp::Reverse, collections::VecDeque};

use crate::{
    algorithms::sorting::IQS,
    data_structures::scheduling_problems::{FlowShop, Job, SchedulingInstance},
    experiments::{ExperimentAlgorithm, PreparedEnumerationAlgorithm},
};

use super::SchedulePartial;

pub type AlgorithmType =
    ExperimentAlgorithm<SchedulingInstance<FlowShop, i32>, SchedulePartial, Vec<SchedulePartial>>;

/// Enumeration algorithm for F2||C_max with IQS for incremental sorting
pub const ENUMERATE_WITH_IQS: AlgorithmType =
    ExperimentAlgorithm::EnumerationAlgorithm("enum-iqs", EnumerateWithIQS::algorithm);

struct EnumerateWithIQS<'a> {
    iqs: std::iter::Chain<IQS<&'a Job<i32>>, IQS<&'a Job<i32>>>,
    time_machine_1: i64,
    next_machine_1: Option<SchedulePartial>,
    time_machine_2: i64,
    queue_machine_2: VecDeque<SchedulePartial>,
}

impl<'a> EnumerateWithIQS<'a> {
    pub fn new(input: &'a SchedulingInstance<FlowShop, i32>) -> Self {
        assert_eq!(
            input.environment.machines, 2,
            "Johnson's algorithm only works for exactly 2 machines."
        );
        let jobs_faster_or_equal_on_machine_1: Vec<&Job<i32>> = input
            .jobs
            .iter()
            .filter(|j| j.operations[0] <= j.operations[1])
            .collect();
        let jobs_faster_on_machine_2: Vec<&Job<i32>> = input
            .jobs
            .iter()
            .filter(|j| j.operations[0] > j.operations[1])
            .collect();
        let iqs = IQS::with_comparator(&jobs_faster_or_equal_on_machine_1, |j1, j2| {
            j1.operations[0].cmp(&j2.operations[0])
        })
        .chain(IQS::with_comparator(&jobs_faster_on_machine_2, |j1, j2| {
            j1.operations[0].cmp(&j2.operations[0]).reverse()
        }));
        Self {
            iqs,
            time_machine_1: 0,
            next_machine_1: None,
            time_machine_2: 0,
            queue_machine_2: VecDeque::new(),
        }
    }

    fn algorithm(
        input: &SchedulingInstance<FlowShop, i32>,
    ) -> PreparedEnumerationAlgorithm<SchedulePartial> {
        Box::new(EnumerateWithIQS::new(input))
    }
}

impl Iterator for EnumerateWithIQS<'_> {
    type Item = SchedulePartial;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_machine_1.is_none() {
            if let Some(j) = self.iqs.next() {
                // start on machine 1 as soon as the machine is free
                let start_time_m1 = self.time_machine_1;
                self.time_machine_1 += i64::from(j.operations[0]);
                // start as soon as both machine 2 is free and the job is done on machine 1
                let start_time_m2 = self.time_machine_2.max(self.time_machine_1);
                self.time_machine_2 = start_time_m2 + i64::from(j.operations[1]);

                self.next_machine_1 = Some(SchedulePartial {
                    job: j.id,
                    machine: 1,
                    time: start_time_m1,
                });

                self.queue_machine_2.push_back(SchedulePartial {
                    job: j.id,
                    machine: 2,
                    time: start_time_m2,
                });
            };
        }

        // emit the next earliest schedule entry
        match min_time(self.next_machine_1, self.queue_machine_2.front().copied()) {
            Some(SchedulePartial { machine: 1, .. }) => self.next_machine_1.take(),
            Some(SchedulePartial { machine: 2, .. }) => self.queue_machine_2.pop_front(),
            None => None,
            _ => panic!("F2||C_max can't schedule on machines other than 1 or 2."),
        }
    }
}

fn min_time(sp1: Option<SchedulePartial>, sp2: Option<SchedulePartial>) -> Option<SchedulePartial> {
    // if one is None, take the other (as result of x = min(x,x))
    // if both are None, returns None
    // if both are Some, compares them by time and returns the partial with smaller time
    std::cmp::min_by_key(sp1.or(sp2), sp2.or(sp1), |sp| sp.map(|p| p.time))
}

/// Total time algorithm for F2||C_max with rust's sort_unstable_by_key
pub const SOLVE_WITH_UNSTABLE_SORT: AlgorithmType =
    ExperimentAlgorithm::TotalTimeAlgorithm("total-unstable-sort", rust_unstable_sort);

fn rust_unstable_sort(input: &SchedulingInstance<FlowShop, i32>) -> Vec<SchedulePartial> {
    assert_eq!(
        input.environment.machines, 2,
        "Johnson's algorithm only works for exactly 2 machines."
    );

    let mut jobs_faster_or_equal_on_machine_1: Vec<&Job<i32>> = input
        .jobs
        .iter()
        .filter(|j| j.operations[0] <= j.operations[1])
        .collect();
    jobs_faster_or_equal_on_machine_1.sort_unstable_by_key(|j| j.operations[0]);
    let mut jobs_faster_on_machine_2: Vec<&Job<i32>> = input
        .jobs
        .iter()
        .filter(|j| j.operations[0] > j.operations[1])
        .collect();
    jobs_faster_on_machine_2.sort_unstable_by_key(|j| Reverse(j.operations[1]));

    let mut schedule = Vec::with_capacity(input.jobs.len());
    let mut time_machine_1 = 0;
    let mut time_machine_2 = 0;
    for j in jobs_faster_or_equal_on_machine_1
        .iter()
        .chain(jobs_faster_on_machine_2.iter())
    {
        let start_time_m1 = time_machine_1;
        time_machine_1 += i64::from(j.operations[0]);
        // start as soon as both machine 2 is free and the job is done on machine 1
        let start_time_m2 = time_machine_2.max(time_machine_1);
        time_machine_2 = start_time_m2 + i64::from(j.operations[1]);
        schedule.push(SchedulePartial {
            job: j.id,
            machine: 1,
            time: start_time_m1,
        });
        schedule.push(SchedulePartial {
            job: j.id,
            machine: 2,
            time: start_time_m2,
        });
    }
    schedule.sort_by_key(|p| p.time);
    schedule
}

#[cfg(test)]
mod test {
    use super::*;

    // this instance is taken from Johnson's paper (see [1] above)
    // First entry is the job id, then come the processing time on the first and second machine.
    const JOHNSON_INSTANCE: [(u32, i32, i32); 5] =
        [(1, 4, 5), (2, 4, 1), (3, 30, 4), (4, 6, 30), (5, 2, 3)];

    // this solution is taken from Johnson's paper (see [1] above)
    // First entry is the job id, then comes the machine and third the start time.
    const JOHNSON_SOLUTION: [(u32, u32, i64); 10] = [
        (5, 1, 0),
        (1, 1, 2),
        (5, 2, 2),
        (4, 1, 6),
        (1, 2, 6),
        (3, 1, 12),
        (4, 2, 12),
        (2, 1, 42),
        (3, 2, 42),
        (2, 2, 46),
    ];

    #[test]
    fn test_f2_cmax_enumeration() {
        let instance = SchedulingInstance {
            environment: FlowShop { machines: 2 },
            jobs: JOHNSON_INSTANCE
                .iter()
                .map(|j| Job {
                    id: j.0,
                    operations: vec![j.1, j.2],
                    deadline: (),
                    release_time: (),
                })
                .collect(),
            precedences: (),
        };
        let mut schedule: Vec<_> = EnumerateWithIQS::new(&instance).collect();

        assert!(
            schedule
                .as_slice()
                .windows(2)
                .all(|p| p[0].time <= p[1].time),
            "Partials not sorted by time."
        );

        schedule.sort_by_key(|p| p.machine);
        schedule.sort_by_key(|p| p.time);

        assert_eq!(
            schedule,
            JOHNSON_SOLUTION.map(|s| SchedulePartial {
                job: s.0,
                machine: s.1,
                time: s.2,
            })
        );
    }

    #[test]
    fn test_f2_cmax_total_time() {
        let instance = SchedulingInstance {
            environment: FlowShop { machines: 2 },
            jobs: JOHNSON_INSTANCE
                .iter()
                .map(|j| Job {
                    id: j.0,
                    operations: vec![j.1, j.2],
                    deadline: (),
                    release_time: (),
                })
                .collect(),
            precedences: (),
        };
        let mut schedule = rust_unstable_sort(&instance);

        assert!(
            schedule
                .as_slice()
                .windows(2)
                .all(|p| p[0].time <= p[1].time),
            "Partials not sorted by time."
        );

        schedule.sort_by_key(|p| p.machine);
        schedule.sort_by_key(|p| p.time);

        assert_eq!(
            schedule,
            JOHNSON_SOLUTION.map(|s| SchedulePartial {
                job: s.0,
                machine: s.1,
                time: s.2,
            })
        );
    }
}
