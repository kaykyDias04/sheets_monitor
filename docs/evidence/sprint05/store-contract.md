# Sprint 05 — proposed persistence contract (Gate 0)

Implementation authorized after Gate 0 and baseline reconciliation. Existing migrations 0001–0004 remain byte-for-byte unchanged. Current integrity writer is centralized in `database/integrity_store.rs`; registered file eligibility and baseline versions already exist. Monitoring tables use provider subject resolution to the non-demo account ID, never caller-provided internal account IDs.

## Scheduling and ownership

`monitoring_control(subject)` returns `{enabled, generation}`; default disabled. `set_monitoring_enabled(subject, enabled, now_ms)` persists the choice, increments generation and clears active leases. The persisted monitoring generation supplements the existing in-memory authentication generation; both must be checked by the service.

`monitoring_candidates(subject)` returns only nonremoved, unpaused registry entries with valid scope, selected sheets matching that scope, current scope hash and active approved baseline. Each candidate contains spreadsheet ID/name, MonitorScope, scope hash, registry revision and baseline ID. Authentication/scope grants remain a service precondition, not invented by the database.

`ensure_monitoring_schedule(subject, candidate, initial_next_ms)` inserts a missing schedule; an existing revision/hash change invalidates the lease and resets the next date. The scheduler supplies deterministic staggering. `monitoring_schedules(subject)` returns rows with last start/completion, next due time, failure count, operation condition, lease owner/expiry and generation/revision. Dates are UTC milliseconds internally and serialized as such; integrity event timestamps remain their existing RFC3339 representation.

`claim_monitoring_check(subject, spreadsheet_id, expected_revision, generation, owner, now_ms, lease_ms)` atomically validates current enabled control, registry/baseline eligibility and due/lease state. It returns a lease token containing generation, registry revision and owner. The service provides global max-three concurrency and shared manual admission; database lease CAS provides per-file exclusion. `finish_monitoring_check(subject, lease, outcome, next_ms, now_ms)` accepts only the exact live lease and current registry/control. Cancelled/contention completion releases ownership without fabricating operational failure. Successful integrity report persistence continues through the Sprint 03 engine and its revision guard.

`recover_monitoring_leases(subject, owner, now_ms)` releases abandoned/expired ownership on process initialization; the scheduler then supplies staggered due dates. No catch-up claim implies observing intermediate Google changes. Expiry must exceed the bounded capture budget or be renewed; a stale worker cannot complete another owner's lease.

## Events and notification transaction

Migration 0005 adds an AFTER INSERT trigger from `integrity_events` to `notification_outbox`, in the same transaction as detection. Existing Sprint 03 incident/content identity and dedup remain authoritative. No historical backfill generates surprise old alerts.

Operational errors and recovery use a separate typed `monitoring_operational_events` table, without fabricated structural reports, baselines or findings. A per-file current condition stores the last stable error code; an unchanged condition emits no repeated events. Changing condition and recovery each receive a persisted unique event ID and matching outbox row atomically. Diagnostics contain allowlisted error codes rather than remote response content.

Outbox rows identify the exact source event and account, spreadsheet, severity, created time, retry eligibility, dispatch state, lease/attempt ID and optional visual grouping ID. Every distinct event has its own row even when grouping is used. Delivery APIs resolve the current subject and claim/account-check immediately before dispatch. Preference off or permission denied records a deferred state while monitoring continues. A later authorized attempt may retry.

`claim_notification(subject, owner, now_ms)` checks the persisted product preference and durably records the attempt before native dispatch. `finish_notification_attempt(subject, id, attempt_id, outcome, now_ms)` uses CAS. Outcomes distinguish dispatch submitted, retryable failure, evidence-backed permission blocked and unsupported. Official stable Tauri plugin research shows `show()` only submits an asynchronous native task and discards its later OS error; successful return therefore records `submitted_ms`, never OS acceptance, display or reading. Desktop permission state also cannot establish Windows settings/DND. Crash between dispatch and SQLite acknowledgement is inherently ambiguous; stable desktop grouping is not supported by the researched plugin, so physical duplicate display after retry remains possible. Pending/ambiguous attempts are never silently called delivered. See `docs/sprint05-native-research.md`.

## History, deletion and retention

Account-scoped notification/operational history uses validated limit/offset pagination. Default history is 90 days, with the effective scope's configured policy retained. Principal Gate 0 decision supersedes the initial indefinite-pending proposal: pending/denied/retry notifications expire explicitly at the effective scope TTL to bound storage. `expire_notifications` marks them `expired`, never submitted/delivered; subsequent source history pruning cascades outbox and attempts. Delivery before TTL and eventual physical display are not guaranteed. Product preference off creates terminal `suppressed_by_preference` rows immediately in the integrity trigger/operational transaction; disabling the preference also suppresses queued work through `suppress_disabled_notifications`. Enabling later never backfills those historical alerts. Suppressed records remain auditable until the same source TTL.

The existing explicit Sprint 04 `delete_activity` operation may remove notification rows and attempts through cascading source foreign keys, and removes operational activity in that same explicit transaction. It must not be blocked by newly introduced FKs. Current baseline references and immutable approved history remain untouched.

## Status overlay and validation

The existing registry status is preserved as confirmed integrity proof. UI/service state reports checking and current operational unavailability separately; operational errors never write an Offline report. A previously confirmed Offline proof remains available during an operational failure. The principal may expose this overlay in the dashboard without duplicating integrity classification.

Focused tests prove migration preservation/rollback; event-trigger atomicity/rollback; distinct event representation; stable operational dedup and recovery; account isolation; generation/revision/owner CAS; lease crash recovery; attempts and ambiguous restart; permission-disabled independence; explicit TTL expiration of undelivered events; and explicit erasure cascades. Tests use synthetic accounts/resources only.

## Lane agreement

Scheduler owns deterministic staggering. `defer_monitoring_schedule(subject, spreadsheet_id, next_ms)` reschedules only unleased work for interval changes/restart/suspension. Lease duration is 300,000ms around a bounded 180,000ms collection budget and individual 30s reads. `MonitoringLease` additionally contains a UUID `lease_id` and `manual` flag; the flag permits explicit manual work while global scheduling is paused, while preserving exact current scope and per-file pause guards.

`is_monitoring_lease_current(subject, lease, now_ms)` validates generation, current registry revision/scope/baseline, ownership and expiry. `persist_monitored_integrity_report(snapshot, report, registry_revision, lease, now_ms)` performs that same validation inside the existing integrity report write transaction; a separate read check would leave a race with pause/configuration changes. Prior persistence entry points retain their signatures. Completion uses `CheckOutcome::{Success, OperationalFailure { code }, Cancelled}` and returns whether CAS succeeded.

Concrete records: `MonitoringControl { enabled, generation }`; `MonitoringCandidate { spreadsheet_id, name, scope, scope_hash, registry_revision, baseline_id }`; `MonitoringSchedule { spreadsheet_id, registry_revision, scope_hash, generation, last_started_ms?, last_completed_ms?, next_run_ms, failure_count, operational_code?, lease_owner?, lease_until_ms? }`. Public store methods return `CommandError` and existing engine writers continue using `EngineError`.

Native lane agrees `NotificationOutcome::DispatchSubmitted` is terminal for the local submission queue only. The immutable attempt journal preserves crash ambiguity and retry history. Permission blocked is used only when actually established; desktop plugin permission reports alone cannot establish it. Clicks carry only an opaque outbox ID and must resolve through the current account before revealing event details.
