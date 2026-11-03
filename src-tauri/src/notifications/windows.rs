//! Direct WinRT submission with observable errors and bounded event lifetimes.
use super::NativeReadiness;
use crate::monitoring::{NotificationOutcome, NotificationRecord};
use std::{
    cell::OnceCell,
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use windows::{
    core::{Error, IInspectable, HSTRING},
    Data::Xml::Dom::XmlDocument,
    Foundation::TypedEventHandler,
    Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED},
    UI::Notifications::{
        NotificationSetting, ToastDismissalReason, ToastDismissedEventArgs, ToastFailedEventArgs,
        ToastNotification, ToastNotificationManager,
    },
};

const DEVELOPMENT_ID: &str =
    "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe";
const TRACKING_LIMIT: usize = 128;
const TRACKING_TTL: Duration = Duration::from_secs(15 * 60);
type Failure = Arc<dyn Fn(String) + Send + Sync>;

struct Apartment(bool);
impl Drop for Apartment {
    fn drop(&mut self) {
        if self.0 {
            unsafe {
                RoUninitialize();
            }
        }
    }
}
thread_local! { static APARTMENT: OnceCell<Result<Apartment,String>> = const { OnceCell::new() }; }
fn apartment() -> Result<(), String> {
    APARTMENT.with(|slot| {
        slot.get_or_init(|| {
            // Initialize once per caller thread; an existing STA is also usable.
            match unsafe { RoInitialize(RO_INIT_MULTITHREADED) } {
                Ok(()) => Ok(Apartment(true)),
                Err(error) if error.code().0 as u32 == 0x80010106 => Ok(Apartment(false)),
                Err(error) => Err(code(&error)),
            }
        })
        .as_ref()
        .map(|_| ())
        .map_err(Clone::clone)
    })
}
fn code(error: &Error) -> String {
    format!("native_hresult_{:08x}", error.code().0 as u32)
}
fn code_at(stage: &str, error: &Error) -> String {
    format!("native_{stage}_hresult_{:08x}", error.code().0 as u32)
}
fn retry(record: &NotificationRecord, error: String) -> NotificationOutcome {
    NotificationOutcome::Retryable {
        code: error,
        next_attempt_ms: chrono::Utc::now().timestamp_millis().saturating_add(
            crate::monitoring::retry_delay_ms(
                record.attempt_count,
                rand::random::<u64>() % 1001,
                None,
            ),
        ),
    }
}
fn identity(configured: &str, development: bool) -> (&str, &'static str) {
    if development {
        (DEVELOPMENT_ID, "development_powershell")
    } else {
        (configured, "configured_application")
    }
}
fn setting_name(setting: NotificationSetting) -> &'static str {
    match setting {
        NotificationSetting::Enabled => "enabled",
        NotificationSetting::DisabledForApplication => "disabled_for_application",
        NotificationSetting::DisabledForUser => "disabled_for_user",
        NotificationSetting::DisabledByGroupPolicy => "disabled_by_group_policy",
        NotificationSetting::DisabledByManifest => "disabled_by_manifest",
        _ => "unknown",
    }
}
fn permits_submission(setting: windows::core::Result<NotificationSetting>) -> Result<bool, String> {
    match setting {
        Ok(NotificationSetting::Enabled) => Ok(true),
        Ok(
            NotificationSetting::DisabledForApplication
            | NotificationSetting::DisabledForUser
            | NotificationSetting::DisabledByGroupPolicy
            | NotificationSetting::DisabledByManifest,
        ) => Ok(false),
        // A read-only probe verified notifier creation succeeds but this getter
        // returns ElementNotFound for the installed development identity. It is
        // unknown permission, not a denial; Show/Failed still observe real errors.
        Err(error) if error.code().0 as u32 == 0x80070490 => Ok(true),
        Err(error) => Err(code_at("setting", &error)),
        Ok(_) => Err("native_setting_unknown".into()),
    }
}
struct Tracked {
    toast: ToastNotification,
    activated: Option<i64>,
    failed: Option<i64>,
    dismissed: Option<i64>,
    born: Instant,
    complete: Arc<AtomicBool>,
}
impl Drop for Tracked {
    fn drop(&mut self) {
        if let Some(token) = self.activated {
            let _ = self.toast.RemoveActivated(token);
        }
        if let Some(token) = self.failed {
            let _ = self.toast.RemoveFailed(token);
        }
        if let Some(token) = self.dismissed {
            let _ = self.toast.RemoveDismissed(token);
        }
    }
}
pub struct WindowsNotifier {
    configured: String,
    open: Arc<dyn Fn(String) + Send + Sync>,
    retained: Mutex<BTreeMap<String, Tracked>>,
}
impl WindowsNotifier {
    pub fn new(configured: String, open: Arc<dyn Fn(String) + Send + Sync>) -> Self {
        Self {
            configured,
            open,
            retained: Mutex::new(BTreeMap::new()),
        }
    }
    pub fn readiness(&self) -> NativeReadiness {
        let (id, mode) = identity(&self.configured, cfg!(debug_assertions));
        let result = (|| {
            apartment()?;
            let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(id))
                .map_err(|e| code_at("create", &e))?;
            notifier
                .Setting()
                .map(setting_name)
                .map_err(|e| code_at("setting", &e))
        })();
        match result {
            Ok(setting) => NativeReadiness {
                identity_mode: mode.into(),
                setting: setting.into(),
                error_code: None,
            },
            Err(error) => NativeReadiness {
                identity_mode: mode.into(),
                setting: "unknown".into(),
                error_code: Some(error),
            },
        }
    }
    pub fn submit(
        &self,
        record: &NotificationRecord,
        title: &str,
        body: &str,
        failure: Failure,
    ) -> NotificationOutcome {
        if let Err(error) = apartment() {
            return retry(record, error);
        }
        let (id, _) = identity(&self.configured, cfg!(debug_assertions));
        let notifier = match ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(id))
        {
            Ok(value) => value,
            Err(error) => return retry(record, code(&error)),
        };
        match permits_submission(notifier.Setting()) {
            Ok(true) => {}
            Ok(false) => return NotificationOutcome::PermissionBlocked,
            Err(error) => return retry(record, error),
        }
        let title = if cfg!(debug_assertions) {
            format!("[Desenvolvimento] {title}")
        } else {
            title.into()
        };
        let xml = match make_xml(&title, body) {
            Ok(value) => value,
            Err(error) => return retry(record, code(&error)),
        };
        let toast = match ToastNotification::CreateToastNotification(&xml) {
            Ok(value) => value,
            Err(error) => return retry(record, code(&error)),
        };
        let complete = Arc::new(AtomicBool::new(false));
        let mut tracked = Tracked {
            toast: toast.clone(),
            activated: None,
            failed: None,
            dismissed: None,
            born: Instant::now(),
            complete: complete.clone(),
        };
        let open = self.open.clone();
        let opaque = record.id.clone();
        let activated_complete = complete.clone();
        tracked.activated = match toast.Activated(&TypedEventHandler::<
            ToastNotification,
            IInspectable,
        >::new(move |_, _| {
            if activated_complete
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                let _ =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| open(opaque.clone())));
            }
            Ok(())
        })) {
            Ok(token) => Some(token),
            Err(error) => return retry(record, code(&error)),
        };
        let failed_complete = complete.clone();
        tracked.failed = match toast.Failed(&TypedEventHandler::<
            ToastNotification,
            ToastFailedEventArgs,
        >::new(move |_, args| {
            if failed_complete
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                return Ok(());
            }
            let error = args
                .as_ref()
                .and_then(|value| value.ErrorCode().ok())
                .map(|value| format!("native_hresult_{:08x}", value.0 as u32))
                .unwrap_or_else(|| "native_async_failed".into());
            // The supplied callback only queues a receipt; never take auth/DB locks here.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| failure(error)));
            Ok(())
        })) {
            Ok(token) => Some(token),
            Err(error) => return retry(record, code(&error)),
        };
        tracked.dismissed = match toast.Dismissed(&TypedEventHandler::<
            ToastNotification,
            ToastDismissedEventArgs,
        >::new(move |_, args| {
            // Keep timed-out banners eligible for activation while this running
            // process retains the delegate; the table and TTL remain bounded.
            if args.as_ref().and_then(|value| value.Reason().ok())
                != Some(ToastDismissalReason::TimedOut)
            {
                complete.store(true, Ordering::SeqCst);
            }
            Ok(())
        })) {
            Ok(token) => Some(token),
            Err(error) => return retry(record, code(&error)),
        };
        {
            let Ok(mut retained) = self.retained.lock() else {
                return retry(record, "native_tracking_unavailable".into());
            };
            retained.retain(|_, value| {
                !value.complete.load(Ordering::SeqCst) && value.born.elapsed() < TRACKING_TTL
            });
            if !retained.contains_key(&record.id) && retained.len() >= TRACKING_LIMIT {
                return retry(record, "native_tracking_capacity".into());
            }
            retained.insert(record.id.clone(), tracked);
        }
        // Show errors are now observed synchronously; Ok remains submission, never display/read.
        match notifier.Show(&toast) {
            Ok(()) => NotificationOutcome::DispatchSubmitted,
            Err(error) => {
                if let Ok(mut retained) = self.retained.lock() {
                    retained.remove(&record.id);
                }
                retry(record, code(&error))
            }
        }
    }
}
fn make_xml(title: &str, body: &str) -> windows::core::Result<XmlDocument> {
    let document = XmlDocument::new()?;
    // DOM text nodes escape XML; payload never becomes markup, URL or a command.
    let toast = document.CreateElement(&HSTRING::from("toast"))?;
    let visual = document.CreateElement(&HSTRING::from("visual"))?;
    let binding = document.CreateElement(&HSTRING::from("binding"))?;
    binding.SetAttribute(&HSTRING::from("template"), &HSTRING::from("ToastGeneric"))?;
    for value in [title, body] {
        let text = document.CreateElement(&HSTRING::from("text"))?;
        text.AppendChild(&document.CreateTextNode(&HSTRING::from(value))?)?;
        binding.AppendChild(&text)?;
    }
    visual.AppendChild(&binding)?;
    toast.AppendChild(&visual)?;
    document.AppendChild(&toast)?;
    Ok(document)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_is_build_policy_and_never_directory_heuristic_or_denial_fallback() {
        assert_eq!(
            identity("configured.release", true),
            (DEVELOPMENT_ID, "development_powershell")
        );
        assert_eq!(
            identity("configured.release", false),
            ("configured.release", "configured_application")
        );
    }
    #[test]
    fn readiness_setting_mapping_is_explicit_without_sending() {
        assert_eq!(setting_name(NotificationSetting::Enabled), "enabled");
        assert_eq!(
            setting_name(NotificationSetting::DisabledForUser),
            "disabled_for_user"
        );
        assert_eq!(
            setting_name(NotificationSetting::DisabledForApplication),
            "disabled_for_application"
        );
    }
    #[test]
    fn hresult_codes_are_sanitized_without_windows_message_data() {
        assert_eq!(
            code(&Error::from_hresult(windows::core::HRESULT(
                0x80070490u32 as i32
            ))),
            "native_hresult_80070490"
        );
    }
    #[test]
    fn in_memory_dom_treats_notification_content_as_text_without_showing() {
        apartment().unwrap();
        let document = make_xml("<unsafe> & title", "</text><audio src='arbitrary'/>").unwrap();
        let xml = document.GetXml().unwrap().to_string();
        assert!(xml.contains("&lt;unsafe&gt; &amp; title"));
        assert!(xml.contains("&lt;/text&gt;&lt;audio"));
        assert!(!xml.contains("<audio"));
    }
    #[test]
    fn missing_setting_is_unknown_but_no_known_block_or_other_error_is_bypassed() {
        let missing = Error::from_hresult(windows::core::HRESULT(0x80070490u32 as i32));
        assert_eq!(permits_submission(Err(missing)), Ok(true));
        for disabled in [
            NotificationSetting::DisabledForApplication,
            NotificationSetting::DisabledForUser,
            NotificationSetting::DisabledByGroupPolicy,
            NotificationSetting::DisabledByManifest,
        ] {
            assert_eq!(permits_submission(Ok(disabled)), Ok(false));
        }
        let denied = Error::from_hresult(windows::core::HRESULT(0x80070005u32 as i32));
        assert_eq!(
            permits_submission(Err(denied)),
            Err("native_setting_hresult_80070005".into())
        );
        assert_eq!(
            permits_submission(Ok(NotificationSetting::Enabled)),
            Ok(true)
        );
    }
}
