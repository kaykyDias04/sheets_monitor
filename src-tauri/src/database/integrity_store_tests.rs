use super::*;
use crate::integrity::{
    ColumnRule, ContentDifference, Coverage, CoverageIssue, GridRect, MonitorScope, ObservedColumn,
    RegionRule, RegionSnapshot, RowDigest, SheetAssessment, SheetRule, SheetSnapshot,
    SNAPSHOT_VERSION,
};
use std::collections::BTreeMap;

fn snapshot(id: &str) -> StructuralSnapshot {
    let scope = MonitorScope {
        schema_version: SNAPSHOT_VERSION,
        sheets: vec![SheetRule {
            sheet_id: 0,
            protect_title: true,
            protect_position: false,
            regions: vec![RegionRule {
                id: "data".into(),
                range: GridRect {
                    start_row: 0,
                    end_row: 5,
                    start_col: 0,
                    end_col: 2,
                },
                header_row: 0,
                columns: vec![
                    ColumnRule {
                        id: "key".into(),
                        header: "ID".into(),
                        position: 0,
                        protect_position: true,
                        required: true,
                    },
                    ColumnRule {
                        id: "amount".into(),
                        header: "Amount".into(),
                        position: 1,
                        protect_position: true,
                        required: true,
                    },
                ],
                key_column: Some("key".into()),
                critical: true,
                allow_record_deletion: true,
                formulas: Vec::new(),
            }],
        }],
        scripts: Vec::new(),
        named_range_ids: Vec::new(),
        snapshot_retention_days: 7,
        history_retention_days: 90,
    };
    let cells: BTreeMap<String, String> = [
        ("key".into(), fingerprint(&"fixture-key")),
        ("amount".into(), fingerprint(&12)),
    ]
    .into();
    StructuralSnapshot {
        schema_version: SNAPSHOT_VERSION,
        id: id.into(),
        account_subject: "fixture_subject".into(),
        spreadsheet_id: "fixture_sheet".into(),
        capture_started_at: "2000-01-01T00:00:00Z".into(),
        captured_at: "2000-01-01T00:00:01Z".into(),
        scope_hash: scope.hash(),
        sheets: vec![SheetSnapshot {
            sheet_id: 0,
            title: "Data".into(),
            index: 0,
            row_count: 100,
            column_count: 10,
            regions: vec![RegionSnapshot {
                region_id: "data".into(),
                range: scope.sheets[0].regions[0].range.clone(),
                complete: true,
                headers: vec![
                    ObservedColumn {
                        column: 0,
                        header: "ID".into(),
                    },
                    ObservedColumn {
                        column: 1,
                        header: "Amount".into(),
                    },
                ],
                rows: vec![RowDigest {
                    row: 1,
                    key_hash: Some(fingerprint(&"fixture-key")),
                    fingerprint: fingerprint(&cells),
                    cell_hashes: cells,
                }],
                formulas: Vec::new(),
            }],
        }],
        scope,
        named_ranges: Vec::new(),
        scripts: Vec::new(),
        coverage: Coverage {
            metadata_complete: true,
            complete: true,
            issues: Vec::new(),
            atomic_across_services: false,
        },
        connectivity: Connectivity::Reachable,
    }
}
fn database() -> Database {
    let mut db = Database::memory().unwrap();
    db.upsert_account(
        "fixture_subject",
        Some("fixture@example.invalid"),
        Some("Synthetic account"),
    )
    .unwrap();
    db.upsert_account("other_subject", None, None).unwrap();
    db
}
fn approve(db: &mut Database, snapshot: &StructuralSnapshot, version: u32) -> BaselineVersion {
    db.save_structural_snapshot(snapshot).unwrap();
    db.approve_structural_baseline(
        &snapshot.account_subject,
        &snapshot.id,
        version,
        true,
        "synthetic:test",
    )
    .unwrap()
}
fn violation() -> Finding {
    Finding {
        key: "header:0:data:amount".into(),
        sheet_id: Some(0),
        rule_id: "amount".into(),
        code: "header_changed".into(),
        severity: Severity::Critical,
        outcome: FindingOutcome::Violation,
        expected: Some("Amount".into()),
        found: Some("Changed".into()),
        location: Some("B1".into()),
    }
}
fn report(
    db: &Database,
    snapshot: &StructuralSnapshot,
    baseline: &BaselineVersion,
    id: &str,
    findings: Vec<Finding>,
    differences: Vec<ContentDifference>,
) -> IntegrityReport {
    let status = if findings
        .iter()
        .any(|f| f.severity == Severity::Critical && f.outcome == FindingOutcome::Violation)
    {
        IntegrityStatus::Offline
    } else if findings
        .iter()
        .any(|f| f.severity == Severity::Critical && f.outcome == FindingOutcome::Inconclusive)
    {
        IntegrityStatus::Indeterminate
    } else {
        IntegrityStatus::Online
    };
    IntegrityReport {
        observation_revision: db
            .content_observation(
                &snapshot.account_subject,
                &snapshot.spreadsheet_id,
                &snapshot.scope_hash,
            )
            .unwrap()
            .revision,
        previous_snapshot_id: db
            .previous_structural_snapshot(
                &snapshot.account_subject,
                &snapshot.spreadsheet_id,
                &snapshot.scope_hash,
            )
            .unwrap()
            .map(|s| s.id),
        check_id: id.into(),
        snapshot_id: snapshot.id.clone(),
        baseline_id: baseline.id.clone(),
        account_subject: snapshot.account_subject.clone(),
        spreadsheet_id: snapshot.spreadsheet_id.clone(),
        detected_at: "2000-01-02T00:00:00Z".into(),
        status,
        connectivity: snapshot.connectivity,
        sheets: vec![SheetAssessment {
            sheet_id: 0,
            status,
        }],
        findings,
        differences,
        coverage: snapshot.coverage.clone(),
    }
}
fn persist(
    db: &mut Database,
    snapshot: &StructuralSnapshot,
    baseline: &BaselineVersion,
    id: &str,
    findings: Vec<Finding>,
) -> Vec<IntegrityEvent> {
    db.save_structural_snapshot(snapshot).unwrap();
    let report = report(db, snapshot, baseline, id, findings, Vec::new());
    db.persist_integrity_report(snapshot, &report).unwrap()
}

