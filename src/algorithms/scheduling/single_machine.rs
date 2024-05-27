pub mod prec_cmax;
pub mod rj_cmax;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SchedulePartial {
    pub job: u32,
    pub time: i64,
}
