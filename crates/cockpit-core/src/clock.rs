//! Integer-millisecond clock and finite-budget primitives.
//!
//! These are descriptive arithmetic inputs, not trusted time or permission.
//! An operation owns its original deadline; a later source extension cannot
//! replenish this budget. UTC rollback is diagnosed conservatively, while a
//! stalled wall clock still consumes monotonic time. Whole-system rollback
//! across process restarts is outside this local clock's assurance.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockSample {
    pub utc_millis: i64,
    pub monotonic_millis: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ClockError {
    #[error("finite window must end strictly after its start")]
    InvalidWindow,
    #[error("finite window has not started")]
    NotYetValid,
    #[error("finite window has expired")]
    Expired,
    #[error("operation budget must be positive")]
    ZeroBudget,
    #[error("UTC clock moved backwards")]
    WallClockRollback,
    #[error("monotonic clock moved backwards")]
    MonotonicRollback,
    #[error("clock sample cannot be represented in integer milliseconds")]
    TimeOutOfRange,
}

pub trait Clock {
    fn sample(&self) -> Result<ClockSample, ClockError>;
}

pub struct SystemClock {
    origin: Instant,
}

impl Default for SystemClock {
    fn default() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl Clock for SystemClock {
    fn sample(&self) -> Result<ClockSample, ClockError> {
        let utc_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ClockError::TimeOutOfRange)?
            .as_millis()
            .try_into()
            .map_err(|_| ClockError::TimeOutOfRange)?;
        let monotonic_millis = self
            .origin
            .elapsed()
            .as_millis()
            .try_into()
            .map_err(|_| ClockError::TimeOutOfRange)?;
        Ok(ClockSample {
            utc_millis,
            monotonic_millis,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FiniteWindow {
    not_before_utc_millis: i64,
    expires_at_utc_millis: i64,
}

impl FiniteWindow {
    pub fn new(not_before_utc_millis: i64, expires_at_utc_millis: i64) -> Result<Self, ClockError> {
        if not_before_utc_millis >= expires_at_utc_millis {
            return Err(ClockError::InvalidWindow);
        }
        Ok(Self {
            not_before_utc_millis,
            expires_at_utc_millis,
        })
    }

    pub fn remaining_at(self, utc_millis: i64) -> Result<u64, ClockError> {
        if utc_millis < self.not_before_utc_millis {
            return Err(ClockError::NotYetValid);
        }
        if utc_millis >= self.expires_at_utc_millis {
            return Err(ClockError::Expired);
        }
        // Widen before subtracting: even MIN..MAX is a finite u64 interval.
        u64::try_from(i128::from(self.expires_at_utc_millis) - i128::from(utc_millis))
            .map_err(|_| ClockError::TimeOutOfRange)
    }

    pub fn not_before_utc_millis(self) -> i64 {
        self.not_before_utc_millis
    }
    pub fn expires_at_utc_millis(self) -> i64 {
        self.expires_at_utc_millis
    }
}

#[derive(Debug)]
pub struct MonotonicBudget {
    window: FiniteWindow,
    last_sample: ClockSample,
    remaining_millis: u64,
    poisoned: Option<ClockError>,
}

impl MonotonicBudget {
    pub fn new(
        window: FiniteWindow,
        sample: ClockSample,
        command_limit_millis: u64,
    ) -> Result<Self, ClockError> {
        if command_limit_millis == 0 {
            return Err(ClockError::ZeroBudget);
        }
        let remaining_millis = window
            .remaining_at(sample.utc_millis)?
            .min(command_limit_millis);
        Ok(Self {
            window,
            last_sample: sample,
            remaining_millis,
            poisoned: None,
        })
    }
    pub fn remaining_millis(&self) -> u64 {
        self.remaining_millis
    }

    pub fn observe(&mut self, sample: ClockSample) -> Result<u64, ClockError> {
        if let Some(error) = self.poisoned {
            return Err(error);
        }
        let error = if sample.monotonic_millis < self.last_sample.monotonic_millis {
            Some(ClockError::MonotonicRollback)
        } else if sample.utc_millis < self.last_sample.utc_millis {
            Some(ClockError::WallClockRollback)
        } else {
            None
        };
        if let Some(error) = error {
            self.remaining_millis = 0;
            self.poisoned = Some(error);
            return Err(error);
        }
        let elapsed = sample.monotonic_millis - self.last_sample.monotonic_millis;
        let utc_remaining = match self.window.remaining_at(sample.utc_millis) {
            Ok(remaining) => remaining,
            Err(ClockError::Expired) => 0,
            Err(error) => {
                self.remaining_millis = 0;
                self.poisoned = Some(error);
                return Err(error);
            }
        };
        self.remaining_millis = self
            .remaining_millis
            .saturating_sub(elapsed)
            .min(utc_remaining);
        self.last_sample = sample;
        Ok(self.remaining_millis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct ControlledClock(Cell<ClockSample>);

    impl Clock for ControlledClock {
        fn sample(&self) -> Result<ClockSample, ClockError> {
            Ok(self.0.get())
        }
    }

    fn sample(utc: i64, monotonic: u64) -> ClockSample {
        ClockSample {
            utc_millis: utc,
            monotonic_millis: monotonic,
        }
    }

    #[test]
    fn window_start_is_inclusive_and_expiry_is_exclusive() {
        let window = FiniteWindow::new(100, 200).unwrap();
        assert_eq!(window.remaining_at(99), Err(ClockError::NotYetValid));
        assert_eq!(window.remaining_at(100), Ok(100));
        assert_eq!(window.remaining_at(199), Ok(1));
        assert_eq!(window.remaining_at(200), Err(ClockError::Expired));
        assert_eq!(window.remaining_at(201), Err(ClockError::Expired));
    }

    #[test]
    fn empty_or_reversed_window_is_rejected() {
        assert_eq!(FiniteWindow::new(100, 100), Err(ClockError::InvalidWindow));
        assert_eq!(FiniteWindow::new(101, 100), Err(ClockError::InvalidWindow));
    }

    #[test]
    fn window_arithmetic_covers_negative_and_extreme_instants() {
        let window = FiniteWindow::new(i64::MIN, i64::MAX).unwrap();
        assert_eq!(window.remaining_at(i64::MIN), Ok(u64::MAX));
        assert_eq!(window.remaining_at(-1), Ok(1_u64 << 63));
        assert_eq!(window.expires_at_utc_millis(), i64::MAX);
        assert_eq!(window.not_before_utc_millis(), i64::MIN);
    }

    #[test]
    fn budget_takes_the_smaller_command_and_window_limit() {
        let window = FiniteWindow::new(0, 200).unwrap();
        assert_eq!(
            MonotonicBudget::new(window, sample(100, 5), 40)
                .unwrap()
                .remaining_millis(),
            40
        );
        assert_eq!(
            MonotonicBudget::new(window, sample(100, 5), 400)
                .unwrap()
                .remaining_millis(),
            100
        );
        assert_eq!(
            MonotonicBudget::new(window, sample(100, 5), 0).unwrap_err(),
            ClockError::ZeroBudget
        );
    }

    #[test]
    fn budget_requires_an_active_window() {
        let window = FiniteWindow::new(100, 200).unwrap();
        assert_eq!(
            MonotonicBudget::new(window, sample(99, 0), 10).unwrap_err(),
            ClockError::NotYetValid
        );
        assert_eq!(
            MonotonicBudget::new(window, sample(200, 0), 10).unwrap_err(),
            ClockError::Expired
        );
    }

    #[test]
    fn monotonic_elapsed_time_decreases_budget_when_wall_time_stalls() {
        let mut budget =
            MonotonicBudget::new(FiniteWindow::new(0, 200).unwrap(), sample(100, 10), 80).unwrap();
        assert_eq!(budget.observe(sample(100, 30)), Ok(60));
        assert_eq!(budget.observe(sample(100, 70)), Ok(20));
        assert_eq!(budget.observe(sample(100, 100)), Ok(0));
    }

    #[test]
    fn forward_wall_jump_shortens_and_then_expires_the_budget() {
        let mut budget =
            MonotonicBudget::new(FiniteWindow::new(0, 200).unwrap(), sample(100, 0), 90).unwrap();
        assert_eq!(budget.observe(sample(199, 1)), Ok(1));
        assert_eq!(budget.observe(sample(200, 2)), Ok(0));
        assert_eq!(budget.observe(sample(201, 3)), Ok(0));
    }

    #[test]
    fn wall_rollback_poisoning_cannot_be_reversed_by_a_later_sample() {
        let mut budget =
            MonotonicBudget::new(FiniteWindow::new(0, 200).unwrap(), sample(100, 10), 80).unwrap();
        assert_eq!(
            budget.observe(sample(99, 11)),
            Err(ClockError::WallClockRollback)
        );
        assert_eq!(budget.remaining_millis(), 0);
        assert_eq!(
            budget.observe(sample(110, 20)),
            Err(ClockError::WallClockRollback)
        );
    }

    #[test]
    fn monotonic_rollback_poisoning_cannot_be_reversed() {
        let mut budget =
            MonotonicBudget::new(FiniteWindow::new(0, 200).unwrap(), sample(100, 10), 80).unwrap();
        assert_eq!(
            budget.observe(sample(101, 9)),
            Err(ClockError::MonotonicRollback)
        );
        assert_eq!(budget.remaining_millis(), 0);
        assert_eq!(
            budget.observe(sample(110, 20)),
            Err(ClockError::MonotonicRollback)
        );
    }

    #[test]
    fn unchanged_observation_never_replenishes_a_consumed_budget() {
        let mut budget =
            MonotonicBudget::new(FiniteWindow::new(0, 200).unwrap(), sample(100, 10), 20).unwrap();
        assert_eq!(budget.observe(sample(105, 15)), Ok(15));
        assert_eq!(budget.observe(sample(105, 15)), Ok(15));
        assert_eq!(budget.observe(sample(125, 35)), Ok(0));
        assert_eq!(budget.observe(sample(125, 35)), Ok(0));
    }

    #[test]
    fn a_clock_can_be_injected_without_sleep_or_global_state() {
        let clock = ControlledClock(Cell::new(sample(100, 0)));
        let mut budget = MonotonicBudget::new(
            FiniteWindow::new(100, 200).unwrap(),
            clock.sample().unwrap(),
            100,
        )
        .unwrap();
        clock.0.set(sample(150, 50));
        assert_eq!(budget.observe(clock.sample().unwrap()), Ok(50));
        clock.0.set(sample(200, 100));
        assert_eq!(budget.observe(clock.sample().unwrap()), Ok(0));
    }

    #[test]
    fn system_clock_supplies_utc_and_non_decreasing_monotonic_samples() {
        let clock = SystemClock::default();
        let first = clock.sample().unwrap();
        let second = clock.sample().unwrap();
        assert!(first.utc_millis > 0);
        assert!(second.monotonic_millis >= first.monotonic_millis);
    }
}