#[test]
fn approval_requires_confirmation_valid_origin_and_complete_snapshot() {
    let mut db = database();
    let mut snap = snapshot("approval");
    db.save_structural_snapshot(&snap).unwrap();
    assert_eq!(
        db.approve_structural_baseline("fixture_subject", &snap.id, 0, false, "human:native"),
        Err(EngineError::ApprovalRequired)
    );
    assert_eq!(
        db.approve_structural_baseline("fixture_subject", &snap.id, 0, true, "automatic"),
        Err(EngineError::ApprovalRequired)
    );
    assert_eq!(
        db.approve_structural_baseline("other_subject", &snap.id, 0, true, "human:native"),
        Err(EngineError::NotFound)
    );
    snap.id = "incomplete".into();
    snap.coverage.complete = false;
    db.save_structural_snapshot(&snap).unwrap();
    assert!(db
        .approve_structural_baseline("fixture_subject", &snap.id, 0, true, "human:native")
        .is_err());
    assert!(db
        .structural_baselines("fixture_subject", "fixture_sheet", &snap.scope_hash)
        .unwrap()
        .is_empty());
}

#[test]
fn immutable_versions_and_compare_and_swap_preserve_history() {
    let mut db = database();
    let first = snapshot("first");
    let v1 = approve(&mut db, &first, 0);
    let mut second = first.clone();
    second.id = "second".into();
    db.save_structural_snapshot(&second).unwrap();
    assert_eq!(
        db.approve_structural_baseline("fixture_subject", &second.id, 0, true, "synthetic:test"),
        Err(EngineError::BaselineConflict)
    );
    let v2 = db
        .approve_structural_baseline("fixture_subject", &second.id, 1, true, "synthetic:test")
        .unwrap();
    assert_eq!(v2.version, 2);
    let versions = db
        .structural_baselines("fixture_subject", "fixture_sheet", &first.scope_hash)
        .unwrap();
    assert_eq!(versions.len(), 2);
    assert!(versions[0].active);
    assert!(!versions[1].active);
    assert_eq!(versions[1].id, v1.id);
    assert_eq!(versions[1].snapshot, first);
    assert!(db
        .connection
        .execute(
            "UPDATE baseline_versions SET approval_origin='human:native' WHERE id=?1",
            [&v1.id]
        )
        .is_err());
    assert!(db
        .connection
        .execute("DELETE FROM baseline_versions WHERE id=?1", [&v1.id])
        .is_err());
    assert!(db
        .connection
        .execute("DELETE FROM structural_snapshots WHERE id=?1", [&first.id])
        .is_err());
    assert_eq!(
        db.previous_structural_snapshot("fixture_subject", "fixture_sheet", &first.scope_hash)
            .unwrap()
            .unwrap()
            .id,
        first.id
    );
}

