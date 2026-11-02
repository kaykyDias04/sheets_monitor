//! Synthetic offline provider. No Google access or implicit baseline approval.
use crate::domain::{Monitor, Snapshot};
use serde_json::json;

pub fn collect(monitor: &Monitor, sequence: u64) -> Snapshot {
    let mut snapshot = Snapshot {
        sheets: json!({"spreadsheetId":monitor.spreadsheet_id,"properties":{"title":monitor.name},"sheets":[{"properties":{"sheetId":0,"title":"Resumo"},"data":[{"rowData":[{"values":[{"userEnteredValue":{"stringValue":"Receita"}},{"userEnteredValue":{"numberValue":1000+sequence},"effectiveFormat":{"numberFormat":{"type":"NUMBER"}}}]}]}]}]}),
        drive: json!({"id":monitor.spreadsheet_id,"name":monitor.name,"mimeType":"application/vnd.google-apps.spreadsheet","description":"FIXTURE LOCAL — dados sintéticos"}),
        permissions: vec![json!({"id":"synthetic-owner","type":"user","role":"owner","emailAddress":"fixture@example.invalid"})],
        ..Default::default()
    };
    for id in &monitor.script_ids {
        snapshot.scripts.insert(id.clone(), json!({"scriptId":id,"files":[{"name":"Code","type":"SERVER_JS","source":format!("function indicador() {{ return {}; }}",sequence)},{"name":"appsscript","type":"JSON","source":"{\"timeZone\":\"America/Sao_Paulo\",\"runtimeVersion\":\"V8\",\"oauthScopes\":[\"https://www.googleapis.com/auth/spreadsheets.readonly\"]}"}]}));
    }
    snapshot.findings = snapshot.scripts.iter().map(|(id, source)| (id.clone(), crate::analysis::analyze_project(source))).collect();
    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixtures_change_cells_and_authorized_script_source() {
        let m = Monitor {id:"m".into(),name:"Teste".into(),spreadsheet_id:"synthetic-sheet".into(),script_ids:vec!["synthetic-script".into()],enabled:true,archived:false,baseline_snapshot_id:None,latest_snapshot_id:None,last_checked_at:None,last_error:None,created_at:String::new()};
        let first = collect(&m, 1);
        let second = collect(&m, 2);
        assert_ne!(first.sheets, second.sheets);
        assert_ne!(first.scripts, second.scripts);
        assert_eq!(first.findings.len(), 1);
    }
}
