//! Local desktop foundation and authorized, read-only Google integration.
pub mod auth;
pub mod database;
pub mod google;
pub mod integration;
pub mod integrity;
pub mod management;
pub mod monitoring;
pub mod notifications;

#[cfg(feature = "desktop")]
mod app;

#[cfg(feature = "desktop")]
pub use app::run;