#[test]
fn snapshot_identity_is_immutable_and_account_scope_isolated() {
    let mut db = database();
    let snap = snapshot("identity");
    approve(&mut db, &snap, 0);
    assert_eq!(
        db.structural_snapshot("other_subject", &snap.id),
        Err(EngineError::NotFound)
    );
    assert!(db
        .structural_baselines("other_subject", "fixture_sheet", &snap.scope_hash)
        .unwrap()
        .is_empty());
    assert!(db
        .active_structural_baseline("other_subject", "fixture_sheet", &snap.scope_hash)
        .unwrap()
        .is_none());
    let mut changed = snap.clone();
    changed.sheets[0].title = "Other".into();
    assert_eq!(
        db.save_structural_snapshot(&changed),
        Err(EngineError::InvalidSnapshot)
    );
    assert!(db
        .connection
        .execute(
            "UPDATE structural_snapshots SET snapshot_json='{}' WHERE id=?1",
            [&snap.id]
        )
        .is_err());
    assert_eq!(
        db.structural_snapshot("fixture_subject", &snap.id).unwrap(),
        snap
    );
}

#[test]
fn changed_scope_needs_its_own_approved_baseline() {
    let mut db = database();
    let first = snapshot("scope-a");
    let a = approve(&mut db, &first, 0);
    let mut second = first.clone();
    second.id = "scope-b".into();
    second.scope.sheets[0].protect_position = true;
    second.scope_hash = second.scope.hash();
    db.save_structural_snapshot(&second).unwrap();
    assert!(db
        .active_structural_baseline("fixture_subject", "fixture_sheet", &second.scope_hash)
        .unwrap()
        .is_none());
    let b = db
        .approve_structural_baseline("fixture_subject", &second.id, 0, true, "synthetic:test")
        .unwrap();
    assert_eq!(b.version, 1);
    assert_ne!(b.id, a.id);
    assert!(
        db.active_structural_baseline("fixture_subject", "fixture_sheet", &first.scope_hash)
            .unwrap()
            .unwrap()
            .active
    );
}

#[test]
fn repeated_violation_recovery_and_recurrence_survive_history_retention() {
    let mut db = database();
    let initial = snapshot("initial");
    let baseline = approve(&mut db, &initial, 0);
    let mut broken = initial.clone();
    broken.id = "broken".into();
    broken.sheets[0].regions[0].headers[1].header = "Changed".into();
    let first = persist(&mut db, &broken, &baseline, "check-a", vec![violation()]);
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].kind, "violation");
    db.prune_integrity_history(
        "fixture_subject",
        "fixture_sheet",
        &initial.scope_hash,
        1,
        1,
    )
    .unwrap();
    broken.id = "broken-again".into();
    assert!(persist(&mut db, &broken, &baseline, "check-b", vec![violation()]).is_empty());
    let mut restored = initial.clone();
    restored.id = "restored".into();
    let recovered = persist(&mut db, &restored, &baseline, "check-c", Vec::new());
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].kind, "recovered");
    db.prune_integrity_history(
        "fixture_subject",
        "fixture_sheet",
        &initial.scope_hash,
        1,
        1,
    )
    .unwrap();
    broken.id = "broken-recurred".into();
    let recurrence = persist(&mut db, &broken, &baseline, "check-d", vec![violation()]);
    assert_eq!(recurrence.len(), 1);
    assert_ne!(recurrence[0].fingerprint, first[0].fingerprint);
    assert!(db
        .structural_snapshot("fixture_subject", &initial.id)
        .is_ok());
    assert!(db
        .structural_snapshot("fixture_subject", &broken.id)
        .is_ok());
    assert!(db
        .latest_integrity_report("fixture_subject", "fixture_sheet", &initial.scope_hash)
        .unwrap()
        .is_some());
}

