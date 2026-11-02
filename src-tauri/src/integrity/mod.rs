//! Boundary reserved for Sprint 03 (SM-012 through SM-017).
//! No baseline is approved and no integrity evaluation runs in Sprint 01.

/// Future evaluation must distinguish operational uncertainty from confirmed damage.
pub trait IntegrityEvaluator {
    type Snapshot;
    type ApprovedBaseline;
    type Assessment;
    type Error;

    fn evaluate(
        &self,
        baseline: &Self::ApprovedBaseline,
        snapshot: &Self::Snapshot,
    ) -> Result<Self::Assessment, Self::Error>;
}
