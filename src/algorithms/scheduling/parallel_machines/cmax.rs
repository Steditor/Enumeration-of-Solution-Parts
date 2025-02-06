//! Approximation algorithms for P||C_max

use std::{cmp::Reverse, collections::HashMap, slice::ChunksExact};

use binary_heap_plus::{BinaryHeap, MinComparator};
use num::cast::AsPrimitive;

use crate::{
    algorithms::sorting::IQS,
    data_structures::{
        scheduling_problems::{Job, ParallelMachines, SchedulingInstance},
        Index,
    },
    experiments::{ExperimentAlgorithm, PreparedEnumerationAlgorithm, ResultMetric},
};

use super::SchedulePartial;

type InstanceType = SchedulingInstance<ParallelMachines, u32, (), (), ()>;
pub type AlgorithmType = ExperimentAlgorithm<InstanceType, SchedulePartial, Vec<SchedulePartial>>;

pub struct Metric {}
impl ResultMetric<InstanceType, SchedulePartial, Vec<SchedulePartial>, u64> for Metric {
    fn output_quality(instance: &InstanceType, output: &Vec<SchedulePartial>) -> u64 {
        Self::partials_quality(instance, output)
    }

    fn partials_quality(instance: &InstanceType, partials: &[SchedulePartial]) -> u64 {
        // prepare job operation length lookup
        let mut jobs: HashMap<_, u64> = HashMap::with_capacity(instance.jobs.len());
        for job in &instance.jobs {
            jobs.insert(job.id, job.operations[0].as_());
        }

        partials
            .iter()
            .map(|entry| {
                entry.time
                    + jobs
                        .get(&entry.job)
                        .expect("A correct solution only references valid jobs.")
            })
            .max()
            .expect("We only consider non-empty instances")
    }
}

pub const ENUMERATE_WITH_LPT: AlgorithmType =
    ExperimentAlgorithm::EnumerationAlgorithm("enum-lpt", prepare_lpt_enumeration);

fn prepare_lpt_enumeration(input: &InstanceType) -> PreparedEnumerationAlgorithm<SchedulePartial> {
    Box::new(ScheduleEnumerator::new(input))
}

enum ScheduleEnumerator<'a> {
    CreditAccumulationPhase {
        machine_loads: BinaryHeap<(u64, u32), MinComparator>,
        chunks: ChunksExact<'a, Job<u32>>,
        extension_jobs: Vec<&'a Job<u32>>,
    },
    ExtensionAndFinalizationPhase {
        machine_loads: BinaryHeap<(u64, u32), MinComparator>,
        lpt_iterator: Box<dyn Iterator<Item = &'a Job<u32>> + 'a>,
    },
}

impl<'a> ScheduleEnumerator<'a> {
    pub fn new(instance: &'a InstanceType) -> Self {
        let m = instance.environment.machines;
        let n = instance.jobs.len();

        // store current machine loads as tuple (load, machine id)
        let mut machine_loads = BinaryHeap::with_capacity_min(m.index());
        for i in 0..m {
            machine_loads.push((0, i)); // lexicographic Ord.cmp ensures ordering first by load, then machine id
        }

        // prepare credit accumulation phase
        let num_acc_parts = n / (3 * m.index());
        let extension_jobs = Vec::<&Job<u32>>::with_capacity(n - num_acc_parts);
        let chunks = instance.jobs.chunks_exact(3 * m.index()); // consider blocks of 3m unseen jobs

        Self::CreditAccumulationPhase {
            machine_loads,
            chunks,
            extension_jobs,
        }
    }
}

