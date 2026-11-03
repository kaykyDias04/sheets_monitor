//! Versioned, bounded contracts for on-demand structural integrity.
mod models;
pub use models::*;

pub mod collector;
pub mod compare;
pub mod formulas;
pub mod runtime;
pub mod scripts;

pub use compare::{evaluate, validate_baseline_snapshot};
#[cfg(test)]
mod engine_tests;
#[cfg(any(test, feature = "native-probe"))]
#[cfg_attr(not(test), allow(dead_code))]
// Probe uses only the shared scope/snapshot fixture helpers.
pub(crate) mod fixtures;

#[cfg(test)]
mod scale_tests;
