pub mod cmax;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SchedulePartial {
    pub job: u32,
    pub machine: u32,
    pub time: u64,
}
