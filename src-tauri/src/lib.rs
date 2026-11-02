//! Sprint 01: local desktop foundation, synthetic examples and SQLite only.
pub mod auth;
pub mod database;
pub mod google;
pub mod integrity;
pub mod monitoring;
pub mod notifications;

#[cfg(feature = "desktop")]
mod app;

#[cfg(feature = "desktop")]
pub use app::run;
