//! Shared admission and operation-scoped cancellation. Independent databases do not contend.
use crate::{integration::CommandError, monitoring::MonitoringLease};
use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock, Weak,
    },
    time::{Duration, Instant},
};
#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
#[derive(Default)]
pub struct WorkCoordinator {
    active: Mutex<HashMap<String, Cancellation>>,
}
static REGISTRY: OnceLock<Mutex<HashMap<PathBuf, Weak<WorkCoordinator>>>> = OnceLock::new();
impl WorkCoordinator {
    pub fn for_database(path: &Path) -> Arc<Self> {
        let absolute = std::fs::canonicalize(path).unwrap_or_else(|_| {
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                std::env::current_dir().unwrap_or_default().join(path)
            }
        });
        let key = PathBuf::from(absolute.to_string_lossy().to_lowercase());
        let mut registry = REGISTRY
            .get_or_init(Mutex::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(existing) = registry.get(&key).and_then(Weak::upgrade) {
            return existing;
        }
        let result = Arc::new(Self::default());
        registry.insert(key, Arc::downgrade(&result));
        result
    }
    pub fn acquire(
        self: &Arc<Self>,
        id: &str,
        cancel: Cancellation,
    ) -> Result<WorkPermit, CommandError> {
        let mut active = self.active.lock().map_err(|_| CommandError::local())?;
        if active.len() >= 3 || active.contains_key(id) {
            return Err(CommandError {
                code: "integrity_busy".into(),
                message: "A verificação está na fila de execução.".into(),
                retryable: true,
            });
        }
        active.insert(id.into(), cancel.clone());
        Ok(WorkPermit {
            coordinator: self.clone(),
            id: id.into(),
            cancel,
        })
    }
    pub fn active_count(&self) -> usize {
        self.active.lock().map(|a| a.len()).unwrap_or(3)
    }
    pub fn cancel_all(&self) {
        if let Ok(active) = self.active.lock() {
            for token in active.values() {
                token.cancel();
            }
        }
    }
}
pub struct WorkPermit {
    coordinator: Arc<WorkCoordinator>,
    id: String,
    pub cancel: Cancellation,
}
impl Drop for WorkPermit {
    fn drop(&mut self) {
        if let Ok(mut active) = self.coordinator.active.lock() {
            active.remove(&self.id);
        }
    }
}
#[derive(Clone)]
struct Context {
    cancel: Cancellation,
    lease: Option<MonitoringLease>,
    deadline: Instant,
    retry_after_ms: Option<i64>,
}
thread_local! {static CONTEXT:RefCell<Option<Context>>=const{RefCell::new(None)};}
struct Restore(Option<Context>);
impl Drop for Restore {
    fn drop(&mut self) {
        CONTEXT.with(|c| *c.borrow_mut() = self.0.take());
    }
}
pub fn in_operation<T>(
    cancel: Cancellation,
    lease: Option<MonitoringLease>,
    operation: impl FnOnce() -> T,
) -> (T, Option<i64>) {
    let old = CONTEXT.with(|c| {
        c.replace(Some(Context {
            cancel,
            lease,
            deadline: Instant::now() + Duration::from_secs(180),
            retry_after_ms: None,
        }))
    });
    let _restore = Restore(old);
    let result = operation();
    let hint = CONTEXT.with(|c| c.borrow().as_ref().and_then(|c| c.retry_after_ms));
    (result, hint)
}
pub fn cancelled() -> bool {
    CONTEXT.with(|c| {
        c.borrow()
            .as_ref()
            .is_some_and(|c| c.cancel.is_cancelled() || Instant::now() >= c.deadline)
    })
}
pub fn checkpoint() -> Result<(), CommandError> {
    CONTEXT.with(|c| {
        let context = c.borrow();
        if let Some(context) = context.as_ref() {
            if context.cancel.is_cancelled() {
                return Err(CommandError {
                    code: "cancelled".into(),
                    message: "Verificação cancelada; resultado descartado.".into(),
                    retryable: false,
                });
            }
            if Instant::now() >= context.deadline {
                return Err(CommandError {
                    code: "timeout".into(),
                    message: "A verificação excedeu o prazo operacional.".into(),
                    retryable: true,
                });
            }
        }
        Ok(())
    })
}
pub fn current_lease() -> Option<MonitoringLease> {
    CONTEXT.with(|c| c.borrow().as_ref().and_then(|c| c.lease.clone()))
}
pub fn defer_retry(duration: Duration) {
    CONTEXT.with(|c| {
        if let Some(c) = c.borrow_mut().as_mut() {
            let value = duration.as_millis().min(i64::MAX as u128) as i64;
            c.retry_after_ms = Some(c.retry_after_ms.unwrap_or(0).max(value));
        }
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn global_three_manual_and_auto_and_per_file_gate_release() {
        let dir = tempfile::tempdir().unwrap();
        let a = WorkCoordinator::for_database(&dir.path().join("db"));
        let b = WorkCoordinator::for_database(&dir.path().join("db"));
        let one = a.acquire("a", Cancellation::default()).unwrap();
        assert!(b.acquire("a", Cancellation::default()).is_err());
        let two = b.acquire("b", Cancellation::default()).unwrap();
        let three = a.acquire("c", Cancellation::default()).unwrap();
        assert!(b.acquire("d", Cancellation::default()).is_err());
        drop(one);
        assert!(b.acquire("d", Cancellation::default()).is_ok());
        drop((two, three));
        assert_eq!(a.active_count(), 0);
    }
    #[test]
    fn contexts_reset_after_error_and_panic_and_databases_are_independent() {
        let token = Cancellation::default();
        token.cancel();
        let (_, hint) = in_operation(token, None, || {
            assert!(cancelled());
            defer_retry(Duration::from_secs(900));
        });
        assert_eq!(hint, Some(900000));
        assert!(!cancelled());
        let _ = std::panic::catch_unwind(|| {
            in_operation(Cancellation::default(), None, || {
                defer_retry(Duration::from_secs(99));
                panic!("fixture")
            })
        });
        assert_eq!(in_operation(Cancellation::default(), None, || ()).1, None);
        let a = WorkCoordinator::for_database(Path::new("fixture-a.db"));
        let b = WorkCoordinator::for_database(Path::new("fixture-b.db"));
        let _p = a.acquire("same", Cancellation::default()).unwrap();
        assert!(b.acquire("same", Cancellation::default()).is_ok());
    }
}