#[test]
fn network_failure_does_not_advance_previous_or_resolve_incidents() {
    let mut db = database();
    let initial = snapshot("net-initial");
    let baseline = approve(&mut db, &initial, 0);
    let mut broken = initial.clone();
    broken.id = "net-broken".into();
    broken.sheets[0].regions[0].headers[1].header = "Changed".into();
    persist(
        &mut db,
        &broken,
        &baseline,
        "net-check-a",
        vec![violation()],
    );
    let mut failed = initial.clone();
    failed.id = "net-failed".into();
    failed.sheets.clear();
    failed.coverage.complete = false;
    failed.coverage.metadata_complete = false;
    failed.connectivity = Connectivity::NetworkFailed;
    failed.coverage.issues.push(CoverageIssue {
        sheet_id: None,
        rule_id: None,
        critical: true,
        code: "network".into(),
    });
    let uncertainty = Finding {
        key: "capture".into(),
        sheet_id: None,
        rule_id: "capture".into(),
        code: "network".into(),
        severity: Severity::Critical,
        outcome: FindingOutcome::Inconclusive,
        expected: None,
        found: None,
        location: None,
    };
    let events = persist(
        &mut db,
        &failed,
        &baseline,
        "net-check-b",
        vec![uncertainty.clone()],
    );
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, "analysis_inconclusive");
    assert_eq!(
        db.previous_structural_snapshot("fixture_subject", "fixture_sheet", &initial.scope_hash)
            .unwrap()
            .unwrap()
            .id,
        broken.id
    );
    assert_eq!(
        db.latest_integrity_report("fixture_subject", "fixture_sheet", &initial.scope_hash)
            .unwrap()
            .unwrap()
            .status,
        IntegrityStatus::Indeterminate
    );
    let active: i64 = db
        .connection
        .query_row(
            "SELECT count(*) FROM integrity_incidents WHERE active=1 AND outcome='violation'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(active, 1);
    failed.id = "net-failed-again".into();
    assert!(persist(
        &mut db,
        &failed,
        &baseline,
        "net-check-c",
        vec![uncertainty]
    )
    .is_empty());
    let mut restored = initial.clone();
    restored.id = "net-restored".into();
    let events = persist(&mut db, &restored, &baseline, "net-check-d", Vec::new());
    assert!(events.iter().any(|e| e.kind == "recovered"));
    assert!(events.iter().any(|e| e.kind == "analysis_concluded"));
}

#[test]
fn old_comparison_and_old_baseline_cannot_commit_a_report() {
    let mut db = database();
    let initial = snapshot("cas-initial");
    let baseline = approve(&mut db, &initial, 0);
    let second = snapshot("cas-second");
    db.save_structural_snapshot(&second).unwrap();
    let old = report(&db, &second, &baseline, "cas-old", Vec::new(), Vec::new());
    let third = snapshot("cas-third");
    persist(&mut db, &third, &baseline, "cas-current", Vec::new());
    assert_eq!(
        db.persist_integrity_report(&second, &old),
        Err(EngineError::BaselineConflict)
    );
    let current = report(
        &db,
        &second,
        &baseline,
        "cas-old-baseline",
        Vec::new(),
        Vec::new(),
    );
    db.approve_structural_baseline("fixture_subject", &third.id, 1, true, "synthetic:test")
        .unwrap();
    assert_eq!(
        db.persist_integrity_report(&second, &current),
        Err(EngineError::BaselineConflict)
    );
}

#[test]
fn repeated_check_is_idempotent_and_changed_check_payload_is_rejected() {
    let mut db = database();
    let first = snapshot("idem-first");
    let baseline = approve(&mut db, &first, 0);
    let current = snapshot("idem-current");
    db.save_structural_snapshot(&current).unwrap();
    let delta = ContentDifference {
        key: "value:0:data:key:amount".into(),
        sheet_id: Some(0),
        kind: "value_changed".into(),
        category: "content".into(),
        location: Some("B2".into()),
        before: Some(fingerprint(&12)),
        after: Some(fingerprint(&13)),
        ambiguous: false,
    };
    let mut r = report(
        &db,
        &current,
        &baseline,
        "idempotent",
        Vec::new(),
        vec![delta],
    );
    let first_events = db.persist_integrity_report(&current, &r).unwrap();
    assert_eq!(first_events.len(), 1);
    assert_eq!(
        db.persist_integrity_report(&current, &r).unwrap(),
        first_events
    );
    r.differences[0].after = Some(fingerprint(&14));
    assert_eq!(
        db.persist_integrity_report(&current, &r),
        Err(EngineError::InvalidState)
    );
}

