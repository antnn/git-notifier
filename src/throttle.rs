use std::time::{Duration, Instant};

pub struct Throttle {
    calls_allowed: u64,
    calls_remaining: u64,
    last_throttle: Instant,
    interval: Duration,
}

impl Throttle {
    pub fn new(interval: Duration, calls_allowed: u64) -> Self {
        Self {
            calls_allowed,
            calls_remaining: calls_allowed,
            interval,
            last_throttle: Instant::now(),
        }
    }

    pub fn should_allow(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_throttle) > self.interval {
            self.calls_remaining = self.calls_allowed;
            self.last_throttle = now;
            return true;
        }
        if self.calls_remaining > 0 {
            self.calls_remaining -= 1;
            return true;
        }
        return false;
    }
}
