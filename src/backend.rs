use crate::{Schedule, Status};

pub(crate) trait Backend {
    fn name(&self) -> &'static str;
    fn status(&self) -> Result<Status, String>;
    fn set_status(&self, status: Status) -> Result<(), String>;
    fn schedule(&self) -> Result<Schedule, String>;
    fn set_schedule(&self, schedule: Schedule) -> Result<(), String>;
    fn strength(&self) -> Result<i32, String>;
    fn set_strength(&self, strength: i32) -> Result<(), String>;
}