impl Iterator for ScheduleEnumerator<'_> {
    type Item = SchedulePartial;

    fn next(&mut self) -> Option<Self::Item> {
        if let Self::CreditAccumulationPhase {
            machine_loads,
            chunks,
            extension_jobs,
        } = self
        {
            if let Some(chunk) = chunks.next() {
                // select in that block the job with smallest processing time
                // and deal with all others in the extension phase
                let mut min_job = &chunk[0];
                for job in chunk.iter().skip(1) {
                    if job.operations[0] < min_job.operations[0] {
                        extension_jobs.push(min_job);
                        min_job = job;
                    } else {
                        extension_jobs.push(job);
                    }
                }

                // schedule the selected job on machine with smallest current load
                let (load, machine) = machine_loads
                    .pop()
                    .expect("Heap of machines cannot run empty");
                machine_loads.push((load + u64::from(min_job.operations[0]), machine));

                // emit the corresponding solution part
                return Some(SchedulePartial {
                    job: min_job.id,
                    machine,
                    time: load,
                });
            } else {
                // next up: extension phase
                // add the remaining unseen jobs to the collection
                chunks
                    .remainder()
                    .iter()
                    .for_each(|j| extension_jobs.push(j));

                let iqs = IQS::with_comparator(extension_jobs, |j1: &&Job<u32>, j2: &&Job<u32>| {
                    j1.operations[0].cmp(&j2.operations[0]).reverse()
                });
                *self = Self::ExtensionAndFinalizationPhase {
                    machine_loads: machine_loads.to_owned(),
                    lpt_iterator: Box::new(iqs),
                };
            }
        }

        if let Self::ExtensionAndFinalizationPhase {
            machine_loads,
            lpt_iterator,
        } = self
        {
            if let Some(job) = lpt_iterator.next() {
                // schedule job on machine with smallest current load
                let (load, machine) = machine_loads
                    .pop()
                    .expect("Heap of machines cannot run empty");
                machine_loads.push((load + u64::from(job.operations[0]), machine));
                return Some(SchedulePartial {
                    job: job.id,
                    machine,
                    time: load,
                });
            } else {
                return None;
            }
        }

        panic!("Iterating on an undefined state is not supported.");
    }
}

pub const ENUMERATE_WITH_LPT_COROUTINE: AlgorithmType = ExperimentAlgorithm::EnumerationAlgorithm(
    "enum-lpt-coroutine",
    prepare_lpt_enumeration_coroutine,
);

fn prepare_lpt_enumeration_coroutine(
    input: &InstanceType,
) -> PreparedEnumerationAlgorithm<SchedulePartial> {
    let algorithm = #[coroutine]
    || {
        let m = input.environment.machines;
        let n = input.jobs.len();

        // store current machine loads as tuple (load, machine id)
        let mut machine_loads = BinaryHeap::with_capacity_min(m.index());
        for i in 0..m {
            machine_loads.push((0, i)); // lexicographic Ord.cmp ensures ordering first by load, then machine id
        }

        // Accumulation phase

        let num_acc_parts = n / (3 * m.index());
        let mut extension_jobs = Vec::<&Job<u32>>::with_capacity(n - num_acc_parts);

        // consider blocks of 3m unseen jobs
        let mut chunks = input.jobs.chunks_exact(3 * m.index());

        #[allow(clippy::while_let_on_iterator)]
        // Clippy does not understand the interaction with coroutine
        while let Some(chunk) = chunks.next() {
            // select in that block the job with smallest processing time
            // and deal with all others in the extension phase
            let mut min_job = &chunk[0];
            for job in chunk.iter().skip(1) {
                if job.operations[0] < min_job.operations[0] {
                    extension_jobs.push(min_job);
                    min_job = job;
                } else {
                    extension_jobs.push(job);
                }
            }

            // schedule the selected job on machine with smallest current load
            let (load, machine) = machine_loads
                .pop()
                .expect("Heap of machines cannot run empty");
            machine_loads.push((load + u64::from(min_job.operations[0]), machine));

            // and emit the corresponding solution part
            yield SchedulePartial {
                job: min_job.id,
                machine,
                time: load,
            };
        }

        // extension phase

        // add the remaining unseen jobs to the collection
        chunks
            .remainder()
            .iter()
            .for_each(|j| extension_jobs.push(j));

        let iqs = IQS::with_comparator(&extension_jobs, |j1: &&Job<u32>, j2: &&Job<u32>| {
            j1.operations[0].cmp(&j2.operations[0]).reverse()
        });

        for job in iqs {
            // schedule job on machine with smallest current load
            let (load, machine) = machine_loads
                .pop()
                .expect("Heap of machines cannot run empty");
            machine_loads.push((load + u64::from(job.operations[0]), machine));
            yield SchedulePartial {
                job: job.id,
                machine,
                time: load,
            }
        }
    };

    Box::new(std::iter::from_coroutine(algorithm))
}

