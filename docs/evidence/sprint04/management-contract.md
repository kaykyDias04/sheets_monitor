# Proposed operational IPC contract - Sprint 04

Named commands return EngineEnvelope<T> (generation, account_subject, data), snake_case nested inputs; always current authenticated account, no account ID accepted from client. Remote discovery owned separate lane.

- list_registered_spreadsheets() -> RegisteredSpreadsheet[] (maximum 30 active rows)
- register_spreadsheet(input {url, expected_generation}) -> RegisteredSpreadsheet; metadata GET validates spreadsheet. URL alternatively raw Google ID from discovery. Initial configuration_pending, never Online.
- remove_spreadsheet(input Mutation + confirmed + history:'preserve'|'delete_activity') -> void. Local removal only. Approved immutable baselines retained in both modes, explicit UI copy.
- pause_spreadsheet/resume_spreadsheet(input Mutation) -> row. Resume requires valid saved scope and matching approved baseline.
- list_spreadsheet_tabs(input {spreadsheet_id}) -> row (refreshes metadata/cache and access diagnostics)
- update_integrity_rules(input Mutation + scope:MonitorScope) -> row. Full scope includes stable sheetId selections. update_monitored_tabs can use this operation rather than duplicate command.
- preview_baseline(input Mutation) -> {revision,preview:CapturePreview}. Captures current saved scope; checks registry revision again after capture.
- approve_baseline(input Mutation + snapshot_id + snapshot_fingerprint + expected_version + confirmed) -> BaselineVersion. Registry revision AND hash pinned through approval.
- run_spreadsheet_check(input Mutation) -> ValidationResult. Checks enabled state, scope/baseline match; captures+evaluates under demand only.
- list_baseline_versions(input {spreadsheet_id,offset,limit}) -> page summaries across scope versions.
- compare_baseline_versions(input {spreadsheet_id,before_id,after_id}) -> backend-only comparison (rule/snapshot differences and report when comparable).
- get_spreadsheet_diagnostics(input {spreadsheet_id}) -> {spreadsheet,report,baseline,scripts}
- list_change_events(input EventFilter) -> Page<Activity>. SQL account-scoped filters + count + limit/offset, preserved removed rows included.
- list_associated_scripts(input {spreadsheet_id}) -> ScriptSummary[]; association refresh through existing integration_test_script result then scope save.
- get_dashboard_summary() -> totals {registered,online,offline,indeterminate,paused,pending,not_checked}, rows/recent_events/recent_failures; zero demo data.
- get_management_preferences()/save_management_preferences(input {expected_generation,preferences}) -> Preferences. interval60..300, notifications preference false/default reservedSprint05, history1..3650, theme system/light/dark, start_with_windows/run_minimized false untilfuture module.

Mutation = {spreadsheet_id,expected_generation,expected_revision}. Registry row: spreadsheet_id,name,url,account_email,paused,removed,status(configuration_pending/not_checked/online/offline/indeterminate/paused), authorization(unknown/available/error), access_error,last_checked_at,baseline_version,scope,scope_hash,revision,tabs. Rule updates invalidate readiness for unmatched scope; historical report not reused as current. All local mutation transactions CAS registry revision. Preview approval guards both revision and engine baseline CAS, no arbitrary frontend computation.
