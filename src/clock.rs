use chrono::DateTime;
use chrono::Utc;

#[derive(Default)]
pub struct Clock;

#[mockall::automock]
impl Clock {
    pub fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
