use super::{
    work::{Cancellation, WorkCoordinator},
    *,
};
use crate::{
    auth::{SCRIPT_SCOPE, SHEETS_SCOPE},
    database::Database,
    integration::{CommandError, IntegrationRuntime},
    management::Mutation,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex, OnceLock, Weak,
    },
    thread,
    time::{Duration, Instant},
};
#[derive(Clone, Default, Serialize)]
pub struct ServiceStatus {
    pub running: bool,
    pub active: usize,
    pub queued: usize,
    pub warning: Option<String>,
}
struct State {
    subject: Option<(String, u64)>,
    last_tick: i64,
    interval: i64,
    active: HashMap<String, (Cancellation, bool)>,
    manual: BTreeMap<String, (String, u64)>,
    warning: Option<String>,
    control_generation: Option<u64>,
    known: HashMap<String, (u64, String)>,
    maintenance_ms: i64,
    pending_count: usize,
}
pub trait CheckExecutor: Send + Sync {
    fn execute(
        &self,
        runtime: &IntegrationRuntime,
        input: Mutation,
    ) -> Result<
        crate::integrity::runtime::EngineEnvelope<crate::integrity::runtime::ValidationResult>,
        CommandError,
    >;
}
pub struct IntegrityCheckExecutor;
impl CheckExecutor for IntegrityCheckExecutor {
    fn execute(
        &self,
        runtime: &IntegrationRuntime,
        input: Mutation,
    ) -> Result<
        crate::integrity::runtime::EngineEnvelope<crate::integrity::runtime::ValidationResult>,
        CommandError,
    > {
        runtime.run_spreadsheet_check_inner(input)
    }
}
pub struct MonitoringService {
    executor: Arc<dyn CheckExecutor>,
    runtime: IntegrationRuntime,
    clock: Arc<dyn MonitoringClock>,
    notify: Arc<dyn Fn() + Send + Sync>,
    coordinator: Arc<WorkCoordinator>,
    owner: String,
    stopped: AtomicBool,
    state: Mutex<State>,
    wake: Condvar,
}
static SERVICES: OnceLock<Mutex<HashMap<PathBuf, Weak<MonitoringService>>>> = OnceLock::new();
impl MonitoringService {
    pub fn start(runtime: IntegrationRuntime, notify: Arc<dyn Fn() + Send + Sync>) -> Arc<Self> {
        Self::with_clock(runtime, notify, Arc::new(SystemMonitoringClock))
    }
    pub fn with_clock(
        runtime: IntegrationRuntime,
        notify: Arc<dyn Fn() + Send + Sync>,
        clock: Arc<dyn MonitoringClock>,
    ) -> Arc<Self> {
        Self::start_internal(runtime, notify, clock, Arc::new(IntegrityCheckExecutor))
    }
    /// Isolated native QA only. No IPC selects executors or fixture accounts.
    #[cfg(feature = "native-probe")]
    pub fn start_with_executor(
        runtime: IntegrationRuntime,
        notify: Arc<dyn Fn() + Send + Sync>,
        clock: Arc<dyn MonitoringClock>,
        executor: Arc<dyn CheckExecutor>,
    ) -> Arc<Self> {
        Self::start_internal(runtime, notify, clock, executor)
    }
    fn start_internal(
        runtime: IntegrationRuntime,
        notify: Arc<dyn Fn() + Send + Sync>,
        clock: Arc<dyn MonitoringClock>,
        executor: Arc<dyn CheckExecutor>,
    ) -> Arc<Self> {
        let path = std::fs::canonicalize(&runtime.database_path)
            .unwrap_or_else(|_| runtime.database_path.clone());
        let mut services = SERVICES
            .get_or_init(Mutex::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(existing) = services
            .get(&path)
            .and_then(Weak::upgrade)
            .filter(|s| !s.stopped.load(Ordering::SeqCst))
        {
            return existing;
        }
        let service = Arc::new(Self {
            executor,
            coordinator: WorkCoordinator::for_database(&runtime.database_path),
            runtime,
            clock,
            notify,
            owner: uuid::Uuid::new_v4().to_string(),
            stopped: AtomicBool::new(false),
            state: Mutex::new(State {
                subject: None,
                last_tick: 0,
                interval: 60_000,
                active: HashMap::new(),
                manual: BTreeMap::new(),
                warning: None,
                control_generation: None,
                known: HashMap::new(),
                maintenance_ms: 0,
                pending_count: 0,
            }),
            wake: Condvar::new(),
        });
        services.insert(path, Arc::downgrade(&service));
        let weak = Arc::downgrade(&service);
        let coordinator_started = thread::Builder::new()
            .name("monitoring-coordinator".into())
            .spawn(move || {
                while let Some(service) = weak.upgrade() {
                    if service.stopped.load(Ordering::SeqCst) {
                        break;
                    }
                    if service.tick().is_err() {
                        if let Ok(mut state) = service.state.lock() {
                            state.warning = Some("local_storage".into());
                        }
                    }
                    if let Ok(state) = service.state.lock() {
                        let _ = service.wake.wait_timeout(state, Duration::from_millis(500));
                    }
                }
            });
        if coordinator_started.is_err() {
            service.stopped.store(true, Ordering::SeqCst);
            if let Ok(mut state) = service.state.lock() {
                state.warning = Some("worker_unavailable".into());
            }
        }
        service
    }
    pub fn wake(&self) {
        self.wake.notify_all();
    }
    pub fn cancel_active(&self) {
        if let Ok(state) = self.state.lock() {
            for (token, _) in state.active.values() {
                token.cancel();
            }
        }
        self.wake();
    }
    pub fn request_check(&self, id: String) -> Result<(), CommandError> {
        // Resolve the identity before taking scheduler state; callers inside an
        // auth guard use request_check_for to avoid re-entering that guard.
        let session = self.runtime.auth.state()?;
        let account = session
            .auth
            .account
            .ok_or_else(|| crate::auth::AuthError::new(crate::auth::AuthCode::ReconnectRequired))?;
        self.request_check_for(&account.subject, session.generation, id)
    }
    pub fn request_check_for(
        &self,
        subject: &str,
        generation: u64,
        id: String,
    ) -> Result<(), CommandError> {
        crate::google::validate_google_id(&id)?;
        let mut state = self.state.lock().map_err(|_| CommandError::local())?;
        let already_active =
            state
                .subject
                .as_ref()
                .is_some_and(|(active_subject, active_generation)| {
                    active_subject == subject && *active_generation == generation
                })
                && state.active.contains_key(&id);
        if !self.stopped.load(Ordering::SeqCst) && !already_active {
            state.manual.insert(id, (subject.to_owned(), generation));
        }
        drop(state);
        self.wake();
        Ok(())
    }
    pub fn status(&self) -> ServiceStatus {
        self.state
            .lock()
            .map(|s| ServiceStatus {
                running: !self.stopped.load(Ordering::SeqCst),
                active: s.active.len(),
                queued: s.pending_count.max(s.manual.len()),
                warning: s.warning.clone(),
            })
            .unwrap_or_default()
    }
    /// Voluntary shutdown cancels work; an in-flight bounded GET may still drain.
    pub fn shutdown(&self, timeout: Duration) -> bool {
        self.stopped.store(true, Ordering::SeqCst);
        self.coordinator.cancel_all();
        self.cancel_active();
        let started = Instant::now();
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        state.manual.clear();
        while !state.active.is_empty() || self.coordinator.active_count() > 0 {
            let Some(remaining) = timeout.checked_sub(started.elapsed()) else {
                return false;
            };
            let Ok((next, result)) = self
                .wake
                .wait_timeout(state, remaining.min(Duration::from_millis(50)))
            else {
                return false;
            };
            state = next;
            if result.timed_out() && started.elapsed() >= timeout {
                return state.active.is_empty() && self.coordinator.active_count() == 0;
            }
        }
        true
    }
    fn tick(self: &Arc<Self>) -> Result<(), CommandError> {
        let auth = self.runtime.auth.state()?;
        let now = self.clock.now_ms();
        let identity = auth
            .auth
            .account
            .as_ref()
            .map(|a| (a.subject.clone(), auth.generation));
        let mut state = self.state.lock().map_err(|_| CommandError::local())?;
        if state.subject != identity {
            for (token, _) in state.active.values() {
                token.cancel();
            }
            state.subject = identity.clone();
            state.last_tick = 0;
            state.known.clear();
            state.maintenance_ms = 0;
        }
        state
            .manual
            .retain(|_, (queued_subject, queued_generation)| {
                identity.as_ref().is_some_and(|(subject, generation)| {
                    subject == queued_subject && generation == queued_generation
                })
            });
        if auth.auth.status != "connected"
            || !auth.auth.granted_scopes.iter().any(|s| s == SHEETS_SCOPE)
        {
            state.warning = Some("authentication_required".into());
            return Ok(());
        }
        let Some((subject, generation)) = identity else {
            return Ok(());
        };
        let mut db =
            Database::open(&self.runtime.database_path).map_err(|_| CommandError::local())?;
        let control = db.monitoring_control(&subject)?;
        let interval = interval_ms(db.management_preferences(&subject)?.interval_seconds);
        let mut candidates = db.monitoring_candidates(&subject)?;
        candidates.sort_by(|a, b| a.spreadsheet_id.cmp(&b.spreadsheet_id));
        let needs_script = candidates.iter().any(|c| !c.scope.scripts.is_empty());
        let script_allowed = auth.auth.granted_scopes.iter().any(|s| s == SCRIPT_SCOPE);
        state.warning = if needs_script && !script_allowed {
            Some("insufficient_scope".into())
        } else {
            None
        };
        candidates.retain(|c| c.scope.scripts.is_empty() || script_allowed);
        let recover = state.last_tick == 0;
        let generation_changed = state.control_generation != Some(control.generation);
        if generation_changed {
            for (token, _) in state.active.values() {
                token.cancel();
            }
        }
        let replan = recover
            || generation_changed
            || state.interval != interval
            || resumed_after_gap(state.last_tick, now, interval);
        state.control_generation = Some(control.generation);
        if recover {
            db.recover_monitoring_leases(&subject, &self.owner, now)?;
        }
        let previous_schedules = db.monitoring_schedules(&subject)?;
        for (i, candidate) in candidates.iter().enumerate() {
            let next = stagger_ms(now, i, candidates.len(), interval);
            let next = previous_schedules
                .iter()
                .find(|r| {
                    r.spreadsheet_id == candidate.spreadsheet_id
                        && r.registry_revision == candidate.registry_revision
                        && r.scope_hash == candidate.scope_hash
                        && r.failure_count > 0
                })
                .map_or(next, |r| next.max(r.next_run_ms));
            let identity = (candidate.registry_revision, candidate.scope_hash.clone());
            if replan || state.known.get(&candidate.spreadsheet_id) != Some(&identity) {
                db.ensure_monitoring_schedule(&subject, candidate, next)?;
                state
                    .known
                    .insert(candidate.spreadsheet_id.clone(), identity);
            }
            if replan {
                db.defer_monitoring_schedule(&subject, &candidate.spreadsheet_id, next)?;
            }
        }
        if state.maintenance_ms == 0 || now.saturating_sub(state.maintenance_ms) >= 3_600_000 {
            db.expire_notifications(&subject, now)?;
            db.prune_monitoring_history(&subject, now)?;
            state.maintenance_ms = now;
        }
        state.last_tick = now;
        state.interval = interval;
        if !control.enabled {
            for (token, manual) in state.active.values() {
                if !manual {
                    token.cancel();
                }
            }
        }
        let schedules = db.monitoring_schedules(&subject)?;
        if schedules.iter().any(|row| {
            row.failure_count == 0
                && ((row.next_run_ms.saturating_add(interval) < now)
                    || row
                        .last_completed_ms
                        .zip(row.last_started_ms)
                        .is_some_and(|(end, start)| end.saturating_sub(start) > interval))
        }) {
            state.warning = Some("interval_capacity_exceeded".into());
        }
        let mut due = Vec::new();
        for row in schedules {
            if let Some(candidate) = candidates
                .iter()
                .find(|c| c.spreadsheet_id == row.spreadsheet_id)
            {
                let manual = state.manual.contains_key(&row.spreadsheet_id);
                if ((manual && !(row.failure_count > 0 && row.next_run_ms > now))
                    || (control.enabled && row.next_run_ms <= now))
                    && !state.active.contains_key(&row.spreadsheet_id)
                {
                    due.push((row, candidate.clone(), manual));
                }
            }
        }
        due.sort_by_key(|(row, _, manual)| (!*manual, row.next_run_ms));
        state.pending_count = due.len();
        let mut started_any = false;
        for (row, candidate, manual) in due {
            if state.active.len() >= 3 || self.coordinator.active_count() >= 3 {
                break;
            }
            let Some(lease) = db.claim_monitoring_check(
                &subject,
                &row.spreadsheet_id,
                candidate.registry_revision,
                control.generation,
                &self.owner,
                now,
                300_000,
                manual,
            )?
            else {
                continue;
            };
            let token = Cancellation::default();
            state
                .active
                .insert(row.spreadsheet_id.clone(), (token.clone(), manual));
            state.manual.remove(&row.spreadsheet_id);
            let service = self.clone();
            let worker_subject = subject.clone();
            let cleanup_lease = lease.clone();
            let cleanup_id = row.spreadsheet_id.clone();
            let spawned = thread::Builder::new()
                .name("monitoring-check".into())
                .spawn(move || {
                    service.execute(
                        worker_subject,
                        generation,
                        candidate,
                        lease,
                        token,
                        interval,
                    )
                });
            if spawned.is_err() {
                state.active.remove(&cleanup_id);
                db.finish_monitoring_check(
                    &subject,
                    &cleanup_lease,
                    CheckOutcome::Cancelled,
                    now.saturating_add(1000),
                    now,
                )?;
                state.warning = Some("worker_unavailable".into());
            } else {
                started_any = true;
                state.pending_count = state.pending_count.saturating_sub(1);
            }
        }
        drop(state);
        if started_any {
            (self.notify)();
        }
        Ok(())
    }
    fn execute(
        self: Arc<Self>,
        subject: String,
        generation: u64,
        candidate: MonitoringCandidate,
        lease: MonitoringLease,
        token: Cancellation,
        interval: i64,
    ) {
        let start = self.clock.now_ms();
        let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let permit = self
                .coordinator
                .acquire(&candidate.spreadsheet_id, token.clone());
            match permit {
                Ok(_permit) => work::in_operation(token.clone(), Some(lease.clone()), || {
                    let result = self.executor.execute(
                        &self.runtime,
                        Mutation {
                            expected_generation: generation,
                            spreadsheet_id: candidate.spreadsheet_id.clone(),
                            expected_revision: candidate.registry_revision,
                        },
                    );
                    work::checkpoint()?;
                    result
                }),
                Err(error) => (Err(error), None),
            }
        }));
        let now = self.clock.now_ms();
        let (outcome, hint, contention) = match attempt {
            Ok((result, hint)) => {
                if token.is_cancelled() || self.stopped.load(Ordering::SeqCst) {
                    (CheckOutcome::Cancelled, hint, false)
                } else {
                    match result {
                        Ok(value)
                            if value.data.report.status
                                == crate::integrity::IntegrityStatus::Indeterminate
                                || !value.data.report.coverage.complete =>
                        {
                            let code = value
                                .data
                                .report
                                .coverage
                                .issues
                                .iter()
                                .find_map(|f| operational_code(&f.code))
                                .unwrap_or("incomplete_coverage");
                            (
                                CheckOutcome::OperationalFailure { code: code.into() },
                                hint,
                                false,
                            )
                        }
                        Ok(_) => (CheckOutcome::Success, None, false),
                        Err(error)
                            if matches!(
                                error.code.as_str(),
                                "auth_busy"
                                    | "integrity_busy"
                                    | "cancelled"
                                    | "operation_superseded"
                                    | "configuration_conflict"
                                    | "configuration_pending"
                                    | "integrity_baseline_conflict"
                            ) =>
                        {
                            (CheckOutcome::Cancelled, hint, true)
                        }
                        Err(error) => (
                            CheckOutcome::OperationalFailure {
                                code: operational_code(&error.code)
                                    .unwrap_or("check_failed")
                                    .into(),
                            },
                            hint,
                            false,
                        ),
                    }
                }
            }
            Err(_) => (
                CheckOutcome::OperationalFailure {
                    code: "check_failed".into(),
                },
                None,
                false,
            ),
        };
        if let Ok(mut db) = Database::open(&self.runtime.database_path) {
            let failure_count = db
                .monitoring_schedules(&subject)
                .ok()
                .and_then(|rows| {
                    rows.into_iter()
                        .find(|r| r.spreadsheet_id == candidate.spreadsheet_id)
                })
                .map_or(0, |r| r.failure_count);
            let delay = if contention {
                1000
            } else if matches!(outcome, CheckOutcome::OperationalFailure { .. }) {
                interval.max(retry_delay_ms(failure_count, self.clock.jitter_ms(), hint))
            } else {
                interval
            };
            let committed = self.runtime.auth.with_generation(generation, || {
                db.finish_monitoring_check(
                    &subject,
                    &lease,
                    outcome,
                    now.saturating_add(delay),
                    now,
                )
            });
            if committed.is_err() {
                // Ownership-only release creates no event for an old session.
                let _ = db.finish_monitoring_check(
                    &subject,
                    &lease,
                    CheckOutcome::Cancelled,
                    now.saturating_add(delay),
                    now,
                );
            }
        }
        if let Ok(mut state) = self.state.lock() {
            state.active.remove(&candidate.spreadsheet_id);
            if now.saturating_sub(start) > interval {
                state.warning = Some("interval_capacity_exceeded".into());
            }
        }
        (self.notify)();
        self.wake.notify_all();
    }
}
fn operational_code(code: &str) -> Option<&str> {
    match code {
        "network"
        | "timeout"
        | "rate_limited"
        | "server"
        | "unauthorized"
        | "forbidden"
        | "not_found"
        | "insufficient_scope"
        | "reconnect_required"
        | "invalid_grant"
        | "local_storage"
        | "response_too_large"
        | "incomplete_coverage"
        | "check_failed" => Some(code),
        _ => None,
    }
}
#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "homologation_tests.rs"]
mod homologation_tests;
