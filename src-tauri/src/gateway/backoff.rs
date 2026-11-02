//! Reconnection delays.

use std::time::Duration;

const BASE_MS: u64 = 500;
const MAX: Duration = Duration::from_secs(30);

/// Exponential backoff capped at 30 s. `jitter` in `[0, 1)` spreads the
/// delay over ±20 % so clients don't all reconnect at once after a restart.
pub fn delay(attempt: u32, jitter: f64) -> Duration {
    let exp = BASE_MS.saturating_mul(1u64 << attempt.min(16));
    let base = Duration::from_millis(exp).min(MAX);
    base.mul_f64(0.8 + 0.4 * jitter.clamp(0.0, 1.0)).min(MAX)
}

/// Cheap jitter source; no need for a real RNG here.
pub fn jitter() -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    f64::from(nanos % 1000) / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grows_then_caps() {
        assert_eq!(delay(0, 0.5), Duration::from_millis(500));
        assert_eq!(delay(1, 0.5), Duration::from_millis(1000));
        assert_eq!(delay(3, 0.5), Duration::from_millis(4000));
        assert_eq!(delay(10, 0.5), MAX);
        assert_eq!(delay(u32::MAX, 0.99), MAX);
    }

    #[test]
    fn jitter_stays_within_20_percent() {
        assert_eq!(delay(2, 0.0), Duration::from_millis(1600));
        assert_eq!(delay(2, 1.0), Duration::from_millis(2400));
        let j = jitter();
        assert!((0.0..1.0).contains(&j));
    }
}
