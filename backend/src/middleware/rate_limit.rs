use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct RateState {
    pub window_start: Instant,
    pub count: u64,
}

impl RateState {
    pub fn new() -> Self {
        Self {
            window_start: Instant::now(),
            count: 0,
        }
    }

    pub fn check_and_increment(&mut self, limit: u64, window: Duration) -> bool {
        if self.window_start.elapsed() > window {
            self.window_start = Instant::now();
            self.count = 0;
        }

        if self.count >= limit {
            false
        } else {
            self.count += 1;
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_within_limit() {
        let mut rl = RateState::new();
        let limit = 3;
        let window = Duration::from_secs(60);

        assert!(rl.check_and_increment(limit, window));
        assert!(rl.check_and_increment(limit, window));
        assert!(rl.check_and_increment(limit, window));
        assert!(!rl.check_and_increment(limit, window)); // exceeds
    }

    #[test]
    fn resets_after_window() {
        let mut rl = RateState::new();
        let limit = 1;
        let window = Duration::from_millis(1);

        assert!(rl.check_and_increment(limit, window));
        std::thread::sleep(Duration::from_millis(2));
        assert!(rl.check_and_increment(limit, window)); // window reset
    }
}
