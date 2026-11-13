//! Scheduling arithmetic is independent of wall-clock waits and Google APIs.
pub trait MonitoringClock: Send + Sync {
    fn now_ms(&self) -> i64;
    fn jitter_ms(&self) -> u64;
}
pub struct SystemMonitoringClock;
impl MonitoringClock for SystemMonitoringClock {
    fn now_ms(&self) -> i64 {
        chrono::Utc::now().timestamp_millis()
    }
    fn jitter_ms(&self) -> u64 {
        rand::random::<u64>() % 1001
    }
}
pub fn interval_ms(seconds: u32) -> i64 {
    i64::from(seconds.clamp(60, 600)) * 1000
}
/// Sorted eligible files are spread across one interval; missed ticks are never replayed.
pub fn stagger_ms(now: i64, index: usize, count: usize, interval: i64) -> i64 {
    now.saturating_add(interval.saturating_mul(index as i64) / count.max(1) as i64)
}
pub fn retry_delay_ms(failures: u32, jitter: u64, server_delay_ms: Option<i64>) -> i64 {
    let exponential = (1_000i64 << failures.min(8)).min(300_000);
    exponential
        .saturating_add((jitter % 1001) as i64)
        .max(server_delay_ms.unwrap_or(0))
}
pub fn resumed_after_gap(previous: i64, now: i64, interval: i64) -> bool {
    now < previous || now.saturating_sub(previous) > (interval * 2).max(120_000)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_and_configured_intervals_and_completion_are_bounded() {
        assert_eq!(interval_ms(60), 60_000);
        assert_eq!(interval_ms(300), 300_000);
        assert_eq!(interval_ms(0), 60_000);
        assert_eq!(interval_ms(600), 600_000);
        assert_eq!(interval_ms(601), 600_000);
        assert_eq!(stagger_ms(10_000, 2, 4, 60_000), 40_000);
    }
    #[test]
    fn backoff_honors_entire_server_delay_and_jitter() {
        assert_eq!(retry_delay_ms(1, 123, Some(3_600_000)), 3_600_000);
        assert_eq!(retry_delay_ms(2, 123, None), 4123);
        assert!(resumed_after_gap(100, 1_000_000, 60_000));
        assert!(resumed_after_gap(100, 0, 60_000));
        assert!(!resumed_after_gap(100, 1100, 60_000));
    }
}
