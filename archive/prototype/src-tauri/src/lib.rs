pub mod analysis;
pub mod domain;
pub mod engine;
pub mod fixtures;
pub mod google;
pub mod oauth;
pub mod store;

#[cfg(feature = "desktop")]
mod app;

#[cfg(feature = "desktop")]
pub use app::run;
