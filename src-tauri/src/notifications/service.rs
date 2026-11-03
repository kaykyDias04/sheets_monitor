use crate::{
    database::Database,
    integration::{CommandError, IntegrationRuntime},
    monitoring::{NotificationOutcome, NotificationRecord},
};
use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};

/// Success is submission to a native adapter, never proof of display or reading.
pub trait NotificationSink: Send + Sync {
    fn submit(&self, record: &NotificationRecord) -> NotificationOutcome;
}
pub struct NotificationService {
    runtime: IntegrationRuntime,
    sink: Arc<dyn NotificationSink>,
    notify: Arc<dyn Fn() + Send + Sync>,
    owner: String,
    stopped: AtomicBool,
    active: Mutex<bool>,
    wake: Condvar,
}
impl NotificationService {
    pub fn start(runtime: IntegrationRuntime, sink: Arc<dyn NotificationSink>, notify: Arc<dyn Fn() + Send + Sync>) -> Arc<Self> {
        let service = Arc::new(Self {
            runtime, sink, notify, owner: uuid::Uuid::new_v4().to_string(),
            stopped: AtomicBool::new(false), active: Mutex::new(false), wake: Condvar::new(),
        });
        let weak = Arc::downgrade(&service);
        thread::Builder::new().name("notification-outbox".into()).spawn(move || {
            while let Some(service) = weak.upgrade() {
                if service.stopped.load(Ordering::SeqCst) { break; }
                if let Ok(mut active) = service.active.lock() {
                    if service.stopped.load(Ordering::SeqCst) { break; }
                    *active = true;
                } else { break; }
                // Failures retain durable pending rows. No provider data is logged.
                let _ = service.pump();
                if let Ok(mut active) = service.active.lock() {
                    *active = false;
                    service.wake.notify_all();
                    if !service.stopped.load(Ordering::SeqCst) {
                        let _ = service.wake.wait_timeout(active, Duration::from_secs(1));
                    }
                }
            }
        }).expect("notification outbox thread");
        service
    }
    pub fn wake(&self) { self.wake.notify_all(); }
    pub fn shutdown(&self, timeout: Duration) -> bool {
        self.stopped.store(true, Ordering::SeqCst);
        self.wake.notify_all();
        let start = Instant::now();
        let Ok(mut active) = self.active.lock() else { return false; };
        while *active {
            let Some(remaining) = timeout.checked_sub(start.elapsed()) else { return false; };
            let Ok((next, waited)) = self.wake.wait_timeout(active, remaining) else { return false; };
            active = next;
            if waited.timed_out() { return !*active; }
        }
        true
    }
    fn pump(&self) -> Result<(), CommandError> {
        let state = self.runtime.auth.state()?;
        let Some(account) = state.auth.account else { return Ok(()); };
        if state.auth.status != "connected" { return Ok(()); }
        let mut changed = false;
        // A bounded batch keeps shutdown responsive under a burst of events.
        for _ in 0..10 {
            if self.stopped.load(Ordering::SeqCst) { break; }
            let dispatched = self.runtime.auth.with_generation(state.generation, || {
                let now = chrono::Utc::now().timestamp_millis();
                let mut db = Database::open(&self.runtime.database_path).map_err(|_| CommandError::local())?;
                db.expire_notifications(&account.subject, now)?;
                db.suppress_disabled_notifications(&account.subject, now)?;
                let Some(attempt) = db.claim_notification(&account.subject, &self.owner, now)? else { return Ok::<bool, CommandError>(false); };
                if self.stopped.load(Ordering::SeqCst) { return Ok(false); }
                // Lock the generation through the short native submission.
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.sink.submit(&attempt.record)))
                    .unwrap_or_else(|_| NotificationOutcome::Retryable { code: "native_prepare_failed".into(), next_attempt_ms: now.saturating_add(crate::monitoring::retry_delay_ms(attempt.record.attempt_count, rand::random::<u64>() % 1001, None)) });
                db.finish_notification_attempt(&account.subject, &attempt.record.id, &attempt.attempt_id, outcome, chrono::Utc::now().timestamp_millis())?;
                Ok(true)
            })??;
            changed |= dispatched;
            if !dispatched { break; }
        }
        if changed { (self.notify)(); }
        Ok(())
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
