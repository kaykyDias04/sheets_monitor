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
#[cfg(test)]
pub(crate) mod fixtures;

#[cfg(test)]
mod scale_tests;
