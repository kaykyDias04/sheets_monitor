use super::*;
use crate::google::SheetMetadata;
use crate::integrity::{evaluate, fixtures, IntegrityStatus, StructuralSnapshot};
fn metadata(id: &str) -> SpreadsheetMetadata {
    SpreadsheetMetadata {
        spreadsheet_id: id.into(),
        spreadsheet_url: None,
        title: format!("Fixture {id}"),
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
    }
}
fn db() -> Database {
    let mut d = Database::memory().unwrap();
    d.upsert_account(
        "fixture_account",
        Some("test@example.invalid"),
        Some("Fixture"),
    )
    .unwrap();
    d.upsert_account("other_account", None, None).unwrap();
    d
}
fn mutation(row: &RegisteredSpreadsheet) -> Mutation {
    Mutation {
        spreadsheet_id: row.spreadsheet_id.clone(),
        expected_generation: 3,
        expected_revision: row.revision,
    }
}
fn configured(d: &mut Database) -> RegisteredSpreadsheet {
    let row = d
        .register_operational_spreadsheet("fixture_account", &metadata("fixture_sheet"))
        .unwrap();
    let row = d
        .set_registered_tabs("fixture_account", &mutation(&row), &[7])
        .unwrap();
    d.set_registered_rules("fixture_account", &mutation(&row), &fixtures::scope())
        .unwrap()
}
fn approved(d: &mut Database) -> (RegisteredSpreadsheet, BaselineVersion) {
    let row = configured(d);
    let snapshot = fixtures::snapshot();
    d.save_structural_snapshot(&snapshot).unwrap();
    let baseline = d
        .approve_registered_structural_baseline(
            "fixture_account",
            &snapshot.id,
            0,
            true,
            "synthetic:test",
            Some(row.revision),
        )
        .unwrap();
    (
        d.registered_spreadsheet("fixture_account", "fixture_sheet")
            .unwrap(),
        baseline,
    )
}
fn save_report(
    d: &mut Database,
    baseline: &BaselineVersion,
    snapshot: &StructuralSnapshot,
    check: &str,
) {
    d.save_structural_snapshot(snapshot).unwrap();
    let obs = d
        .content_observation("fixture_account", "fixture_sheet", &snapshot.scope_hash)
        .unwrap();
    let mut report = evaluate(baseline, obs.snapshot.as_ref(), snapshot, check).unwrap();
    report.previous_snapshot_id = d
        .previous_structural_snapshot("fixture_account", "fixture_sheet", &snapshot.scope_hash)
        .unwrap()
        .map(|s| s.id);
    report.observation_revision = obs.revision;
    report.detected_at = chrono::Utc::now().to_rfc3339();
    d.persist_integrity_report(snapshot, &report).unwrap();
}
#[test]
fn registry_uses_current_account_never_demo_or_default_online() {
    let mut d = db();
    d.seed_demo().unwrap();
    let row = d
        .register_operational_spreadsheet("fixture_account", &metadata("fixture_sheet"))
        .unwrap();
    assert_eq!(row.status, "configuration_pending");
    assert!(row.last_checked_at.is_none());
    assert!(d
        .registered_spreadsheets("other_account")
        .unwrap()
        .is_empty());
    assert_eq!(
        d.operational_dashboard("fixture_account")
            .unwrap()
            .registered,
        1
    );
    assert_eq!(
        d.operational_dashboard("fixture_account").unwrap().online,
        0
    );
    assert!(d
        .register_operational_spreadsheet("fixture_account", &metadata("fixture_sheet"))
        .is_err());
}
#[test]
fn registry_capacity_thirty_includes_paused_and_frees_removed_slots() {
    let mut d = db();
    for i in 0..30 {
        d.register_operational_spreadsheet("fixture_account", &metadata(&format!("file_{i}")))
            .unwrap();
    }
    assert_eq!(
        d.register_operational_spreadsheet("other_account", &metadata("file_extra"))
            .unwrap_err()
            .code,
        "capacity"
    );
    let row = d
        .registered_spreadsheet("fixture_account", "file_0")
        .unwrap();
    let row = d
        .set_registered_paused("fixture_account", &mutation(&row), true)
        .unwrap();
    assert!(d
        .register_operational_spreadsheet("fixture_account", &metadata("file_extra"))
        .is_err());
    d.remove_registered_spreadsheet(
        "fixture_account",
        &RemoveInput {
            spreadsheet_id: row.spreadsheet_id,
            expected_generation: 3,
            expected_revision: row.revision,
            confirmed: true,
            history: "preserve".into(),
        },
    )
    .unwrap();
    d.register_operational_spreadsheet("other_account", &metadata("file_extra"))
        .unwrap();
    assert!(d
        .register_operational_spreadsheet("fixture_account", &metadata("file_0"))
        .is_err());
}
#[test]
fn selected_tabs_persist_before_rules_and_revision_rejects_lost_updates() {
    let mut d = db();
    let row = d
        .register_operational_spreadsheet("fixture_account", &metadata("fixture_sheet"))
        .unwrap();
    assert!(d
        .set_registered_tabs("fixture_account", &mutation(&row), &[99])
        .is_err());
    let saved = d
        .set_registered_tabs("fixture_account", &mutation(&row), &[7])
        .unwrap();
    assert_eq!(saved.selected_sheet_ids, vec![7]);
    assert_eq!(saved.status, "configuration_pending");
    assert!(d
        .set_registered_tabs("fixture_account", &mutation(&row), &[])
        .is_err());
    assert!(d
        .set_registered_rules("other_account", &mutation(&saved), &fixtures::scope())
        .is_err());
    assert!(d
        .set_registered_paused("fixture_account", &mutation(&saved), false)
        .is_err());
}
#[test]
fn configuration_reopen_preferences_and_stable_sheet_ids() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fixture.db");
    {
        let mut d = Database::open(&path).unwrap();
        d.upsert_account("fixture_account", None, None).unwrap();
        let row = configured(&mut d);
        let mut meta = metadata("fixture_sheet");
        meta.sheets[0].title = "Renamed".into();
        d.refresh_registered_metadata("fixture_account", &meta)
            .unwrap();
        let p = Preferences {
            interval_seconds: 120,
            notifications_enabled: true,
            theme: "dark".into(),
            ..Default::default()
        };
        d.save_management_preferences("fixture_account", &p)
            .unwrap();
        assert!(row.scope.is_some());
    }
    {
        let d = Database::open(&path).unwrap();
        let row = d
            .registered_spreadsheet("fixture_account", "fixture_sheet")
            .unwrap();
        assert_eq!(row.selected_sheet_ids, vec![7]);
        assert_eq!(row.tabs[0].title, "Renamed");
        assert_eq!(row.scope.unwrap(), fixtures::scope());
        assert_eq!(
            d.management_preferences("fixture_account")
                .unwrap()
                .interval_seconds,
            120
        );
    }
}
#[test]
fn baseline_approval_and_current_scope_status_are_independent() {
    let mut d = db();
    let (row, baseline) = approved(&mut d);
    assert_eq!(row.status, "not_checked");
    assert_eq!(row.baseline_version, Some(1));
    let mut snapshot = fixtures::snapshot();
    snapshot.id = "observation2".into();
    save_report(&mut d, &baseline, &snapshot, "check_one");
    assert_eq!(
        d.registered_spreadsheet("fixture_account", "fixture_sheet")
            .unwrap()
            .status,
        "online"
    );
    let mut scope = fixtures::scope();
    scope.sheets[0].protect_position = true;
    let row = d
        .set_registered_rules("fixture_account", &mutation(&row), &scope)
        .unwrap();
    assert_eq!(row.status, "configuration_pending");
    assert!(d
        .approve_structural_baseline("fixture_account", &snapshot.id, 1, true, "synthetic:test")
        .is_err());
    let obs = d
        .content_observation("fixture_account", "fixture_sheet", &snapshot.scope_hash)
        .unwrap();
    let mut report = evaluate(
        &baseline,
        obs.snapshot.as_ref(),
        &snapshot,
        "late_old_check",
    )
    .unwrap();
    report.previous_snapshot_id = Some(snapshot.id.clone());
    report.observation_revision = obs.revision;
    assert!(d.persist_integrity_report(&snapshot, &report).is_err());
}
#[test]
fn approval_guard_revision_tabs_and_explicit_consent() {
    let mut d = db();
    let row = configured(&mut d);
    let snap = fixtures::snapshot();
    d.save_structural_snapshot(&snap).unwrap();
    assert!(d
        .approve_registered_structural_baseline(
            "fixture_account",
            &snap.id,
            0,
            false,
            "synthetic:test",
            Some(row.revision)
        )
        .is_err());
    let next = d
        .set_registered_tabs("fixture_account", &mutation(&row), &[7])
        .unwrap();
    assert!(d
        .approve_registered_structural_baseline(
            "fixture_account",
            &snap.id,
            0,
            true,
            "synthetic:test",
            Some(row.revision)
        )
        .is_err());
    d.set_registered_tabs("fixture_account", &mutation(&next), &[])
        .unwrap();
    assert!(d
        .approve_structural_baseline("fixture_account", &snap.id, 0, true, "synthetic:test")
        .is_err());
    assert!(d
        .active_structural_baseline("fixture_account", "fixture_sheet", &snap.scope_hash)
        .unwrap()
        .is_none());
}
#[test]
fn new_baseline_does_not_reuse_previous_online_check() {
    let mut d = db();
    let (row, baseline) = approved(&mut d);
    let mut snap = fixtures::snapshot();
    snap.id = "check_snap".into();
    save_report(&mut d, &baseline, &snap, "check_before");
    assert_eq!(
        d.registered_spreadsheet("fixture_account", "fixture_sheet")
            .unwrap()
            .status,
        "online"
    );
    d.approve_registered_structural_baseline(
        "fixture_account",
        &snap.id,
        1,
        true,
        "synthetic:test",
        Some(row.revision),
    )
    .unwrap();
    assert_eq!(
        d.registered_spreadsheet("fixture_account", "fixture_sheet")
            .unwrap()
            .status,
        "not_checked"
    );
}
#[test]
fn pause_and_remove_guard_reports_and_preserve_baselines() {
    let mut d = db();
    let (row, baseline) = approved(&mut d);
    let row = d
        .set_registered_paused("fixture_account", &mutation(&row), true)
        .unwrap();
    assert_eq!(row.status, "paused");
    let mut snap = fixtures::snapshot();
    snap.id = "paused_snapshot".into();
    d.save_structural_snapshot(&snap).unwrap();
    let mut report = evaluate(&baseline, Some(&baseline.snapshot), &snap, "paused_check").unwrap();
    report.previous_snapshot_id = Some(baseline.snapshot.id.clone());
    report.observation_revision = d
        .content_observation("fixture_account", "fixture_sheet", &snap.scope_hash)
        .unwrap()
        .revision;
    assert!(d.persist_integrity_report(&snap, &report).is_err());
    let removal = RemoveInput {
        spreadsheet_id: row.spreadsheet_id.clone(),
        expected_generation: 3,
        expected_revision: row.revision,
        confirmed: false,
        history: "preserve".into(),
    };
    assert!(d
        .remove_registered_spreadsheet("fixture_account", &removal)
        .is_err());
    d.remove_registered_spreadsheet(
        "fixture_account",
        &RemoveInput {
            confirmed: true,
            ..removal
        },
    )
    .unwrap();
    assert!(d
        .registered_spreadsheets("fixture_account")
        .unwrap()
        .is_empty());
    assert!(d
        .active_structural_baseline("fixture_account", "fixture_sheet", &snap.scope_hash)
        .unwrap()
        .is_some());
    assert!(d
        .approve_structural_baseline("fixture_account", &snap.id, 1, true, "synthetic:test")
        .is_err());
}
#[test]
fn history_filters_paginate_in_sql_and_archived_file_remains_searchable() {
    let mut d = db();
    let (row, baseline) = approved(&mut d);
    let mut snap = fixtures::snapshot();
    snap.id = "changed_snapshot".into();
    snap.sheets[0].regions[0].rows[0] = fixtures::row(1, "record_a", 123);
    save_report(&mut d, &baseline, &snap, "change_check");
    let page = d
        .operational_events(
            "fixture_account",
            &EventFilter {
                limit: 1,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(page.total > 0);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].spreadsheet_name, "Fixture fixture_sheet");
    assert_eq!(page.items[0].sheet_title.as_deref(), Some("Data"));
    assert_eq!(page.items[0].status, "online");
    assert_eq!(
        d.operational_events(
            "other_account",
            &EventFilter {
                limit: 10,
                ..Default::default()
            }
        )
        .unwrap()
        .total,
        0
    );
    let last = d
        .operational_events(
            "fixture_account",
            &EventFilter {
                limit: 1,
                offset: page.total as u32,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(last.items.is_empty());
    assert_eq!(
        d.operational_events(
            "fixture_account",
            &EventFilter {
                limit: 10,
                severity: Some("critical".into()),
                ..Default::default()
            }
        )
        .unwrap()
        .total,
        0
    );
    d.remove_registered_spreadsheet(
        "fixture_account",
        &RemoveInput {
            spreadsheet_id: row.spreadsheet_id,
            expected_generation: 3,
            expected_revision: row.revision,
            confirmed: true,
            history: "preserve".into(),
        },
    )
    .unwrap();
    let page = d
        .operational_events(
            "fixture_account",
            &EventFilter {
                limit: 10,
                query: "fixture".into(),
                sheet_id: Some(7),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(page.items.iter().all(|a| a.removed));
    assert!(page.total > 0);
    assert!(d
        .operational_events(
            "fixture_account",
            &EventFilter {
                limit: 101,
                ..Default::default()
            }
        )
        .is_err());
}
#[test]
fn activity_erasure_retains_approved_baseline_and_clears_report_pointers() {
    let mut d = db();
    let (row, baseline) = approved(&mut d);
    let mut snap = fixtures::snapshot();
    snap.id = "changed_to_erase".into();
    snap.sheets[0].regions[0].rows[0] = fixtures::row(1, "record_a", 123);
    save_report(&mut d, &baseline, &snap, "erase_check");
    assert!(
        d.operational_events(
            "fixture_account",
            &EventFilter {
                limit: 10,
                ..Default::default()
            }
        )
        .unwrap()
        .total
            > 0
    );
    d.remove_registered_spreadsheet(
        "fixture_account",
        &RemoveInput {
            spreadsheet_id: row.spreadsheet_id,
            expected_generation: 3,
            expected_revision: row.revision,
            confirmed: true,
            history: "delete_activity".into(),
        },
    )
    .unwrap();
    assert_eq!(
        d.operational_events(
            "fixture_account",
            &EventFilter {
                limit: 10,
                ..Default::default()
            }
        )
        .unwrap()
        .total,
        0
    );
    assert!(d
        .active_structural_baseline("fixture_account", "fixture_sheet", &snap.scope_hash)
        .unwrap()
        .is_some());
    assert!(d
        .latest_integrity_report("fixture_account", "fixture_sheet", &snap.scope_hash)
        .unwrap()
        .is_none());
    d.register_operational_spreadsheet("fixture_account", &metadata("fixture_sheet"))
        .unwrap();
    assert_eq!(
        d.registered_spreadsheet("fixture_account", "fixture_sheet")
            .unwrap()
            .status,
        "not_checked"
    );
}
#[test]
fn scripts_require_authorized_association_and_preferences_are_validated() {
    let mut d = db();
    let row = configured(&mut d);
    let mut scope = fixtures::scope();
    scope.scripts.push(crate::integrity::ScriptRule {
        script_id: "unauthorized_script".into(),
        critical: true,
    });
    assert!(d
        .set_registered_rules("fixture_account", &mutation(&row), &scope)
        .is_err());
    assert!(d
        .save_management_preferences(
            "fixture_account",
            &Preferences {
                start_with_windows: true,
                ..Default::default()
            }
        )
        .is_err());
    assert!(d
        .save_management_preferences(
            "fixture_account",
            &Preferences {
                interval_seconds: 30,
                ..Default::default()
            }
        )
        .is_err());
    assert_eq!(
        d.management_preferences("other_account").unwrap(),
        Preferences::default()
    );
}
#[test]
fn ipc_flat_json_inputs_accept_valid_and_reject_unknown_keys() {
    let base = serde_json::json!({"spreadsheet_id":"fixture_sheet","expected_generation":3,"expected_revision":4});
    let mut tabs = base.clone();
    tabs["sheet_ids"] = serde_json::json!([7]);
    assert!(serde_json::from_value::<TabsInput>(tabs.clone()).is_ok());
    tabs["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<TabsInput>(tabs).is_err());
    let mut rules = base.clone();
    rules["scope"] = serde_json::to_value(fixtures::scope()).unwrap();
    assert!(serde_json::from_value::<RulesInput>(rules.clone()).is_ok());
    rules["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<RulesInput>(rules).is_err());
    let mut approval = base.clone();
    for (key, val) in [
        ("snapshot_id", serde_json::json!("snapshot_fixture_1")),
        ("snapshot_fingerprint", serde_json::json!("a".repeat(64))),
        ("expected_version", serde_json::json!(0)),
        ("confirmed", serde_json::json!(true)),
    ] {
        approval[key] = val;
    }
    assert!(serde_json::from_value::<ApprovalInput>(approval.clone()).is_ok());
    approval["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ApprovalInput>(approval).is_err());
    let mut removal = base;
    removal["confirmed"] = serde_json::json!(true);
    removal["history"] = serde_json::json!("delete_activity");
    assert!(serde_json::from_value::<RemoveInput>(removal.clone()).is_ok());
    removal["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<RemoveInput>(removal).is_err());
}
#[test]
fn diagnostics_do_not_invent_integrity_for_unapproved_scope() {
    let mut d = db();
    let row = configured(&mut d);
    let diag = d
        .operational_diagnostics("fixture_account", &row.spreadsheet_id)
        .unwrap();
    assert!(diag.report.is_none());
    assert!(diag.baseline.is_none());
    assert!(diag.scripts.is_empty());
    assert_eq!(diag.spreadsheet.status, "configuration_pending");
    let _status = IntegrityStatus::Online;
}
#[test]
fn capture_capacity_reuses_slot_after_retained_registry_removal() {
    let mut d = db();
    for i in 0..30 {
        let id = format!("registered_file_{i}");
        let mut snap = fixtures::snapshot();
        snap.id = format!("capacity_snapshot_{i}");
        snap.spreadsheet_id = id.clone();
        d.register_operational_spreadsheet("fixture_account", &metadata(&id))
            .unwrap();
        d.save_structural_snapshot(&snap).unwrap();
    }
    let row = d
        .registered_spreadsheet("fixture_account", "registered_file_0")
        .unwrap();
    d.remove_registered_spreadsheet(
        "fixture_account",
        &RemoveInput {
            spreadsheet_id: row.spreadsheet_id,
            expected_generation: 3,
            expected_revision: row.revision,
            confirmed: true,
            history: "preserve".into(),
        },
    )
    .unwrap();
    d.register_operational_spreadsheet("fixture_account", &metadata("new_registered_file"))
        .unwrap();
    let mut snap = fixtures::snapshot();
    snap.id = "new_capacity_snapshot".into();
    snap.spreadsheet_id = "new_registered_file".into();
    d.save_structural_snapshot(&snap).unwrap();
    assert_eq!(
        d.registered_spreadsheets("fixture_account").unwrap().len(),
        30
    );
    assert!(d
        .structural_snapshot("fixture_account", "capacity_snapshot_0")
        .is_ok());
}
#[test]
fn report_transaction_rejects_stale_revision_after_pause_resume_same_scope() {
    let mut d = db();
    let (row, baseline) = approved(&mut d);
    let mut snap = fixtures::snapshot();
    snap.id = "captured_before_pause".into();
    d.save_structural_snapshot(&snap).unwrap();
    let obs = d
        .content_observation("fixture_account", "fixture_sheet", &snap.scope_hash)
        .unwrap();
    let mut report = evaluate(
        &baseline,
        obs.snapshot.as_ref(),
        &snap,
        "stale_registry_report",
    )
    .unwrap();
    report.previous_snapshot_id = Some(baseline.snapshot.id.clone());
    report.observation_revision = obs.revision;
    let paused = d
        .set_registered_paused("fixture_account", &mutation(&row), true)
        .unwrap();
    let resumed = d
        .set_registered_paused("fixture_account", &mutation(&paused), false)
        .unwrap();
    assert!(d
        .persist_registered_integrity_report(&snap, &report, Some(row.revision))
        .is_err());
    assert!(d
        .latest_integrity_report("fixture_account", "fixture_sheet", &snap.scope_hash)
        .unwrap()
        .is_none());
    d.persist_registered_integrity_report(&snap, &report, Some(resumed.revision))
        .unwrap();
}
#[test]
fn baseline_version_queries_are_paged_and_account_isolated() {
    let mut d = db();
    let (row, baseline) = approved(&mut d);
    let page = d
        .operational_baselines(
            "fixture_account",
            &BaselinesQuery {
                spreadsheet_id: row.spreadsheet_id.clone(),
                offset: 0,
                limit: 1,
            },
        )
        .unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].id, baseline.id);
    assert!(d
        .operational_baseline_by_id("other_account", "fixture_sheet", &baseline.id)
        .is_err());
    assert!(d
        .operational_baselines(
            "fixture_account",
            &BaselinesQuery {
                spreadsheet_id: row.spreadsheet_id,
                offset: 1,
                limit: 1
            }
        )
        .unwrap()
        .items
        .is_empty());
}
#[test]
fn only_current_configuration_baseline_is_active_across_scope_versions() {
    let mut d = db();
    let (row, before) = approved(&mut d);
    let mut scope = fixtures::scope();
    scope.sheets[0].protect_position = true;
    let row = d
        .set_registered_rules("fixture_account", &mutation(&row), &scope)
        .unwrap();
    let mut snap = fixtures::snapshot();
    snap.id = "second_scope_snapshot".into();
    snap.scope = scope;
    snap.scope_hash = snap.scope.hash();
    d.save_structural_snapshot(&snap).unwrap();
    let after = d
        .approve_registered_structural_baseline(
            "fixture_account",
            &snap.id,
            0,
            true,
            "synthetic:test",
            Some(row.revision),
        )
        .unwrap();
    let page = d
        .operational_baselines(
            "fixture_account",
            &BaselinesQuery {
                spreadsheet_id: "fixture_sheet".into(),
                offset: 0,
                limit: 20,
            },
        )
        .unwrap();
    assert_eq!(page.items.iter().filter(|b| b.active).count(), 1);
    assert!(page.items.iter().find(|b| b.id == after.id).unwrap().active);
    assert!(
        !page
            .items
            .iter()
            .find(|b| b.id == before.id)
            .unwrap()
            .active
    );
    assert!(
        !d.operational_baseline_by_id("fixture_account", "fixture_sheet", &before.id)
            .unwrap()
            .active
    );
    let row = d
        .set_registered_tabs("fixture_account", &mutation(&row), &[])
        .unwrap();
    assert_eq!(row.status, "configuration_pending");
    assert!(d
        .operational_baselines(
            "fixture_account",
            &BaselinesQuery {
                spreadsheet_id: "fixture_sheet".into(),
                offset: 0,
                limit: 20
            }
        )
        .unwrap()
        .items
        .iter()
        .all(|b| !b.active));
}
#[test]
fn legacy_baselines_are_listed_inactive_before_operational_rules_are_configured() {
    let mut d = db();
    let snap = fixtures::snapshot();
    d.save_structural_snapshot(&snap).unwrap();
    let baseline = d
        .approve_structural_baseline("fixture_account", &snap.id, 0, true, "synthetic:test")
        .unwrap();
    d.register_operational_spreadsheet("fixture_account", &metadata("fixture_sheet"))
        .unwrap();
    let page = d
        .operational_baselines(
            "fixture_account",
            &BaselinesQuery {
                spreadsheet_id: "fixture_sheet".into(),
                offset: 0,
                limit: 10,
            },
        )
        .unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].id, baseline.id);
    assert!(!page.items[0].active);
    assert!(
        !d.operational_baseline_by_id("fixture_account", "fixture_sheet", &baseline.id)
            .unwrap()
            .active
    );
}
