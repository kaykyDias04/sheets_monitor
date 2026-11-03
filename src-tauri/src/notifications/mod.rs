//! Durable, account-scoped delivery of every eligible detected event.
pub mod service;
pub use service::{NotificationService, NotificationSink};
