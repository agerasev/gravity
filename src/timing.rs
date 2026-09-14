use std::time::Duration;

/// At most 100 ms of catch-up, then discard excess wall time after a stall.
/// Paused, unfocused, and resized frames clear the accumulator.
const STEP: Duration =
    Duration::from_nanos(1_000_000_000_u64.div_ceil(gravity::STEPS_PER_SECOND as u64));
const MAX_STEPS: u32 = gravity::STEPS_PER_SECOND / 10;

#[derive(Default)]
pub struct Clock {
    accumulated: Duration,
}

impl Clock {
    pub fn reset(&mut self) {
        self.accumulated = Duration::ZERO;
    }

    pub fn advance(&mut self, elapsed: Duration, running: bool) -> u32 {
        if !running {
            self.reset();
            return 0;
        }
        self.accumulated += elapsed.min(STEP * MAX_STEPS);
        let steps = (self.accumulated.as_nanos() / STEP.as_nanos()) as u32;
        self.accumulated -= STEP * steps;
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stalls_and_pauses_do_not_accumulate_unbounded_catch_up() {
        let mut clock = Clock::default();
        assert_eq!(clock.advance(STEP / 2, true), 0);
        assert_eq!(clock.advance(Duration::from_secs(60), true), MAX_STEPS);
        assert_eq!(clock.advance(Duration::from_secs(60), false), 0);
        assert_eq!(clock.advance(STEP, true), 1);
        assert_eq!(clock.accumulated, Duration::ZERO);
    }

    #[test]
    fn frame_partitioning_preserves_simulated_time() {
        let mut clock = Clock::default();
        let mut steps = 0;
        for _ in 0..100 {
            steps += clock.advance(Duration::from_millis(10), true);
        }
        assert_eq!(steps, 239);
        assert_eq!(clock.accumulated, Duration::from_secs(1) - STEP * steps);
    }
}