/// A total-time implementation of the LPT scheduling rule.
///
/// The implementation uses [binary_heap_plus] as priority queue for selecting machines.
pub const APPROXIMATE_WITH_LPT: AlgorithmType =
    ExperimentAlgorithm::TotalTimeAlgorithm("total-lpt", |input| Ok(lpt(input)));

fn lpt(input: &InstanceType) -> Vec<SchedulePartial> {
    let mut sortable_jobs: Vec<_> = input.jobs.iter().collect();
    sortable_jobs.sort_unstable_by_key(|j| Reverse(j.operations[0]));

    let mut schedule = Vec::with_capacity(sortable_jobs.len());
    // store current machine loads as tuple (load, machine id)
    let mut machine_loads = BinaryHeap::with_capacity_min(input.environment.machines.index());
    for i in 0..input.environment.machines {
        machine_loads.push((0, i)); // lexicographic Ord.cmp ensures ordering first by load, then machine id
    }

    for j in sortable_jobs {
        let (load, machine) = machine_loads
            .pop()
            .expect("Heap of machines cannot run empty");
        schedule.push(SchedulePartial {
            job: j.id,
            machine,
            time: load,
        });
        machine_loads.push((load + u64::from(j.operations[0]), machine));
    }

    schedule
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::data_structures::scheduling_problems::Job;

    use super::*;

    /// worst case instance for 3 machines with lpt-apx of 4/3 - 1/(3*3).
    const LPT_WORST_CASE_INSTANCE: [(u32, u32); 7] =
        [(1, 5), (2, 5), (3, 4), (4, 4), (5, 3), (6, 3), (7, 3)];

    const LPT_WORST_CASE_SOLUTION: [(u32, u32, u64); 7] = [
        (1, 0, 0),
        (2, 1, 0),
        (3, 2, 0),
        (4, 2, 4),
        (5, 0, 5),
        (6, 1, 5),
        (7, 0, 8),
    ];

    /// hand-made instance with makespan 164 on all 3 machines in a perfect schedule.
    const ENUMERATION_INSTANCE: [(u32, u32); 37] = [
        (1, 1),
        (2, 2),
        (3, 3),
        (4, 4),
        (5, 5),
        (6, 6),
        (7, 7),
        (8, 8),
        (9, 9),
        (10, 2),
        (11, 4),
        (12, 6),
        (13, 8),
        (14, 10),
        (15, 12),
        (16, 14),
        (17, 16),
        (18, 18),
        (19, 3),
        (20, 6),
        (21, 9),
        (22, 12),
        (23, 15),
        (24, 18),
        (25, 21),
        (26, 24),
        (27, 27),
        (28, 4),
        (29, 8),
        (30, 12),
        (31, 16),
        (32, 20),
        (33, 24),
        (34, 28),
        (35, 32),
        (36, 36),
        (37, 42),
    ];

    /// hand-checked solution of the lpt enumeration algorithm wich achieves optimum makespan
    const ENUMERATION_SOLUTION: [(u32, u32, u64); 37] = [
        (1, 0, 0),
        (10, 1, 0),
        (19, 2, 0),
        (28, 0, 1),
        (37, 1, 2),
        (36, 2, 3),
        (35, 0, 5),
        (34, 0, 37),
        (27, 2, 39),
        (33, 1, 44),
        (26, 0, 65),
        (25, 2, 66),
        (32, 1, 68),
        (24, 2, 87),
        (18, 1, 88),
        (31, 0, 89),
        (17, 0, 105),
        (23, 2, 105),
        (16, 1, 106),
        (30, 1, 120),
        (22, 2, 120),
        (15, 0, 121),
        (14, 1, 132),
        (21, 2, 132),
        (9, 0, 133),
        (13, 2, 141),
        (8, 0, 142),
        (29, 1, 142),
        (7, 2, 149),
        (20, 0, 150),
        (12, 1, 150),
        (6, 0, 156),
        (5, 1, 156),
        (4, 2, 156),
        (11, 2, 160),
        (3, 1, 161),
        (2, 0, 162),
    ];

    fn build_instance(jobs: &[(u32, u32)]) -> SchedulingInstance<ParallelMachines, u32> {
        SchedulingInstance {
            environment: ParallelMachines { machines: 3 },
            jobs: jobs
                .iter()
                .map(|j| Job {
                    id: j.0,
                    operations: vec![j.1],
                    deadline: (),
                    release_time: (),
                })
                .collect(),
            precedences: (),
        }
    }

    /// check whether the schedule is sound for the given instance
    fn check_schedule(
        instance: &SchedulingInstance<ParallelMachines, u32>,
        schedule: &[SchedulePartial],
    ) {
        // all tasks are scheduled exactly once?
        assert_eq!(schedule.len(), instance.jobs.len());
        for Job { id, .. } in &instance.jobs {
            assert!(schedule.iter().any(|entry| entry.job == *id))
        }

        // schedule is given with non-decreasing start times?
        assert!(schedule.is_sorted_by_key(|entry| entry.time));

        // prepare job operation length lookup
        let mut jobs = HashMap::with_capacity(instance.jobs.len());
        for job in &instance.jobs {
            jobs.insert(job.id, job.operations[0]);
        }

        // check each machine
        for i in 0..instance.environment.machines {
            // schedule for that machine
            let schedule_i: Vec<_> = schedule.iter().filter(|entry| entry.machine == i).collect();
            // schedule must start at 0
            assert_eq!(schedule_i[0].time, 0);
            // schedule must not have idle time, must not interrupt jobs
            for entries in schedule_i.windows(2) {
                let e1 = entries[0];
                let e2 = entries[1];
                assert_eq!(
                    e2.time,
                    e1.time
                        + u64::from(*jobs.get(&e1.job).expect("job set equality checked above"))
                );
            }
        }
    }

    #[test]
    fn test_enumeration_lpt_big() {
        let instance = build_instance(&ENUMERATION_INSTANCE);

        let schedule: Vec<_> = prepare_lpt_enumeration(&instance).collect();

        check_schedule(&instance, &schedule);

        assert_eq!(
            schedule,
            ENUMERATION_SOLUTION.map(|s| SchedulePartial {
                job: s.0,
                machine: s.1,
                time: s.2,
            })
        )
    }

    #[test]
    fn test_enumeration_lpt_small() {
        let instance = build_instance(&LPT_WORST_CASE_INSTANCE);

        let schedule: Vec<_> = prepare_lpt_enumeration(&instance).collect();

        check_schedule(&instance, &schedule);
    }

    #[test]
    fn test_enumeration_coroutine_lpt_big() {
        let instance = build_instance(&ENUMERATION_INSTANCE);

        let schedule: Vec<_> = prepare_lpt_enumeration_coroutine(&instance).collect();

        check_schedule(&instance, &schedule);

        assert_eq!(
            schedule,
            ENUMERATION_SOLUTION.map(|s| SchedulePartial {
                job: s.0,
                machine: s.1,
                time: s.2,
            })
        )
    }

    #[test]
    fn test_enumeration_coroutine_lpt_small() {
        let instance = build_instance(&LPT_WORST_CASE_INSTANCE);

        let schedule: Vec<_> = prepare_lpt_enumeration_coroutine(&instance).collect();

        check_schedule(&instance, &schedule);
    }

    #[test]
    fn test_total_time_lpt() {
        let instance = build_instance(&LPT_WORST_CASE_INSTANCE);

        let schedule = lpt(&instance);

        check_schedule(&instance, &schedule);

        assert_eq!(
            schedule,
            LPT_WORST_CASE_SOLUTION.map(|s| SchedulePartial {
                job: s.0,
                machine: s.1,
                time: s.2,
            })
        );
    }
}
