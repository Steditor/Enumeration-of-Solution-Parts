pub mod f2_cmax;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SchedulePartial {
    pub job: u32,
    pub machine: u32,
    pub time: i64,
}
