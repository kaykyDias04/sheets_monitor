//! Optional account sync runs without the renderer, with bounded retry cadence.
use crate::{auth::CLOUD_SCOPE, integration::IntegrationRuntime};
use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};
#[derive(Default)]
pub struct CloudSyncService {
    stop: Mutex<bool>,
    wake: Condvar,
}
impl CloudSyncService {
    pub fn start(runtime: IntegrationRuntime, changed: Arc<dyn Fn() + Send + Sync>) -> Arc<Self> {
        let service = Arc::new(Self::default());
        let worker = service.clone();
        thread::spawn(move || {
            let mut failures = 0_u32;
            let mut generation = None;
            loop {
                if *worker.stop.lock().unwrap_or_else(|e| e.into_inner()) {
                    return;
                }
                let mut active = false;
                let mut server_wait = 0;
                if let Ok(session) = runtime.auth.state() {
                    if generation != Some(session.generation) {
                        generation = Some(session.generation);
                        failures = 0;
                    }
                    if session.auth.status == "connected"
                        && runtime.auth.features().is_ok_and(|f| f.0)
                        && session.auth.granted_scopes.iter().any(|s| s == CLOUD_SCOPE)
                    {
                        active = true;
                        let (result, hint) = crate::monitoring::work::in_operation(
                            crate::monitoring::work::Cancellation::default(),
                            None,
                            || runtime.sync_configuration(session.generation, None, None),
                        );
                        server_wait = (hint.unwrap_or(0).max(0) as u64).div_ceil(1000);
                        match result {
                            Ok(status) => {
                                failures = 0;
                                if status.data.state != "disabled" {
                                    changed();
                                }
                            }
                            Err(_) => {
                                failures = failures.saturating_add(1);
                                changed();
                            }
                        }
                    }
                }
                let delay = (if active { retry_seconds(failures) } else { 5 }).max(server_wait);
                let stop = worker.stop.lock().unwrap_or_else(|e| e.into_inner());
                if *stop {
                    return;
                }
                let (stop, _) = worker
                    .wake
                    .wait_timeout(stop, Duration::from_secs(delay))
                    .unwrap_or_else(|e| e.into_inner());
                if *stop {
                    return;
                }
            }
        });
        service
    }
    pub fn stop(&self) {
        *self.stop.lock().unwrap_or_else(|e| e.into_inner()) = true;
        self.wake.notify_all();
    }
}
fn retry_seconds(failures: u32) -> u64 {
    (60_u64.saturating_mul(1_u64 << failures.min(4))).min(900)
}
#[cfg(test)]
mod tests {
    #[test]
    fn sync_backoff_is_bounded_without_busy_looping() {
        assert_eq!(super::retry_seconds(0), 60);
        assert_eq!(super::retry_seconds(1), 120);
        assert_eq!(super::retry_seconds(4), 900);
        assert_eq!(super::retry_seconds(u32::MAX), 900);
    }
}
