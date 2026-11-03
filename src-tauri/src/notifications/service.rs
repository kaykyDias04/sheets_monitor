use crate::{
    database::Database,
    integration::{CommandError, IntegrationRuntime},
    monitoring::{NotificationOutcome, NotificationRecord},
};
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

/// Success is submission to a native adapter, never proof of display or reading.
pub trait NotificationSink: Send + Sync {
    fn submit(&self, record: &NotificationRecord) -> NotificationOutcome;
    /// Optional native failure event after initial submission; never a display receipt.
    fn submit_with_failure(
        &self,
        record: &NotificationRecord,
        _failed: Arc<dyn Fn(String) + Send + Sync>,
    ) -> NotificationOutcome {
        self.submit(record)
    }
}
pub struct NotificationService {
    runtime: IntegrationRuntime,
    sink: Arc<dyn NotificationSink>,
    notify: Arc<dyn Fn() + Send + Sync>,
    owner: String,
    stopped: AtomicBool,
    active: Mutex<bool>,
    wake: Arc<Condvar>,
    failures: Arc<Mutex<VecDeque<NativeFailure>>>,
}
struct NativeFailure {
    subject: String,
    generation: u64,
    notification_id: String,
    attempt_id: String,
    attempt_count: u32,
    code: String,
}
impl NotificationService {
    pub fn start(
        runtime: IntegrationRuntime,
        sink: Arc<dyn NotificationSink>,
        notify: Arc<dyn Fn() + Send + Sync>,
    ) -> Arc<Self> {
        let service = Arc::new(Self {
            runtime,
            sink,
            notify,
            owner: uuid::Uuid::new_v4().to_string(),
            stopped: AtomicBool::new(false),
            active: Mutex::new(false),
            wake: Arc::new(Condvar::new()),
            failures: Arc::default(),
        });
        let weak = Arc::downgrade(&service);
        thread::Builder::new()
            .name("notification-outbox".into())
            .spawn(move || {
                while let Some(service) = weak.upgrade() {
                    if service.stopped.load(Ordering::SeqCst) {
                        break;
                    }
                    if let Ok(mut active) = service.active.lock() {
                        if service.stopped.load(Ordering::SeqCst) {
                            break;
                        }
                        *active = true;
                    } else {
                        break;
                    }
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
            })
            .expect("notification outbox thread");
        service
    }
    pub fn wake(&self) {
        self.wake.notify_all();
    }
    pub fn shutdown(&self, timeout: Duration) -> bool {
        self.stopped.store(true, Ordering::SeqCst);
        self.wake.notify_all();
        let start = Instant::now();
        let Ok(mut active) = self.active.lock() else {
            return false;
        };
        while *active {
            let Some(remaining) = timeout.checked_sub(start.elapsed()) else {
                return false;
            };
            let Ok((next, waited)) = self.wake.wait_timeout(active, remaining) else {
                return false;
            };
            active = next;
            if waited.timed_out() {
                return !*active;
            }
        }
        true
    }
    fn pump(&self) -> Result<(), CommandError> {
        let mut changed = self.reconcile_failures()?;
        let state = self.runtime.auth.state()?;
        let Some(account) = state.auth.account else {
            return Ok(());
        };
        if state.auth.status != "connected" {
            return Ok(());
        }
        // A bounded batch keeps shutdown responsive under a burst of events.
        for _ in 0..10 {
            if self.stopped.load(Ordering::SeqCst) {
                break;
            }
            let dispatched = self.runtime.auth.with_generation(state.generation, || {
                let now = chrono::Utc::now().timestamp_millis();
                let mut db = Database::open(&self.runtime.database_path)
                    .map_err(|_| CommandError::local())?;
                db.expire_notifications(&account.subject, now)?;
                db.suppress_disabled_notifications(&account.subject, now)?;
                let Some(attempt) = db.claim_notification(&account.subject, &self.owner, now)?
                else {
                    return Ok::<bool, CommandError>(false);
                };
                if self.stopped.load(Ordering::SeqCst) {
                    return Ok(false);
                }
                // Lock the generation through the short native submission.
                let failure_queue = self.failures.clone();
                let failure_wake = self.wake.clone();
                let failure_subject = account.subject.clone();
                let failure_id = attempt.record.id.clone();
                let failure_attempt = attempt.attempt_id.clone();
                let failure_generation = state.generation;
                let failure_count = attempt.record.attempt_count;
                let failed = Arc::new(move |code: String| {
                    // WinRT may call Failed during Show. Never re-enter the auth
                    // mutex here: the dispatcher still owns its generation guard.
                    // Native tracking is bounded and each delegate reports once.
                    failure_queue
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .push_back(NativeFailure {
                            subject: failure_subject.clone(),
                            generation: failure_generation,
                            notification_id: failure_id.clone(),
                            attempt_id: failure_attempt.clone(),
                            attempt_count: failure_count,
                            code,
                        });
                    failure_wake.notify_all();
                });
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    self.sink.submit_with_failure(&attempt.record, failed)
                }))
                .unwrap_or_else(|_| NotificationOutcome::Retryable {
                    code: "native_prepare_failed".into(),
                    next_attempt_ms: now.saturating_add(crate::monitoring::retry_delay_ms(
                        attempt.record.attempt_count,
                        rand::random::<u64>() % 1001,
                        None,
                    )),
                });
                db.finish_notification_attempt(
                    &account.subject,
                    &attempt.record.id,
                    &attempt.attempt_id,
                    outcome,
                    chrono::Utc::now().timestamp_millis(),
                )?;
                Ok(true)
            })??;
            changed |= dispatched;
            if !dispatched {
                break;
            }
        }
        changed |= self.reconcile_failures()?;
        if changed {
            (self.notify)();
        }
        Ok(())
    }
    fn reconcile_failures(&self) -> Result<bool, CommandError> {
        let mut receipts =
            std::mem::take(&mut *self.failures.lock().map_err(|_| CommandError::local())?);
        let mut changed = false;
        while let Some(receipt) = receipts.pop_front() {
            let now = chrono::Utc::now().timestamp_millis();
            let result = self.runtime.auth.with_generation(receipt.generation, || {
                let mut db = Database::open(&self.runtime.database_path)
                    .map_err(|_| CommandError::local())?;
                db.retry_native_submission(
                    &receipt.subject,
                    &receipt.notification_id,
                    &receipt.attempt_id,
                    &receipt.code,
                    now.saturating_add(crate::monitoring::retry_delay_ms(
                        receipt.attempt_count,
                        rand::random::<u64>() % 1001,
                        None,
                    )),
                    now,
                )
            });
            // Logout/account changes cannot modify another session's journal.
            if let Ok(result) = result {
                match result {
                    Ok(updated) => changed |= updated,
                    Err(error) => {
                        let mut queue = self.failures.lock().map_err(|_| CommandError::local())?;
                        queue.push_back(receipt);
                        queue.extend(receipts);
                        return Err(error);
                    }
                }
            }
        }
        Ok(changed)
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
