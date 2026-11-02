//! Boundary reserved for Sprint 05 (SM-023).
//! No native notification is dispatched by the foundation sprint.

/// Future delivery boundary must preserve every detected event during consolidation.
pub trait NativeNotificationDelivery {
    type Event;
    type Receipt;
    type Error;

    fn deliver(&self, event: &Self::Event) -> Result<Self::Receipt, Self::Error>;
}
