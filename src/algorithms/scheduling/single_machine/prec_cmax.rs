//! Exact algorithms for 1|prec|C_max
//!
//! Optimize makespan by scheduling in any topological order without idle time.

use crate::{
    algorithms::graphs::topological_ordering::{
        dfs_finish_time, idfs_finish_time, IterativeSourceRemoval,
    },
    data_structures::{
        graphs::InOutAdjacencyArraysGraph,
        scheduling_problems::{SchedulingInstance, SingleMachine},
        Index,
    },
    experiments::{ExperimentAlgorithm, PreparedEnumerationAlgorithm},
};

use super::SchedulePartial;

type InstanceType = SchedulingInstance<SingleMachine, u32, (), (), InOutAdjacencyArraysGraph<u32>>;

pub type AlgorithmType = ExperimentAlgorithm<InstanceType, SchedulePartial, Vec<SchedulePartial>>;

/// Enumeration algorithm for 1|prec|C_max with iterative topological ordering.
///
/// Note that the algorithm assumes that the index of a job in the jobs vector,
/// the job's id and the corresponding vertex id in the precedence graph are all identical.
/// The precedence graph is also expected to have exactly one vertex per job.
/// *No checks are made to verify those assumptions!*
pub const ENUMERATE_WITH_TOPO_SORT: AlgorithmType =
    ExperimentAlgorithm::EnumerationAlgorithm("enum-topo-sort", prepare_enumeration_algorithm);

struct EnumerateWithISR<'a> {
    isr: IterativeSourceRemoval<'a, InOutAdjacencyArraysGraph<u32>, u32, ()>,
    instance: &'a InstanceType,
    time: u64,
}

fn prepare_enumeration_algorithm(
    input: &InstanceType,
) -> PreparedEnumerationAlgorithm<SchedulePartial> {
    let isr = IterativeSourceRemoval::new(&input.precedences);
    Box::new(EnumerateWithISR {
        isr,
        instance: input,
        time: 0,
    })
}

impl Iterator for EnumerateWithISR<'_> {
    type Item = SchedulePartial;

    fn next(&mut self) -> Option<Self::Item> {
        self.isr.next().map(|r| {
            let job = r.expect("Precedence graph should not include cycles.");
            let j = &self.instance.jobs[job.index()];
            let start_time = self.time;
            self.time += u64::from(j.operations[0]);
            SchedulePartial {
                job,
                time: start_time,
            }
        })
    }
}

/// Total time algorithm for 1|prec|C_max with topological ordering via incremental dfs finish time.
///
/// Note that the algorithm assumes that the index of a job in the jobs vector,
/// the job's id and the corresponding vertex id in the precedence graph are all identical.
/// The precedence graph is also expected to have exactly one vertex per job.
/// *No checks are made to verify those assumptions!*
pub const SOLVE_WITH_IDFS_FINISH_TIME: AlgorithmType =
    ExperimentAlgorithm::TotalTimeAlgorithm("total-idfs-finish-time", |input| {
        Ok(order_by_idfs_finish_time(input))
    });

fn order_by_idfs_finish_time(input: &InstanceType) -> Vec<SchedulePartial> {
    let order =
        idfs_finish_time(&input.precedences).expect("Precedence graph should not include cycles.");

    let mut schedule = Vec::with_capacity(input.jobs.len());
    let mut time = 0;
    for job in order {
        schedule.push(SchedulePartial { job, time });
        time += u64::from(input.jobs[job.index()].operations[0]);
    }
    schedule
}

/// Total time algorithm for 1|prec|C_max with topological ordering via dfs finish time.
///
/// Note that the algorithm assumes that the index of a job in the jobs vector,
/// the job's id and the corresponding vertex id in the precedence graph are all identical.
/// The precedence graph is also expected to have exactly one vertex per job.
/// *No checks are made to verify those assumptions!*
pub const SOLVE_WITH_DFS_FINISH_TIME: AlgorithmType =
    ExperimentAlgorithm::TotalTimeAlgorithm("total-dfs-finish-time", |input| {
        Ok(order_by_dfs_finish_time(input))
    });

fn order_by_dfs_finish_time(input: &InstanceType) -> Vec<SchedulePartial> {
    let order =
        dfs_finish_time(&input.precedences).expect("Precedence graph should not include cycles.");

    let mut schedule = Vec::with_capacity(input.jobs.len());
    let mut time = 0;
    for job in order {
        schedule.push(SchedulePartial { job, time });
        time += u64::from(input.jobs[job.index()].operations[0]);
    }
    schedule
}

#[cfg(test)]
mod test {
    use crate::data_structures::{graphs::Graph, scheduling_problems::Job};

    use super::*;

    // pair of job id, processing time
    const JOBS: [(u32, u32); 5] = [(0, 54), (1, 83), (2, 15), (3, 71), (4, 77)];
    // precedences of the jobs
    const EDGES: [(u32, u32); 5] = [(0, 3), (1, 2), (3, 4), (3, 1), (4, 2)];
    // solution to the above instance; pair of job id, start time
    const SOLUTION: [(u32, u64); 5] = [(0, 0), (3, 54), (1, 125), (4, 208), (2, 285)];

    #[test]
    fn test_prec_cmax_enumeration() {
        let graph = InOutAdjacencyArraysGraph::new(5, &EDGES);
        let instance = SchedulingInstance {
            environment: SingleMachine,
            jobs: JOBS
                .iter()
                .map(|j| Job {
                    id: j.0,
                    operations: vec![j.1],
                    deadline: (),
                    release_time: (),
                })
                .collect(),
            precedences: graph,
        };
        let schedule: Vec<_> = prepare_enumeration_algorithm(&instance).collect();

        assert_eq!(
            schedule,
            SOLUTION.map(|s| SchedulePartial {
                job: s.0,
                time: s.1,
            }),
        )
    }

    #[test]
    fn test_prec_cmax_total_time_idfs() {
        let graph = InOutAdjacencyArraysGraph::new(5, &EDGES);
        let instance = SchedulingInstance {
            environment: SingleMachine,
            jobs: JOBS
                .iter()
                .map(|j| Job {
                    id: j.0,
                    operations: vec![j.1],
                    deadline: (),
                    release_time: (),
                })
                .collect(),
            precedences: graph,
        };
        let schedule: Vec<_> = order_by_idfs_finish_time(&instance);

        assert_eq!(
            schedule,
            SOLUTION.map(|s| SchedulePartial {
                job: s.0,
                time: s.1,
            }),
        )
    }

    #[test]
    fn test_prec_cmax_total_time_dfs() {
        let graph = InOutAdjacencyArraysGraph::new(5, &EDGES);
        let instance = SchedulingInstance {
            environment: SingleMachine,
            jobs: JOBS
                .iter()
                .map(|j| Job {
                    id: j.0,
                    operations: vec![j.1],
                    deadline: (),
                    release_time: (),
                })
                .collect(),
            precedences: graph,
        };
        let schedule: Vec<_> = order_by_dfs_finish_time(&instance);

        assert_eq!(
            schedule,
            SOLUTION.map(|s| SchedulePartial {
                job: s.0,
                time: s.1,
            }),
        )
    }
}
