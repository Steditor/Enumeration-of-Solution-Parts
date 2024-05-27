mod machine_environment;

pub use machine_environment::*;
use serde::{Deserialize, Serialize};

use super::graphs::Index;

/// A scheduling instance.
///
/// An instance consists of a configuration of machines, jobs and (optional)
/// precedence constraints.
/// Details of the environment, operation types, potential deadlines / release
/// times and precedence constraints are left to the respective generic types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingInstance<
    Machines: MachineEnvironment,
    Operation,
    Deadline = (),
    ReleaseTime = (),
    Precedences = (),
> where
    Operation: Default,
    Deadline: Default,
    ReleaseTime: Default,
{
    pub environment: Machines,
    pub jobs: Vec<Job<Operation, Deadline, ReleaseTime>>,
    pub precedences: Precedences,
}

/// A job to be scheduled.
///
/// In Scheduling instances, a job consists of potentially many operations.
/// Each operation should have a processing time; the exact details are left to
/// the generic `Operation` type.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job<Operation, Deadline = (), ReleaseTime = ()>
where
    Operation: Default,
    Deadline: Default,
    ReleaseTime: Default,
{
    pub id: u32,
    pub operations: Vec<Operation>,
    pub deadline: Deadline,
    pub release_time: ReleaseTime,
}

impl<Operation, Deadline, ReleaseTime> Job<Operation, Deadline, ReleaseTime>
where
    Operation: Default,
    Deadline: Default,
    ReleaseTime: Default,
{
    pub fn for_num_operations(id: u32, num_operations: u32) -> Self
    where
        Operation: Clone,
    {
        Self {
            id,
            operations: vec![Operation::default(); num_operations.index()],
            deadline: Deadline::default(),
            release_time: ReleaseTime::default(),
        }
    }
}
