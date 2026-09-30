//! Adaptive short measurements using adjacent batch residuals.
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct QuickConfig {
    pub min_time: Duration,
    pub max_time: Duration,
    /// Relative residual target, not a confidence interval or significance test.
    pub relative_deviation: f64,
}
impl Default for QuickConfig {
    fn default() -> Self {
        Self {
            min_time: Duration::from_millis(100),
            max_time: Duration::from_secs(5),
            relative_deviation: 0.05,
        }
    }
}
impl QuickConfig {
    pub(crate) fn validate(self) -> crate::Result<()> {
        if self.max_time.is_zero()
            || self.min_time > self.max_time
            || !self.relative_deviation.is_finite()
            || self.relative_deviation <= 0.0
            || self.relative_deviation >= 1.0
        {
            return Err(crate::error(
                "quick mode requires 0 <= min_time <= max_time, positive max_time and relative_deviation in (0,1)",
            ));
        }
        Ok(())
    }
    pub(crate) fn stop(
        self,
        previous: Option<(u64, u128)>,
        current: (u64, u128),
        elapsed: Duration,
    ) -> Option<&'static str> {
        if elapsed >= self.max_time {
            return Some("max_time");
        }
        if elapsed < self.min_time {
            return None;
        }
        let (n, before) = previous?;
        let ratio = current.0 as f64 / n as f64;
        let before = before as f64;
        let now = current.1 as f64;
        let estimate = (before + ratio * now) / (1.0 + ratio * ratio);
        let residual = (before - estimate).hypot(now - ratio * estimate);
        (estimate > 0.0 && residual < self.relative_deviation * estimate).then_some("stable")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn residual_stop_obeys_minimum_deadline_zero_and_iteration_cap() {
        let c = QuickConfig::default();
        assert_eq!(
            c.stop(Some((1, 100)), (2, 200), Duration::from_millis(99)),
            None
        );
        assert_eq!(c.stop(Some((1, 100)), (2, 200), c.min_time), Some("stable"));
        assert_eq!(c.stop(Some((1, 100)), (2, 1000), c.min_time), None);
        assert_eq!(c.stop(Some((1, 0)), (2, 0), c.min_time), None);
        assert_eq!(
            c.stop(Some((64, 6400)), (64, 6400), c.min_time),
            Some("stable")
        );
        assert_eq!(c.stop(None, (1, 100), c.max_time), Some("max_time"));
        assert!(
            QuickConfig {
                relative_deviation: f64::NAN,
                ..c
            }
            .validate()
            .is_err()
        );
        assert!(
            QuickConfig {
                min_time: Duration::from_secs(6),
                ..c
            }
            .validate()
            .is_err()
        );
    }
}
