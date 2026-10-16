use cerebro_domain::ports::Clock;
use cerebro_domain::time::Timestamp;

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp(cerebro_protocol::now_ms())
    }
}
