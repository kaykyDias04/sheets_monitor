use super::*;
use crate::{
    google::{SheetMetadata, SpreadsheetMetadata},
    integrity::{evaluate, fixtures, BaselineVersion},
    management::{Mutation, Preferences, RemoveInput},
};
const SUBJECT: &str = "fixture_account";
const NOW: i64 = 1_800_000_000_000;
fn fixture() -> (Database, MonitoringCandidate, BaselineVersion) {
    let mut db = Database::memory().unwrap();
    db.upsert_account(SUBJECT, None, None).unwrap();
    db.upsert_account("other_account", None, None).unwrap();
    let metadata = SpreadsheetMetadata {
        spreadsheet_id: "fixture_sheet".into(),
        spreadsheet_url: None,
        title: "Synthetic Sheet".into(),
        locale: None,
        time_zone: None,
        sheets: vec![SheetMetadata {
            sheet_id: 7,
            title: "Data".into(),
            index: 0,
            sheet_type: Some("GRID".into()),
            row_count: Some(100),
            column_count: Some(8),
            hidden: false,
        }],
    };
    let r = db
        .register_operational_spreadsheet(SUBJECT, &metadata)
        .unwrap();
    let m = |revision| Mutation {
        spreadsheet_id: "fixture_sheet".into(),
        expected_generation: 0,
        expected_revision: revision,
    };
    let r = db
        .set_registered_tabs(SUBJECT, &m(r.revision), &[7])
        .unwrap();
    let r = db
        .set_registered_rules(SUBJECT, &m(r.revision), &fixtures::scope())
        .unwrap();
    let s = fixtures::snapshot();
    db.save_structural_snapshot(&s).unwrap();
    let b = db
        .approve_registered_structural_baseline(
            SUBJECT,
            &s.id,
            0,
            true,
            "synthetic:test",
            Some(r.revision),
        )
        .unwrap();
    let c = db.monitoring_candidates(SUBJECT).unwrap().remove(0);
    db.ensure_monitoring_schedule(SUBJECT, &c, NOW).unwrap();
    (db, c, b)
}
fn lease(db: &mut Database, c: &MonitoringCandidate, now: i64) -> MonitoringLease {
    let ctl = db.monitoring_control(SUBJECT).unwrap();
    db.claim_monitoring_check(
        SUBJECT,
        &c.spreadsheet_id,
        c.registry_revision,
        ctl.generation,
        "owner",
        now,
        300_000,
        true,
    )
    .unwrap()
    .unwrap()
}
fn preference(db: &mut Database, on: bool) {
    db.save_management_preferences(
        SUBJECT,
        &Preferences {
            notifications_enabled: on,
            ..Default::default()
        },
    )
    .unwrap();
}
fn failure(db: &mut Database, c: &MonitoringCandidate, code: &str, now: i64) {
    let l = lease(db, c, now);
    assert!(db
        .finish_monitoring_check(
            SUBJECT,
            &l,
            CheckOutcome::OperationalFailure { code: code.into() },
            now + 60_000,
            now + 1
        )
        .unwrap());
}
fn integrity_event(
    db: &mut Database,
    b: &BaselineVersion,
    check: &str,
    now: i64,
) -> crate::integrity::EngineResult<Vec<crate::integrity::IntegrityEvent>> {
    let mut s = fixtures::snapshot();
    s.id = format!("snapshot-{check}");
    s.sheets[0].title = "Changed fixture title".into();
    db.save_structural_snapshot(&s).unwrap();
    let obs = db
        .content_observation(SUBJECT, &s.spreadsheet_id, &s.scope_hash)
        .unwrap();
    let mut report = evaluate(b, obs.snapshot.as_ref(), &s, check).unwrap();
    report.observation_revision = obs.revision;
    report.previous_snapshot_id = db
        .previous_structural_snapshot(SUBJECT, &s.spreadsheet_id, &s.scope_hash)
        .unwrap()
        .map(|s| s.id);
    report.detected_at = chrono::DateTime::from_timestamp_millis(now)
        .unwrap()
        .to_rfc3339();
    db.persist_integrity_report(&s, &report)
}
#[test]
fn default_disabled_and_account_isolation() {
    let (db, _, _) = fixture();
    assert!(!db.monitoring_control(SUBJECT).unwrap().enabled);
    assert!(db
        .monitoring_candidates("other_account")
        .unwrap()
        .is_empty());
    assert!(db
        .notification_history("other_account", 0, 10)
        .unwrap()
        .items
        .is_empty());
    assert!(db.monitoring_control("missing").is_err());
}
#[test]
fn control_generation_invalidates_leases() {
    let (mut d, c, _) = fixture();
    let l = lease(&mut d, &c, NOW);
    d.set_monitoring_enabled(SUBJECT, false, NOW + 1).unwrap();
    assert!(!d.is_monitoring_lease_current(SUBJECT, &l, NOW + 2).unwrap());
    assert!(!d
        .finish_monitoring_check(SUBJECT, &l, CheckOutcome::Success, NOW + 60_000, NOW + 2)
        .unwrap());
}
#[test]
fn schedule_preserves_due_until_revision_changes() {
    let (mut d, c, _) = fixture();
    d.ensure_monitoring_schedule(SUBJECT, &c, NOW + 999)
        .unwrap();
    assert_eq!(d.monitoring_schedules(SUBJECT).unwrap()[0].next_run_ms, NOW);
}
#[test]
fn automatic_requires_enabled_and_manual_requires_current_scope() {
    let (mut d, c, _) = fixture();
    assert!(d
        .claim_monitoring_check(
            SUBJECT,
            &c.spreadsheet_id,
            c.registry_revision,
            0,
            "owner",
            NOW,
            300_000,
            false
        )
        .unwrap()
        .is_none());
    assert!(d
        .claim_monitoring_check(
            SUBJECT,
            &c.spreadsheet_id,
            c.registry_revision + 1,
            0,
            "owner",
            NOW,
            300_000,
            true
        )
        .unwrap()
        .is_none());
    assert!(d
        .claim_monitoring_check(
            "other_account",
            &c.spreadsheet_id,
            c.registry_revision,
            0,
            "owner",
            NOW,
            300_000,
            true
        )
        .unwrap()
        .is_none());
}
#[test]
fn lease_prevents_duplicate_and_stale_owner_completion() {
    let (mut d, c, _) = fixture();
    let old = lease(&mut d, &c, NOW);
    assert!(d
        .claim_monitoring_check(
            SUBJECT,
            &c.spreadsheet_id,
            c.registry_revision,
            0,
            "other",
            NOW + 1,
            300_000,
            true
        )
        .unwrap()
        .is_none());
    d.recover_monitoring_leases(SUBJECT, "new-process", NOW + 2)
        .unwrap();
    let new = lease(&mut d, &c, NOW + 3);
    assert_ne!(old.lease_id, new.lease_id);
    assert!(!d
        .finish_monitoring_check(SUBJECT, &old, CheckOutcome::Success, NOW + 60_000, NOW + 4)
        .unwrap());
    assert!(d
        .is_monitoring_lease_current(SUBJECT, &new, NOW + 4)
        .unwrap());
}
#[test]
fn lease_expiry_prevents_commit() {
    let (mut d, c, _) = fixture();
    let l = lease(&mut d, &c, NOW);
    assert!(!d
        .is_monitoring_lease_current(SUBJECT, &l, NOW + 300_000)
        .unwrap());
}
#[test]
fn registry_revision_change_invalidates_work() {
    let (mut d, c, _) = fixture();
    let l = lease(&mut d, &c, NOW);
    d.set_registered_paused(
        SUBJECT,
        &Mutation {
            spreadsheet_id: c.spreadsheet_id.clone(),
            expected_generation: 0,
            expected_revision: c.registry_revision,
        },
        true,
    )
    .unwrap();
    assert!(!d.is_monitoring_lease_current(SUBJECT, &l, NOW + 1).unwrap());
    assert!(d.monitoring_candidates(SUBJECT).unwrap().is_empty());
}
#[test]
fn operational_failure_deduplicates_and_recovers_without_integrity_report() {
    let (mut d, c, _) = fixture();
    preference(&mut d, true);
    failure(&mut d, &c, "network_failed", NOW);
    failure(&mut d, &c, "network_failed", NOW + 60_000);
    assert_eq!(
        d.monitoring_operational_history(SUBJECT, 0, 10)
            .unwrap()
            .total,
        1
    );
    let l = lease(&mut d, &c, NOW + 120_000);
    d.finish_monitoring_check(
        SUBJECT,
        &l,
        CheckOutcome::Success,
        NOW + 180_000,
        NOW + 120_001,
    )
    .unwrap();
    let events = d.monitoring_operational_history(SUBJECT, 0, 10).unwrap();
    assert_eq!(events.total, 2);
    assert_eq!(events.items[0].kind, "recovered");
    assert_eq!(d.notification_history(SUBJECT, 0, 10).unwrap().total, 2);
    assert_eq!(
        d.connection
            .query_row("SELECT count(*) FROM integrity_reports", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn changed_operational_condition_is_a_distinct_event() {
    let (mut d, c, _) = fixture();
    failure(&mut d, &c, "network_failed", NOW);
    failure(&mut d, &c, "permission_denied", NOW + 60_000);
    assert_eq!(
        d.monitoring_operational_history(SUBJECT, 0, 10)
            .unwrap()
            .total,
        2
    );
}
#[test]
fn cancellation_creates_no_failure() {
    let (mut d, c, _) = fixture();
    let l = lease(&mut d, &c, NOW);
    d.finish_monitoring_check(SUBJECT, &l, CheckOutcome::Cancelled, NOW + 1, NOW + 1)
        .unwrap();
    assert_eq!(
        d.monitoring_operational_history(SUBJECT, 0, 10)
            .unwrap()
            .total,
        0
    );
    assert_eq!(d.monitoring_schedules(SUBJECT).unwrap()[0].failure_count, 0);
}
#[test]
fn integrity_event_outbox_is_atomic_and_deduplicated() {
    let (mut d, _, b) = fixture();
    preference(&mut d, true);
    let events = integrity_event(&mut d, &b, "check1", NOW).unwrap();
    assert!(!events.is_empty());
    assert_eq!(
        d.notification_history(SUBJECT, 0, 10).unwrap().total,
        events.len() as u64
    );
    let repeated = integrity_event(&mut d, &b, "check2", NOW + 60_000).unwrap();
    assert!(repeated.is_empty());
    assert_eq!(
        d.notification_history(SUBJECT, 0, 10).unwrap().total,
        events.len() as u64
    );
    let n = d
        .notification_history(SUBJECT, 0, 10)
        .unwrap()
        .items
        .remove(0);
    assert_eq!(n.source, "integrity");
    assert_eq!(n.state, "pending");
}
#[test]
fn outbox_failure_rolls_back_integrity_event_transaction() {
    let (mut d, _, b) = fixture();
    preference(&mut d, true);
    d.connection.execute_batch("CREATE TRIGGER block_notice BEFORE INSERT ON notification_outbox BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();
    let result = integrity_event(&mut d, &b, "failcheck", NOW);
    assert!(result.is_err());
    assert_eq!(
        d.connection
            .query_row("SELECT count(*) FROM integrity_events", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        d.connection
            .query_row("SELECT count(*) FROM integrity_reports", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn preference_off_is_terminal_and_no_backfill() {
    let (mut d, c, _) = fixture();
    failure(&mut d, &c, "network_failed", NOW);
    let n = d
        .notification_history(SUBJECT, 0, 10)
        .unwrap()
        .items
        .remove(0);
    assert_eq!(n.state, "suppressed_by_preference");
    preference(&mut d, true);
    assert!(d
        .claim_notification(SUBJECT, "native", NOW + 1)
        .unwrap()
        .is_none());
}
#[test]
fn disabling_preference_suppresses_queued_and_active_attempt() {
    let (mut d, c, _) = fixture();
    preference(&mut d, true);
    failure(&mut d, &c, "network_failed", NOW);
    let a = d
        .claim_notification(SUBJECT, "native", NOW + 2)
        .unwrap()
        .unwrap();
    preference(&mut d, false);
    assert_eq!(
        d.suppress_disabled_notifications(SUBJECT, NOW + 3).unwrap(),
        0
    );
    assert!(!d
        .finish_notification_attempt(
            SUBJECT,
            &a.record.id,
            &a.attempt_id,
            NotificationOutcome::DispatchSubmitted,
            NOW + 4
        )
        .unwrap());
    assert_eq!(
        d.notification_by_id(SUBJECT, &a.record.id)
            .unwrap()
            .unwrap()
            .state,
        "suppressed_by_preference"
    );
}
#[test]
fn dispatch_attempt_cas_and_account_guard() {
    let (mut d, c, _) = fixture();
    preference(&mut d, true);
    failure(&mut d, &c, "network_failed", NOW);
    let a = d
        .claim_notification(SUBJECT, "native", NOW + 2)
        .unwrap()
        .unwrap();
    assert!(d
        .notification_by_id("other_account", &a.record.id)
        .unwrap()
        .is_none());
    assert!(!d
        .finish_notification_attempt(
            "other_account",
            &a.record.id,
            &a.attempt_id,
            NotificationOutcome::DispatchSubmitted,
            NOW + 3
        )
        .unwrap());
    assert!(d
        .finish_notification_attempt(
            SUBJECT,
            &a.record.id,
            &a.attempt_id,
            NotificationOutcome::DispatchSubmitted,
            NOW + 3
        )
        .unwrap());
    assert!(!d
        .finish_notification_attempt(
            SUBJECT,
            &a.record.id,
            &a.attempt_id,
            NotificationOutcome::DispatchSubmitted,
            NOW + 4
        )
        .unwrap());
    let n = d
        .notification_by_id(SUBJECT, &a.record.id)
        .unwrap()
        .unwrap();
    assert_eq!(n.submitted_ms, Some(NOW + 3));
    assert_eq!(n.state, "dispatch_submitted");
}
#[test]
fn interrupted_dispatch_is_ambiguous_and_retried_durably() {
    let (mut d, c, _) = fixture();
    preference(&mut d, true);
    failure(&mut d, &c, "network_failed", NOW);
    let first = d
        .claim_notification(SUBJECT, "old-process", NOW + 2)
        .unwrap()
        .unwrap();
    let second = d
        .claim_notification(SUBJECT, "new-process", NOW + 3)
        .unwrap()
        .unwrap();
    assert_ne!(first.attempt_id, second.attempt_id);
    assert_eq!(first.record.id, second.record.id);
    assert_eq!(second.record.attempt_count, 2);
    assert_eq!(
        d.connection
            .query_row(
                "SELECT outcome FROM notification_attempts WHERE id=?1",
                [first.attempt_id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "ambiguous"
    );
}
#[test]
fn permission_denied_does_not_interrupt_monitoring() {
    let (mut d, c, _) = fixture();
    preference(&mut d, true);
    failure(&mut d, &c, "network_failed", NOW);
    let a = d
        .claim_notification(SUBJECT, "native", NOW + 2)
        .unwrap()
        .unwrap();
    d.finish_notification_attempt(
        SUBJECT,
        &a.record.id,
        &a.attempt_id,
        NotificationOutcome::PermissionBlocked,
        NOW + 3,
    )
    .unwrap();
    assert!(d
        .claim_notification(SUBJECT, "native", NOW + 60_000)
        .unwrap()
        .is_none());
    let l = lease(&mut d, &c, NOW + 60_000);
    assert!(d
        .finish_monitoring_check(
            SUBJECT,
            &l,
            CheckOutcome::Success,
            NOW + 120_000,
            NOW + 60_001
        )
        .unwrap());
    assert_eq!(
        d.retry_blocked_notifications(SUBJECT, NOW + 60_002)
            .unwrap(),
        1
    );
}
#[test]
fn pending_ttl_expires_explicitly_and_prunes_without_success() {
    let (mut d, c, _) = fixture();
    preference(&mut d, true);
    failure(&mut d, &c, "network_failed", NOW);
    let n = d
        .notification_history(SUBJECT, 0, 10)
        .unwrap()
        .items
        .remove(0);
    let expired = NOW + 91 * 86400000;
    assert_eq!(d.expire_notifications(SUBJECT, expired).unwrap(), 1);
    let n = d.notification_by_id(SUBJECT, &n.id).unwrap().unwrap();
    assert_eq!(n.state, "expired");
    assert!(n.submitted_ms.is_none());
    assert!(d
        .claim_notification(SUBJECT, "native", expired)
        .unwrap()
        .is_none());
    assert_eq!(d.prune_monitoring_history(SUBJECT, expired).unwrap(), 1);
    assert_eq!(d.notification_history(SUBJECT, 0, 10).unwrap().total, 0);
}
#[test]
fn explicit_erase_cascades_outbox_attempts_and_operational_activity() {
    let (mut d, c, _) = fixture();
    preference(&mut d, true);
    failure(&mut d, &c, "network_failed", NOW);
    d.claim_notification(SUBJECT, "native", NOW + 2)
        .unwrap()
        .unwrap();
    d.remove_registered_spreadsheet(
        SUBJECT,
        &RemoveInput {
            spreadsheet_id: c.spreadsheet_id,
            expected_generation: 0,
            expected_revision: c.registry_revision,
            confirmed: true,
            history: "delete_activity".into(),
        },
    )
    .unwrap();
    assert_eq!(d.notification_history(SUBJECT, 0, 10).unwrap().total, 0);
    assert_eq!(
        d.monitoring_operational_history(SUBJECT, 0, 10)
            .unwrap()
            .total,
        0
    );
    assert_eq!(
        d.connection
            .query_row("SELECT count(*) FROM notification_attempts", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn paging_limits_and_sanitized_error_codes() {
    let (mut d, c, _) = fixture();
    assert!(d.notification_history(SUBJECT, 0, 0).is_err());
    assert!(d.monitoring_operational_history(SUBJECT, 0, 101).is_err());
    let l = lease(&mut d, &c, NOW);
    assert!(d
        .finish_monitoring_check(
            SUBJECT,
            &l,
            CheckOutcome::OperationalFailure {
                code: "token=secret".into()
            },
            NOW + 60_000,
            NOW + 1
        )
        .is_err());
}

#[test]
fn operational_transition_rolls_back_with_outbox_failure() {
    let (mut d, c, _) = fixture();
    let l = lease(&mut d, &c, NOW);
    d.connection.execute_batch("CREATE TRIGGER reject_notice BEFORE INSERT ON notification_outbox BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();
    assert!(d
        .finish_monitoring_check(
            SUBJECT,
            &l,
            CheckOutcome::OperationalFailure {
                code: "network_failed".into()
            },
            NOW + 60_000,
            NOW + 1
        )
        .is_err());
    assert_eq!(
        d.monitoring_operational_history(SUBJECT, 0, 10)
            .unwrap()
            .total,
        0
    );
    let schedule = d.monitoring_schedules(SUBJECT).unwrap().remove(0);
    assert_eq!(schedule.failure_count, 0);
    assert!(schedule.operational_code.is_none());
    assert!(d.is_monitoring_lease_current(SUBJECT, &l, NOW + 2).unwrap());
}

#[test]
fn preference_toggle_off_on_before_dispatch_does_not_resurrect_queue() {
    let (mut d, c, _) = fixture();
    preference(&mut d, true);
    failure(&mut d, &c, "network_failed", NOW);
    preference(&mut d, false);
    preference(&mut d, true);
    assert!(d
        .claim_notification(SUBJECT, "native", NOW + 2)
        .unwrap()
        .is_none());
    assert_eq!(
        d.notification_history(SUBJECT, 0, 10).unwrap().items[0].state,
        "suppressed_by_preference"
    );
}

#[test]
fn manual_claim_respects_persisted_operational_backoff() {
    let (mut d, c, _) = fixture();
    failure(&mut d, &c, "rate_limited", NOW);
    assert!(d
        .claim_monitoring_check(
            SUBJECT,
            &c.spreadsheet_id,
            c.registry_revision,
            0,
            "owner",
            NOW + 2,
            300_000,
            true
        )
        .unwrap()
        .is_none());
    assert!(d
        .claim_monitoring_check(
            SUBJECT,
            &c.spreadsheet_id,
            c.registry_revision,
            0,
            "owner",
            NOW + 60_000,
            300_000,
            true
        )
        .unwrap()
        .is_some());
}

#[test]
fn paused_lease_cannot_commit_report_inside_transaction() {
    let (mut d, c, b) = fixture();
    let l = lease(&mut d, &c, NOW);
    let mut s = fixtures::snapshot();
    s.id = "guarded-report-snapshot".into();
    d.save_structural_snapshot(&s).unwrap();
    let obs = d
        .content_observation(SUBJECT, &s.spreadsheet_id, &s.scope_hash)
        .unwrap();
    let mut report = evaluate(&b, obs.snapshot.as_ref(), &s, "guarded-check").unwrap();
    report.observation_revision = obs.revision;
    report.previous_snapshot_id = d
        .previous_structural_snapshot(SUBJECT, &s.spreadsheet_id, &s.scope_hash)
        .unwrap()
        .map(|s| s.id);
    d.set_monitoring_enabled(SUBJECT, false, NOW + 1).unwrap();
    assert_eq!(
        d.persist_monitored_integrity_report(&s, &report, Some(c.registry_revision), &l, NOW + 2),
        Err(crate::integrity::EngineError::BaselineConflict)
    );
    assert_eq!(
        d.connection
            .query_row(
                "SELECT count(*) FROM integrity_reports WHERE check_id='guarded-check'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
}
