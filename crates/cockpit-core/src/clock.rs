//! Integer-millisecond clock and finite-budget primitives.
//!
//! These are descriptive arithmetic inputs, not trusted time or permission.
//! An operation owns its original deadline; a later source extension cannot
//! replenish this budget. UTC rollback is diagnosed conservatively, while a
//! stalled wall clock still consumes monotonic time. Whole-system rollback
//! across process restarts is outside this local clock's assurance.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
    #[error("clock reads crossed a monotonic millisecond boundary; obtain a new sample")]
    SamplingGap,
    #[error("clock sample cannot be represented in integer milliseconds")]
    TimeOutOfRange,
}

pub trait Clock {
    /// Capture a UTC/monotonic pair coherent at integer-millisecond precision.
    /// A sampling error must stop the current deadline decision; cached budget
    /// values cannot replace the failed sample. Samples can age after capture.
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
        Self::sample_from(SystemTime::now, || self.origin.elapsed())
    }
}

impl SystemClock {
    fn sample_from(
        utc_now: impl FnOnce() -> SystemTime,
        mut monotonic_elapsed: impl FnMut() -> Duration,
    ) -> Result<ClockSample, ClockError> {
        let before = monotonic_elapsed();
        let utc = utc_now();
        let after = monotonic_elapsed();
        // The UTC read must lie in one monotonic millisecond bucket. Refuse
        // a wider read interval instead of losing it at this decision or
        // advancing UTC (which could prematurely satisfy notBefore).
        if after < before {
            return Err(ClockError::MonotonicRollback);
        }
        if after.as_millis() != before.as_millis() {
            return Err(ClockError::SamplingGap);
        }
        let monotonic_millis = before
            .as_millis()
            .try_into()
            .map_err(|_| ClockError::TimeOutOfRange)?;
        let utc_millis = utc
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ClockError::TimeOutOfRange)?
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
    /// Return the cached budget from the last observation without sampling time.
    ///
    /// Before a deadline decision, obtain a fresh [`Clock::sample`] and pass it
    /// to [`Self::observe`], propagating either error. This cache can stay
    /// positive after actual expiry or a failed sampling attempt.
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
    fn system_clock_returns_a_coherent_sample_or_reports_a_read_gap() {
        let clock = SystemClock::default();
        let inspect = |result: Result<ClockSample, ClockError>| match result {
            Ok(sample) => {
                assert!(sample.utc_millis > 0);
                Some(sample)
            }
            Err(ClockError::SamplingGap) => None,
            Err(error) => panic!("unexpected system clock error: {error}"),
        };
        let first = inspect(clock.sample());
        let second = inspect(clock.sample());
        if let (Some(first), Some(second)) = (first, second) {
            assert!(second.monotonic_millis >= first.monotonic_millis);
        }
    }

    #[test]
    fn regression_gap_after_utc_read_is_rejected_before_a_budget_decision() {
        let elapsed = Cell::new(Duration::ZERO);
        let paired = SystemClock::sample_from(
            || {
                let utc = UNIX_EPOCH + Duration::from_millis(100);
                elapsed.set(Duration::from_millis(100));
                utc
            },
            || elapsed.get(),
        );
        assert_eq!(paired, Err(ClockError::SamplingGap));
    }

    #[test]
    fn regression_gap_before_utc_read_is_rejected() {
        let elapsed = Cell::new(Duration::ZERO);
        let paired = SystemClock::sample_from(
            || {
                elapsed.set(Duration::from_millis(100));
                UNIX_EPOCH + Duration::from_millis(200)
            },
            || elapsed.get(),
        );
        assert_eq!(paired, Err(ClockError::SamplingGap));
    }

    #[test]
    fn remaining_millis_is_cached_until_a_fresh_sample_is_observed() {
        let clock = ControlledClock(Cell::new(sample(100, 0)));
        let mut budget = MonotonicBudget::new(
            FiniteWindow::new(100, 200).unwrap(),
            clock.sample().unwrap(),
            100,
        )
        .unwrap();
        clock.0.set(sample(200, 100));
        assert_eq!(budget.remaining_millis(), 100);
        assert_eq!(budget.observe(clock.sample().unwrap()), Ok(0));
        assert_eq!(budget.remaining_millis(), 0);
    }

    #[test]
    fn regression_latest_sample_gap_cannot_leave_a_positive_budget_decision() {
        let mut budget =
            MonotonicBudget::new(FiniteWindow::new(0, 150).unwrap(), sample(100, 0), 50).unwrap();
        let elapsed = Cell::new(Duration::from_millis(10));
        let decision = SystemClock::sample_from(
            || {
                elapsed.set(Duration::from_millis(100));
                UNIX_EPOCH + Duration::from_millis(100)
            },
            || elapsed.get(),
        )
        .and_then(|fresh| budget.observe(fresh));
        assert!(
            !matches!(decision, Ok(remaining) if remaining > 0),
            "the latest read gap outlasted the entire budget: {decision:?}"
        );
    }

    #[test]
    fn coherent_reads_preserve_utc_without_advancing_a_window_start() {
        let window = FiniteWindow::new(100, 150).unwrap();
        for (utc, remaining) in [
            (99, Err(ClockError::NotYetValid)),
            (100, Ok(50)),
            (150, Err(ClockError::Expired)),
        ] {
            let mut ticks = [Duration::from_micros(25), Duration::from_micros(75)].into_iter();
            let paired = SystemClock::sample_from(
                || UNIX_EPOCH + Duration::from_millis(utc),
                || ticks.next().unwrap(),
            )
            .unwrap();
            assert_eq!(paired, sample(utc as i64, 0));
            assert_eq!(window.remaining_at(paired.utc_millis), remaining);
        }
    }

    #[test]
    fn even_a_submillisecond_read_gap_crossing_a_bucket_is_rejected() {
        let mut ticks = [Duration::from_micros(999), Duration::from_micros(1001)].into_iter();
        assert_eq!(
            SystemClock::sample_from(
                || UNIX_EPOCH + Duration::from_millis(100),
                || ticks.next().unwrap(),
            ),
            Err(ClockError::SamplingGap)
        );
    }

    #[test]
    fn monotonic_rollback_within_one_millisecond_bucket_is_rejected() {
        let mut ticks = [Duration::from_micros(900), Duration::from_micros(100)].into_iter();
        assert_eq!(
            SystemClock::sample_from(
                || UNIX_EPOCH + Duration::from_millis(100),
                || ticks.next().unwrap(),
            ),
            Err(ClockError::MonotonicRollback)
        );
    }
}
