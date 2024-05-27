use serde::{Deserialize, Serialize};

/// Marker trait for machine environments
pub trait MachineEnvironment: std::fmt::Debug {}

/// Machine environment: Single machine
#[derive(Debug, Serialize, Deserialize)]
pub struct SingleMachine;
impl MachineEnvironment for SingleMachine {}

/// Machine environment: Identical parallel machines
#[derive(Debug, Serialize, Deserialize)]
pub struct ParallelMachines {
    pub machines: u32,
}
impl MachineEnvironment for ParallelMachines {}

/// Machine environment: Flow Shop
///
/// Each job consists of m operations to be processed on machines
/// M_1, ..., M_m in this order.
#[derive(Debug, Serialize, Deserialize)]
pub struct FlowShop {
    pub machines: u32,
}
impl MachineEnvironment for FlowShop {}

/// Machine environment: Open Shop
///
/// Each job consists of m operations, one for each machine, to be processed
/// in any order.
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenShop {
    pub machines: u32,
}
impl MachineEnvironment for OpenShop {}

/// Machine environment: Job Shop
///
/// Each job consists of a sequence of n_j operations which have to be
/// processed in this order. Associated with each operation is a set of
/// machines on which the operation may be processed.
#[derive(Debug, Serialize, Deserialize)]
pub struct JobShop {
    pub machines: u32,
}
impl MachineEnvironment for JobShop {}
