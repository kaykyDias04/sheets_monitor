use super::*;
use crate::cloud_sync::*;
use crate::google::SheetMetadata;
use std::collections::BTreeMap;
#[derive(Default)]
struct MemoryDrive {
    commits: BTreeMap<String, Vec<u8>>,
    offline: bool,
    downloads: usize,
}
impl SyncTransport for MemoryDrive {
    fn list(&mut self) -> SyncResult<Vec<RemoteFile>> {
        if self.offline {
            return Err(error("cloud_network"));
        }
        Ok(self
            .commits
            .keys()
            .map(|id| RemoteFile {
                file_id: id.clone(),
                commit_id: id.clone(),
            })
            .collect())
    }
    fn download(&mut self, file: &RemoteFile) -> SyncResult<Vec<u8>> {
        self.downloads += 1;
        Ok(self.commits[&file.commit_id].clone())
    }
    fn upload(&mut self, id: &str, bytes: &[u8]) -> SyncResult<()> {
        if self.offline {
            return Err(error("cloud_network"));
        }
        self.commits.insert(id.into(), bytes.to_vec());
        Ok(())
    }
}
fn db() -> Database {
    let mut db = Database::memory().unwrap();
    db.upsert_account("fixture_account", None, None).unwrap();
    db
}
fn sync(
    db: &mut Database,
    drive: &mut MemoryDrive,
    resolution: Option<&str>,
) -> SyncResult<SyncStatus> {
    synchronize(
        db,
        "fixture_account",
        drive,
        resolution,
        None,
        |db, hash, id, p| db.cloud_finish("fixture_account", hash, id, p),
    )
}
fn resolve(
    db: &mut Database,
    drive: &mut MemoryDrive,
    resolution: &str,
    conflict: &SyncStatus,
) -> SyncResult<SyncStatus> {
    synchronize(
        db,
        "fixture_account",
        drive,
        Some(resolution),
        conflict.conflict_token.as_deref(),
        |db, hash, id, p| db.cloud_finish("fixture_account", hash, id, p),
    )
}
fn fixture(db: &mut Database, name: &str, baseline: bool) {
    let capture = db.cloud_capture("fixture_account").unwrap();
    let hash = canonical_hash(&capture.payload).unwrap();
    let mut payload = capture.payload;
    let s = crate::integrity::fixtures::snapshot();
    payload.registries = vec![SyncRegistry {
        spreadsheet_id: s.spreadsheet_id.clone(),
        name: name.into(),
        paused: false,
        removed: false,
        scope: Some(s.scope.clone()),
        selected_sheet_ids: vec![7],
        tabs: vec![SheetMetadata {
            sheet_id: 7,
            title: "Records".into(),
            index: 0,
            sheet_type: Some("GRID".into()),
            row_count: Some(100),
            column_count: Some(10),
            hidden: false,
        }],
    }];
    if baseline {
        payload.baselines = vec![BaselineVersion {
            id: "original-human-reference".into(),
            version: 1,
            active: true,
            approved_at: "2026-01-01T09:00:00Z".into(),
            approval_origin: "human:native".into(),
            snapshot: s,
        }];
    }
    db.cloud_finish("fixture_account", &hash, "test-seed", Some(&payload))
        .unwrap();
    if let Some(base) = capture.base_commit {
        db.connection.execute("UPDATE cloud_sync_state SET base_commit=?1,base_payload_hash=?2,last_synced_at=?3 WHERE account_subject='fixture_account'",params![base,capture.base_payload_hash,capture.last_synced_at]).unwrap();
    } else {
        db.connection
            .execute(
                "DELETE FROM cloud_sync_state WHERE account_subject='fixture_account'",
                [],
            )
            .unwrap();
    }
}
#[test]
fn two_databases_round_trip_preserves_approved_reference_and_cache() {
    let mut a = db();
    let mut b = db();
    let mut drive = MemoryDrive::default();
    fixture(&mut a, "Original", true);
    sync(&mut a, &mut drive, None).unwrap();
    sync(&mut b, &mut drive, None).unwrap();
    let pa = a.cloud_capture("fixture_account").unwrap().payload;
    let pb = b.cloud_capture("fixture_account").unwrap().payload;
    assert_eq!(canonical_hash(&pa).unwrap(), canonical_hash(&pb).unwrap());
    assert_eq!(pb.baselines[0].id, "original-human-reference");
    assert_eq!(pb.baselines[0].approval_origin, "human:native");
    let reads = drive.downloads;
    sync(&mut b, &mut drive, None).unwrap();
    assert_eq!(reads, drive.downloads);
    let encoded = encode(&pa).unwrap();
    for forbidden in [
        "access_token",
        "refresh_token",
        "client_secret",
        "start_with_windows",
        "run_minimized",
        "authorization",
    ] {
        assert!(!encoded.contains(forbidden));
    }
}
#[test]
fn offline_edit_conflicts_and_explicit_resolution_merges_heads() {
    let mut a = db();
    let mut b = db();
    let mut drive = MemoryDrive::default();
    fixture(&mut a, "Original", false);
    sync(&mut a, &mut drive, None).unwrap();
    sync(&mut b, &mut drive, None).unwrap();
    fixture(&mut a, "Device A", false);
    fixture(&mut b, "Device B", false);
    drive.offline = true;
    let before = canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap();
    assert!(sync(&mut b, &mut drive, None).is_err());
    assert_eq!(
        before,
        canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap()
    );
    drive.offline = false;
    sync(&mut a, &mut drive, None).unwrap();
    let conflict = sync(&mut b, &mut drive, None).unwrap();
    assert_eq!(conflict.state, "conflict");
    assert_eq!(
        b.cloud_capture("fixture_account")
            .unwrap()
            .payload
            .registries[0]
            .name,
        "Device B"
    );
    let resolution = format!("remote:{}", conflict.conflicts[0].commit_id);
    resolve(&mut b, &mut drive, &resolution, &conflict).unwrap();
    assert_eq!(
        b.cloud_capture("fixture_account")
            .unwrap()
            .payload
            .registries[0]
            .name,
        "Device A"
    );
    sync(&mut a, &mut drive, None).unwrap();
}
#[test]
fn first_connection_with_local_configuration_requires_decision() {
    let mut a = db();
    let mut b = db();
    let mut drive = MemoryDrive::default();
    fixture(&mut a, "A", false);
    fixture(&mut b, "B", false);
    sync(&mut a, &mut drive, None).unwrap();
    let conflict = sync(&mut b, &mut drive, None).unwrap();
    assert_eq!(conflict.state, "conflict");
    resolve(&mut b, &mut drive, "keep_local", &conflict).unwrap();
    sync(&mut a, &mut drive, None).unwrap();
    assert_eq!(
        a.cloud_capture("fixture_account")
            .unwrap()
            .payload
            .registries[0]
            .name,
        "B"
    );
}
#[test]
fn account_hash_and_incomplete_graph_are_rejected_without_apply() {
    let mut a = db();
    let mut drive = MemoryDrive::default();
    fixture(&mut a, "Original", false);
    sync(&mut a, &mut drive, None).unwrap();
    let mut b = Database::memory().unwrap();
    b.upsert_account("another_account", None, None).unwrap();
    assert!(synchronize(
        &mut b,
        "another_account",
        &mut drive,
        None,
        None,
        |db, h, id, p| db.cloud_finish("another_account", h, id, p)
    )
    .is_err());
    assert!(b
        .cloud_capture("another_account")
        .unwrap()
        .payload
        .registries
        .is_empty());
    let bytes = drive.commits.values().next().unwrap().clone();
    let mut c: Commit = serde_json::from_slice(&bytes).unwrap();
    c.parents = vec!["a".repeat(64)];
    drive.commits.clear();
    drive
        .commits
        .insert(c.id().unwrap(), encode(&c).unwrap().into_bytes());
    assert_eq!(
        sync(&mut a, &mut drive, None).unwrap_err().code,
        "cloud_missing_parent"
    );
}
#[test]
fn local_cas_and_invalid_baseline_roll_back() {
    let mut a = db();
    let mut drive = MemoryDrive::default();
    fixture(&mut a, "A", true);
    sync(&mut a, &mut drive, None).unwrap();
    let mut b = db();
    let result = synchronize(
        &mut b,
        "fixture_account",
        &mut drive,
        None,
        None,
        |db, h, id, p| {
            fixture(db, "Concurrent", false);
            db.cloud_finish("fixture_account", h, id, p)
        },
    );
    assert_eq!(result.unwrap_err().code, "cloud_local_changed");
    assert_eq!(
        b.cloud_capture("fixture_account")
            .unwrap()
            .payload
            .registries[0]
            .name,
        "Concurrent"
    );
    let current = b.cloud_capture("fixture_account").unwrap().payload;
    let hash = canonical_hash(&current).unwrap();
    let mut invalid = a.cloud_capture("fixture_account").unwrap().payload;
    invalid.baselines[0].snapshot.account_subject = "another_account".into();
    assert!(b
        .cloud_finish("fixture_account", &hash, "invalid", Some(&invalid))
        .is_err());
    assert_eq!(
        hash,
        canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap()
    );
}
#[test]
fn canonical_objects_and_migration_bounds() {
    assert_eq!(
        canonical_hash(&serde_json::json!({"b":2,"a":1})).unwrap(),
        canonical_hash(&serde_json::json!({"a":1,"b":2})).unwrap()
    );
    let c = rusqlite::Connection::open_in_memory().unwrap();
    c.execute_batch(include_str!("../migrations/0001_foundation.sql"))
        .unwrap();
    c.execute_batch(include_str!("../migrations/0002_google_integration.sql"))
        .unwrap();
    c.execute_batch(include_str!("../migrations/0003_integrity_engine.sql"))
        .unwrap();
    c.execute_batch(include_str!(
        "../migrations/0004_operational_management.sql"
    ))
    .unwrap();
    c.execute_batch(include_str!("../migrations/0005_continuous_monitoring.sql"))
        .unwrap();
    c.execute_batch(
        "PRAGMA foreign_keys=ON;
         INSERT INTO accounts(id,provider_subject,display_name,is_demo,created_at)
             VALUES('account','migration-fixture','Synthetic migration',0,'2026-01-01T00:00:00Z');
         INSERT INTO registered_spreadsheets(account_id,spreadsheet_id,name,tabs_json,created_at,updated_at)
             VALUES('account','sheet','Synthetic migration','[]','2026-01-01T00:00:00Z','2026-01-01T00:00:00Z');
         INSERT INTO integrity_scopes(id,account_id,spreadsheet_id,scope_hash,scope_json,created_at)
             VALUES('scope','account','sheet',printf('%064d',0),'{}','2026-01-01T00:00:00Z');
         INSERT INTO structural_snapshots(id,scope_id,account_id,captured_at,fingerprint,snapshot_json,complete)
             VALUES('snapshot','scope','account','2026-01-01T00:00:00Z',printf('%064d',0),'{}',1);
         INSERT INTO baseline_versions(id,scope_id,account_id,snapshot_id,version,approved_at,approval_origin)
             VALUES('baseline','scope','account','snapshot',1,'2026-01-01T00:00:00Z','synthetic:test');
         INSERT INTO integrity_reports(check_id,scope_id,account_id,snapshot_id,baseline_id,status,connectivity,detected_at,report_json)
             VALUES('check','scope','account','snapshot','baseline','online','reachable','2026-01-01T00:00:00Z','{}');
         INSERT INTO monitoring_operational_events(id,account_id,spreadsheet_id,kind,code,detected_ms)
             VALUES('operational','account','sheet','unavailable','network',0);",
    )
    .unwrap();
    c.execute(
        "UPDATE app_settings SET interval_seconds=180,demo_seeded=1,updated_at='preserved'",
        [],
    )
    .unwrap();
    for (id, source, state, category, rule) in [
        ("free-pending", "integrity", "pending", "content", ""),
        ("free-retry", "integrity", "retry", "content", ""),
        (
            "free-permission",
            "integrity",
            "permission_blocked",
            "integrity",
            "",
        ),
        ("active-dispatch", "integrity", "dispatching", "content", ""),
        (
            "sent-history",
            "integrity",
            "dispatch_submitted",
            "content",
            "",
        ),
        (
            "actionable",
            "integrity",
            "pending",
            "integrity",
            "approved-rule",
        ),
        ("operational", "operational", "pending", "content", ""),
    ] {
        if source == "integrity" {
            let json = serde_json::json!({"category":category,"rule_id":rule}).to_string();
            c.execute("INSERT INTO integrity_events(event_id,check_id,scope_id,account_id,severity,detected_at,fingerprint,event_json) VALUES(?1,'check','scope','account','info','2026-01-01T00:00:00Z',?1,?2)",params![id,json]).unwrap();
            c.execute("UPDATE notification_outbox SET id=?1,state=?2,lease_owner='owner',lease_until_ms=100,attempt_id='attempt' WHERE integrity_event_id=?1",params![id,state]).unwrap();
        } else {
            c.execute("INSERT INTO notification_outbox(id,account_id,spreadsheet_id,operational_event_id,source,spreadsheet_name,severity,diagnosis,detected_at,created_ms,expires_ms,state,next_attempt_ms,group_id,lease_owner,lease_until_ms,attempt_id) VALUES(?1,'account','sheet',?1,'operational','name','info','diagnosis','2026-01-01T00:00:00Z',0,999999999,?2,0,'group','owner',100,'attempt')",params![id,state]).unwrap();
        }
    }
    c.execute_batch(include_str!("../migrations/0006_cloud_sync.sql"))
        .unwrap();
    for (id, expected) in [
        ("free-pending", "suppressed_by_preference"),
        ("free-retry", "suppressed_by_preference"),
        ("free-permission", "suppressed_by_preference"),
        ("active-dispatch", "dispatching"),
        ("sent-history", "dispatch_submitted"),
        ("actionable", "pending"),
        ("operational", "pending"),
    ] {
        let (actual, owner): (String, Option<String>) = c
            .query_row(
                "SELECT state,lease_owner FROM notification_outbox WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(actual, expected);
        assert_eq!(owner.is_none(), expected == "suppressed_by_preference");
    }
    let saved: (u32, u32, String) = c
        .query_row(
            "SELECT interval_seconds,demo_seeded,updated_at FROM app_settings",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(saved, (180, 1, "preserved".into()));
    assert!(c
        .execute("UPDATE app_settings SET interval_seconds=600", [])
        .is_ok());
    assert!(c
        .execute("UPDATE app_settings SET interval_seconds=601", [])
        .is_err());
    assert_eq!(
        c.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
            .get::<_, u32>(
            0
        ))
        .unwrap(),
        0
    );
}

#[test]
fn multiple_heads_require_explicit_merge_and_keep_all_parents() {
    let mut a = db();
    let mut drive = MemoryDrive::default();
    fixture(&mut a, "Original", false);
    sync(&mut a, &mut drive, None).unwrap();
    let base = drive.commits.keys().next().unwrap().clone();
    let original: Commit = serde_json::from_slice(&drive.commits[&base]).unwrap();
    let mut ids = Vec::new();
    for name in ["Branch A", "Branch B"] {
        let mut child = original.clone();
        child.parents = vec![base.clone()];
        child.device_id = name.into();
        child.payload.registries[0].name = name.into();
        let id = child.id().unwrap();
        drive
            .commits
            .insert(id.clone(), encode(&child).unwrap().into_bytes());
        ids.push(id);
    }
    let conflict = sync(&mut a, &mut drive, None).unwrap();
    assert_eq!(conflict.conflicts.len(), 2);
    resolve(&mut a, &mut drive, "keep_local", &conflict).unwrap();
    let capture = a.cloud_capture("fixture_account").unwrap();
    let merged: Commit =
        serde_json::from_slice(&drive.commits[capture.base_commit.as_ref().unwrap()]).unwrap();
    assert!(ids.iter().all(|id| merged.parents.contains(id)));
    assert_eq!(capture.payload.registries[0].name, "Original");
}

#[test]
fn restore_preserves_device_flags_and_backup_can_be_recovered() {
    let mut a = db();
    let mut b = db();
    let mut drive = MemoryDrive::default();
    fixture(&mut a, "Remote", false);
    fixture(&mut b, "Local", false);
    let aid: String = b
        .connection
        .query_row(
            "SELECT id FROM accounts WHERE provider_subject='fixture_account'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let prefs = Preferences {
        start_with_windows: true,
        run_minimized: true,
        ..Preferences::default()
    };
    b.connection.execute("INSERT INTO management_preferences(account_id,preferences_json) VALUES(?1,?2) ON CONFLICT(account_id) DO UPDATE SET preferences_json=excluded.preferences_json",params![aid,encode(&prefs).unwrap()]).unwrap();
    sync(&mut a, &mut drive, None).unwrap();
    let conflict = sync(&mut b, &mut drive, None).unwrap();
    let chosen = format!("remote:{}", conflict.conflicts[0].commit_id);
    resolve(&mut b, &mut drive, &chosen, &conflict).unwrap();
    let stored: String = b
        .connection
        .query_row(
            "SELECT preferences_json FROM management_preferences WHERE account_id=?1",
            [&aid],
            |r| r.get(0),
        )
        .unwrap();
    let stored: Preferences = decode(&stored).unwrap();
    assert!(stored.start_with_windows && stored.run_minimized);
    let backup:String=b.connection.query_row("SELECT id FROM cloud_sync_backups WHERE account_subject='fixture_account' AND json_extract(payload_json,'$.registries[0].name')='Local' ORDER BY rowid DESC LIMIT 1",[],|r|r.get(0)).unwrap();
    let hash = canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap();
    b.cloud_restore_backup("fixture_account", &backup, &hash)
        .unwrap();
    assert_eq!(
        b.cloud_capture("fixture_account")
            .unwrap()
            .payload
            .registries[0]
            .name,
        "Local"
    );
}

#[test]
fn pending_tab_selection_and_existing_history_preferences_remain_syncable() {
    let mut a = db();
    fixture(&mut a, "Pending tabs", false);
    let mut p = a.cloud_capture("fixture_account").unwrap().payload;
    p.registries[0].selected_sheet_ids = vec![];
    for days in [1, 3650] {
        p.preferences.history_days = days;
        p.validate("fixture_account").unwrap();
    }
    for days in [0, 3651] {
        p.preferences.history_days = days;
        assert!(p.validate("fixture_account").is_err());
    }
}

#[test]
fn effective_restore_hash_preserves_extra_local_tombstone_without_reupload() {
    let mut a = db();
    let mut b = db();
    let mut drive = MemoryDrive::default();
    fixture(&mut a, "Remote", false);
    fixture(&mut b, "Local", false);
    let mut current = b.cloud_capture("fixture_account").unwrap().payload;
    let mut tombstone = current.registries[0].clone();
    tombstone.spreadsheet_id = "archived_fixture".into();
    tombstone.removed = true;
    current.registries.push(tombstone);
    let hash = canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap();
    b.cloud_finish("fixture_account", &hash, "fixture-local", Some(&current))
        .unwrap();
    b.connection
        .execute(
            "DELETE FROM cloud_sync_state WHERE account_subject='fixture_account'",
            [],
        )
        .unwrap();
    sync(&mut a, &mut drive, None).unwrap();
    let conflict = sync(&mut b, &mut drive, None).unwrap();
    resolve(
        &mut b,
        &mut drive,
        &format!("remote:{}", conflict.conflicts[0].commit_id),
        &conflict,
    )
    .unwrap();
    let capture = b.cloud_capture("fixture_account").unwrap();
    assert_eq!(
        capture.base_payload_hash.as_deref(),
        Some(canonical_hash(&capture.payload).unwrap().as_str())
    );
    assert!(capture
        .payload
        .registries
        .iter()
        .any(|r| r.spreadsheet_id == "archived_fixture" && r.removed));
    let before = drive.commits.len();
    sync(&mut b, &mut drive, None).unwrap();
    assert_eq!(before, drive.commits.len());
}

#[test]
fn conflict_decisions_require_the_displayed_heads_and_local_configuration() {
    let mut a = db();
    let mut b = db();
    let mut drive = MemoryDrive::default();
    fixture(&mut a, "Remote", false);
    fixture(&mut b, "Local", false);
    sync(&mut a, &mut drive, None).unwrap();
    let old = sync(&mut b, &mut drive, None).unwrap();
    fixture(&mut a, "New remote head", false);
    sync(&mut a, &mut drive, None).unwrap();
    let before = canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap();
    let uploads = drive.commits.len();
    for choice in [
        "keep_local".to_string(),
        format!("remote:{}", old.conflicts[0].commit_id),
    ] {
        assert_eq!(
            resolve(&mut b, &mut drive, &choice, &old).unwrap_err().code,
            "cloud_stale_resolution"
        );
    }
    assert_eq!(drive.commits.len(), uploads);
    assert_eq!(
        canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap(),
        before
    );
    let current = sync(&mut b, &mut drive, None).unwrap();
    fixture(&mut b, "Changed after viewing", false);
    let before = canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap();
    for choice in [
        "keep_local".to_string(),
        format!("remote:{}", current.conflicts[0].commit_id),
    ] {
        assert_eq!(
            resolve(&mut b, &mut drive, &choice, &current)
                .unwrap_err()
                .code,
            "cloud_stale_resolution"
        );
    }
    assert_eq!(drive.commits.len(), uploads);
    assert_eq!(
        canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap(),
        before
    );
    assert_eq!(
        sync(&mut b, &mut drive, Some("keep_local"))
            .unwrap_err()
            .code,
        "cloud_stale_resolution"
    );
    let legacy: SyncStatus =
        decode(r#"{"state":"conflict","message":"legacy","conflicts":[],"last_synced_at":null}"#)
            .unwrap();
    assert!(legacy.conflict_token.is_none());
}

#[test]
fn long_names_and_deleted_selected_tabs_round_trip_without_weakening_limits() {
    let mut a = db();
    let mut b = db();
    let mut drive = MemoryDrive::default();
    fixture(&mut a, &"x".repeat(4096), false);
    let capture = a.cloud_capture("fixture_account").unwrap();
    let mut p = capture.payload;
    p.registries[0].tabs.clear();
    p.registries[0].selected_sheet_ids = vec![7];
    let old = canonical_hash(&a.cloud_capture("fixture_account").unwrap().payload).unwrap();
    a.cloud_finish("fixture_account", &old, "deleted-tab", Some(&p))
        .unwrap();
    sync(&mut a, &mut drive, None).unwrap();
    sync(&mut b, &mut drive, None).unwrap();
    assert_eq!(
        canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap(),
        canonical_hash(&p).unwrap()
    );
    p.registries[0].name.push('x');
    assert!(p.validate("fixture_account").is_err());
    p.registries[0].name.pop();
    p.registries[0].selected_sheet_ids = vec![7, 7];
    assert!(p.validate("fixture_account").is_err());
    p.registries[0].selected_sheet_ids =
        (0..=crate::integrity::MAX_MONITORED_TABS as u32).collect();
    assert!(p.validate("fixture_account").is_err());
}

#[test]
fn independent_baseline_ordinals_preserve_uuids_history_and_future_sync() {
    let mut a = db();
    let mut b = db();
    let mut c = db();
    let mut drive = MemoryDrive::default();
    fixture(&mut a, "Remote", true);
    let remote = a.cloud_capture("fixture_account").unwrap().payload;
    let mut local = remote.clone();
    local.registries[0].name = "Local".into();
    local.baselines[0].id = "independent-human-reference".into();
    local.baselines[0].snapshot.id = "independent-snapshot".into();
    let hash = canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap();
    b.cloud_finish("fixture_account", &hash, "seed-local", Some(&local))
        .unwrap();
    b.connection
        .execute("DELETE FROM cloud_sync_state", [])
        .unwrap();
    b.connection
        .execute(
            "UPDATE integrity_scopes SET previous_snapshot_id='independent-snapshot'",
            [],
        )
        .unwrap();
    b.connection.execute("INSERT INTO integrity_observation_components(scope_id,component_key,component_json,observed_at) SELECT id,'old','{}','2026-01-01T00:00:00Z' FROM integrity_scopes", []).unwrap();
    let revision: u64 = b
        .connection
        .query_row(
            "SELECT observation_revision FROM integrity_scopes",
            [],
            |r| r.get(0),
        )
        .unwrap();
    sync(&mut a, &mut drive, None).unwrap();
    let conflict = sync(&mut b, &mut drive, None).unwrap();
    resolve(
        &mut b,
        &mut drive,
        &format!("remote:{}", conflict.conflicts[0].commit_id),
        &conflict,
    )
    .unwrap();
    let capture = b.cloud_capture("fixture_account").unwrap();
    assert_eq!(capture.payload.baselines.len(), 2);
    assert!(capture.payload.baselines.iter().all(|b| b.version == 1));
    assert_eq!(
        capture
            .payload
            .baselines
            .iter()
            .filter(|b| b.active)
            .count(),
        1
    );
    let ordinals: Vec<(String, u32, u32, String)> = b
        .connection
        .prepare(
            "SELECT id,version,sync_version,snapshot_id FROM baseline_versions ORDER BY version",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(
        ordinals,
        vec![
            (
                "independent-human-reference".into(),
                1,
                1,
                "independent-snapshot".into()
            ),
            (
                "original-human-reference".into(),
                2,
                1,
                remote.baselines[0].snapshot.id.clone()
            ),
        ]
    );
    let observation: (Option<String>, u64) = b
        .connection
        .query_row(
            "SELECT previous_snapshot_id,observation_revision FROM integrity_scopes",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(observation, (None, revision + 1));
    assert_eq!(
        b.connection
            .query_row(
                "SELECT count(*) FROM integrity_observation_components",
                [],
                |r| r.get::<_, u32>(0)
            )
            .unwrap(),
        0
    );
    let uploads = drive.commits.len();
    sync(&mut b, &mut drive, None).unwrap();
    assert_eq!(drive.commits.len(), uploads);
    let hash = canonical_hash(&capture.payload).unwrap();
    b.cloud_finish(
        "fixture_account",
        &hash,
        capture.base_commit.as_ref().unwrap(),
        Some(&remote),
    )
    .unwrap();
    assert_eq!(
        canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap(),
        hash
    );
    fixture(&mut b, "Publish both histories", false);
    sync(&mut b, &mut drive, None).unwrap();
    sync(&mut c, &mut drive, None).unwrap();
    assert_eq!(
        canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap(),
        canonical_hash(&c.cloud_capture("fixture_account").unwrap().payload).unwrap()
    );
    sync(&mut a, &mut drive, None).unwrap();
    assert_eq!(
        canonical_hash(&a.cloud_capture("fixture_account").unwrap().payload).unwrap(),
        canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap()
    );
    assert_eq!(
        a.connection
            .query_row(
                "SELECT version FROM baseline_versions WHERE id='original-human-reference'",
                [],
                |r| r.get::<_, u32>(0)
            )
            .unwrap(),
        1
    );
    let uploads = drive.commits.len();
    for database in [&mut a, &mut b, &mut c] {
        assert_eq!(sync(database, &mut drive, None).unwrap().state, "synced");
    }
    assert_eq!(drive.commits.len(), uploads);
    let backups: u32 = b
        .connection
        .query_row("SELECT count(*) FROM cloud_sync_backups", [], |r| r.get(0))
        .unwrap();
    let mut invalid = remote.clone();
    invalid.baselines[0].approved_at = "2026-01-02T09:00:00Z".into();
    let hash = canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap();
    assert_eq!(
        b.cloud_finish("fixture_account", &hash, "invalid", Some(&invalid))
            .unwrap_err()
            .code,
        "cloud_immutable_collision"
    );
    assert_eq!(
        b.connection
            .query_row("SELECT count(*) FROM cloud_sync_backups", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        backups
    );
    assert_eq!(
        b.connection
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                .get::<_, u32>(
                0
            ))
            .unwrap(),
        0
    );
}

#[test]
fn restored_script_links_require_real_reads_and_keep_existing_local_authorization() {
    let mut a = db();
    let mut b = db();
    let mut drive = MemoryDrive::default();
    fixture(&mut a, "Script reference", false);
    let capture = a.cloud_capture("fixture_account").unwrap();
    let hash = canonical_hash(&capture.payload).unwrap();
    let mut p = capture.payload;
    let mut snapshot = crate::integrity::fixtures::snapshot();
    crate::integrity::fixtures::attach_script(&mut snapshot);
    p.registries[0].scope = Some(snapshot.scope.clone());
    p.baselines = vec![BaselineVersion {
        id: "script-human-reference".into(),
        version: 1,
        active: true,
        approved_at: "2026-01-01T09:00:00Z".into(),
        approval_origin: "human:native".into(),
        snapshot,
    }];
    a.cloud_finish("fixture_account", &hash, "script-seed", Some(&p))
        .unwrap();
    a.connection
        .execute("DELETE FROM cloud_sync_state", [])
        .unwrap();
    sync(&mut a, &mut drive, None).unwrap();
    sync(&mut b, &mut drive, None).unwrap();
    assert_eq!(
        b.connection
            .query_row("SELECT count(*) FROM integration_associations", [], |r| r
                .get::<_, u32>(
                0
            ))
            .unwrap(),
        1
    );
    let resources: u32 = b.connection.query_row("SELECT count(*) FROM integration_resources WHERE last_status='error' AND last_error_kind='not_consulted' AND last_queried_at='' AND source_hash IS NULL", [], |r| r.get(0)).unwrap();
    assert_eq!(resources, 2);
    assert_eq!(
        b.connection
            .query_row("SELECT count(*) FROM integration_queries", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        b.connection
            .query_row(
                "SELECT authorization FROM registered_spreadsheets",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "unknown"
    );
    b.connection.execute("UPDATE integration_resources SET last_status='success',last_error_kind=NULL,last_queried_at='2026-11-01T00:00:00Z'", []).unwrap();
    let hash = canonical_hash(&b.cloud_capture("fixture_account").unwrap().payload).unwrap();
    b.cloud_finish("fixture_account", &hash, "repeat", Some(&p))
        .unwrap();
    assert_eq!(b.connection.query_row("SELECT count(*) FROM integration_resources WHERE last_status='success' AND last_queried_at='2026-11-01T00:00:00Z'", [], |r| r.get::<_, u32>(0)).unwrap(), 2);
}
