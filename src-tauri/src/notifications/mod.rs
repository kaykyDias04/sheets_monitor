//! Durable, account-scoped delivery of every eligible detected event.
#[cfg(feature = "desktop")]
pub mod native;
pub mod service;
pub use service::{NotificationService, NotificationSink};