#[test]
fn retention_preserves_all_baselines_latest_good_latest_report_and_incident_state() {
    let mut db = database();
    let initial = snapshot("retain-initial");
    let baseline = approve(&mut db, &initial, 0);
    let old = snapshot("retain-old");
    persist(
        &mut db,
        &old,
        &baseline,
        "retain-check-old",
        vec![violation()],
    );
    let latest = snapshot("retain-latest");
    persist(
        &mut db,
        &latest,
        &baseline,
        "retain-check-latest",
        vec![violation()],
    );
    let unused = snapshot("retain-unused");
    db.save_structural_snapshot(&unused).unwrap();
    assert!(
        db.prune_integrity_history(
            "fixture_subject",
            "fixture_sheet",
            &initial.scope_hash,
            1,
            1
        )
        .unwrap()
            > 0
    );
    assert_eq!(
        db.structural_snapshot("fixture_subject", &old.id),
        Err(EngineError::NotFound)
    );
    assert_eq!(
        db.structural_snapshot("fixture_subject", &unused.id),
        Err(EngineError::NotFound)
    );
    assert!(db
        .structural_snapshot("fixture_subject", &initial.id)
        .is_ok());
    assert!(db
        .structural_snapshot("fixture_subject", &latest.id)
        .is_ok());
    assert_eq!(
        db.connection
            .query_row(
                "SELECT count(*) FROM integrity_incidents WHERE active=1",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert!(db
        .prune_integrity_history(
            "fixture_subject",
            "fixture_sheet",
            &initial.scope_hash,
            0,
            1
        )
        .is_err());
}

#[test]
fn real_capacity_is_thirty_distinct_files_and_demos_do_not_consume_it() {
    let mut db = database();
    db.seed_demo().unwrap();
    for n in 0..30 {
        let mut snap = snapshot(&format!("cap-{n}"));
        snap.spreadsheet_id = format!("file_{n}");
        db.save_structural_snapshot(&snap).unwrap();
    }
    let mut overflow = snapshot("cap-overflow");
    overflow.spreadsheet_id = "file_31".into();
    assert_eq!(
        db.save_structural_snapshot(&overflow),
        Err(EngineError::Limit)
    );
    let mut additional_scope = snapshot("cap-new-scope");
    additional_scope.spreadsheet_id = "file_0".into();
    additional_scope.scope.sheets[0].protect_position = true;
    additional_scope.scope_hash = additional_scope.scope.hash();
    db.save_structural_snapshot(&additional_scope).unwrap();
}

#[test]
fn concurrent_baseline_approval_allows_only_one_expected_version() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("concurrent.db");
    let mut first = Database::open(&path).unwrap();
    first.upsert_account("fixture_subject", None, None).unwrap();
    let snap = snapshot("concurrent");
    first.save_structural_snapshot(&snap).unwrap();
    let second = Database::open(&path).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let handles = [first, second]
        .into_iter()
        .map(|mut db| {
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                db.approve_structural_baseline(
                    "fixture_subject",
                    "concurrent",
                    0,
                    true,
                    "synthetic:test",
                )
            })
        })
        .collect::<Vec<_>>();
    let results = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(r, Err(EngineError::BaselineConflict)))
            .count(),
        1
    );
}

