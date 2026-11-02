//! Boundary reserved for Sprint 05 (SM-022/024/025).
//! Interval settings are persisted metadata; no scheduler runs in this sprint.

/// Future local scheduler contract. No implementation is registered at startup.
pub trait MonitoringScheduler {
    type Error;

    fn start(&self) -> Result<(), Self::Error>;
    fn pause(&self) -> Result<(), Self::Error>;
}