#[test]
fn report_transaction_rolls_back_when_event_persistence_fails() {
    let mut db = database();
    let initial = snapshot("rollback-initial");
    let baseline = approve(&mut db, &initial, 0);
    let current = snapshot("rollback-current");
    db.save_structural_snapshot(&current).unwrap();
    db.connection.execute_batch("CREATE TRIGGER fail_integrity_event BEFORE INSERT ON integrity_events BEGIN SELECT RAISE(ABORT,'synthetic_failure'); END;").unwrap();
    let r = report(
        &db,
        &current,
        &baseline,
        "rollback-check",
        vec![violation()],
        Vec::new(),
    );
    assert_eq!(
        db.persist_integrity_report(&current, &r),
        Err(EngineError::Storage)
    );
    assert!(db
        .latest_integrity_report("fixture_subject", "fixture_sheet", &initial.scope_hash)
        .unwrap()
        .is_none());
    assert_eq!(
        db.connection
            .query_row("SELECT count(*) FROM integrity_incidents", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        db.previous_structural_snapshot("fixture_subject", "fixture_sheet", &initial.scope_hash)
            .unwrap()
            .unwrap()
            .id,
        initial.id
    );
}
#[test]
fn partial_capture_records_each_observed_content_change_once_and_tracks_reversal() {
    let mut db = database();
    let base = snapshot("partial-base");
    let baseline = approve(&mut db, &base, 0);
    for (id, amount, expected) in [
        ("partial-b", 20, 1usize),
        ("partial-b-again", 20, 0),
        ("partial-a", 12, 1),
    ] {
        let mut current = base.clone();
        current.id = id.into();
        current.coverage.complete = false;
        current.connectivity = Connectivity::Partial;
        current.coverage.issues.push(CoverageIssue {
            sheet_id: None,
            rule_id: Some("unavailable_script".into()),
            critical: true,
            code: "network".into(),
        });
        let row = &mut current.sheets[0].regions[0].rows[0];
        row.cell_hashes
            .insert("amount".into(), fingerprint(&amount));
        row.fingerprint = fingerprint(&row.cell_hashes);
        db.save_structural_snapshot(&current).unwrap();
        let good = db
            .previous_structural_snapshot("fixture_subject", "fixture_sheet", &base.scope_hash)
            .unwrap();
        let context = db
            .content_observation("fixture_subject", "fixture_sheet", &base.scope_hash)
            .unwrap();
        let mut result =
            crate::integrity::evaluate(&baseline, context.snapshot.as_ref(), &current, id).unwrap();
        result.previous_snapshot_id = good.map(|s| s.id);
        result.observation_revision = context.revision;
        let events = db.persist_integrity_report(&current, &result).unwrap();
        assert_eq!(
            events.iter().filter(|e| e.category == "content").count(),
            expected,
            "{id}"
        );
        assert_eq!(
            db.previous_structural_snapshot("fixture_subject", "fixture_sheet", &base.scope_hash)
                .unwrap()
                .unwrap()
                .id,
            "partial-base"
        );
    }
    db.prune_integrity_history("fixture_subject", "fixture_sheet", &base.scope_hash, 1, 1)
        .unwrap();
    let context = db
        .content_observation("fixture_subject", "fixture_sheet", &base.scope_hash)
        .unwrap();
    assert_eq!(
        context.snapshot.unwrap().sheets[0].regions[0].rows[0].cell_hashes["amount"],
        fingerprint(&12)
    );
}
#[test]
fn partial_observation_cas_rejects_a_competing_report_even_when_good_snapshot_stays_same() {
    let mut db = database();
    let base = snapshot("cursor-base");
    let baseline = approve(&mut db, &base, 0);
    let mut partial = base.clone();
    partial.id = "cursor-partial".into();
    partial.coverage.complete = false;
    db.save_structural_snapshot(&partial).unwrap();
    let a = report(&db, &partial, &baseline, "cursor-check-a", vec![], vec![]);
    let b = report(&db, &partial, &baseline, "cursor-check-b", vec![], vec![]);
    db.persist_integrity_report(&partial, &a).unwrap();
    assert_eq!(
        db.persist_integrity_report(&partial, &b),
        Err(EngineError::BaselineConflict)
    );
}
#[test]
fn baseline_history_is_bounded_and_pageable() {
    let mut db = database();
    let mut base = snapshot("page-0");
    for version in 0..22 {
        base.id = format!("page-{version}");
        approve(&mut db, &base, version);
    }
    assert_eq!(
        db.structural_baselines("fixture_subject", "fixture_sheet", &base.scope_hash)
            .unwrap()
            .len(),
        20
    );
    assert_eq!(
        db.structural_baselines_page("fixture_subject", "fixture_sheet", &base.scope_hash, 20, 20)
            .unwrap()
            .len(),
        2
    );
}
