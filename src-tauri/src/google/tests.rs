use super::*;
use serde_json::json;
use std::{cell::RefCell, collections::VecDeque};
#[derive(Default)]
struct Mock {
    replies: RefCell<VecDeque<Result<HttpReply, ApiError>>>,
    urls: RefCell<Vec<Url>>,
    waits: RefCell<Vec<Duration>>,
}
impl ReadTransport for Mock {
    fn get(
        &self,
        url: &Url,
        authorization: &HeaderValue,
        _: Duration,
    ) -> Result<HttpReply, ApiError> {
        assert!(authorization.is_sensitive());
        self.urls.borrow_mut().push(url.clone());
        self.replies
            .borrow_mut()
            .pop_front()
            .expect("unexpected request")
    }
    fn wait(&self, d: Duration) {
        self.waits.borrow_mut().push(d);
    }
    fn jitter(&self) -> Duration {
        Duration::ZERO
    }
}
fn response(status: u16, body: serde_json::Value) -> Result<HttpReply, ApiError> {
    Ok(HttpReply {
        status,
        body: serde_json::to_vec(&body).unwrap(),
        retry_after: None,
    })
}
fn api(replies: Vec<Result<HttpReply, ApiError>>) -> GoogleApi<Mock> {
    GoogleApi::with_transport(
        "synthetic-access".into(),
        Mock {
            replies: RefCell::new(replies.into()),
            ..Default::default()
        },
    )
    .unwrap()
}
fn metadata() -> serde_json::Value {
    json!({"spreadsheetId":"sheet_1","properties":{"title":"Fixture","locale":"pt_BR","timeZone":"America/Sao_Paulo"},"sheets":[{"properties":{"sheetId":0,"title":"Dados","index":0,"gridProperties":{"rowCount":100,"columnCount":12}}},{"properties":{"sheetId":9,"title":"Resumo","index":1,"hidden":true}}]})
}
#[test]
fn strict_url_and_id_extraction() {
    for input in [
        "sheet_1",
        "https://docs.google.com/spreadsheets/d/sheet_1/edit#gid=9",
        "https://docs.google.com/spreadsheets/u/0/d/sheet_1/edit?usp=sharing",
    ] {
        assert_eq!(parse_spreadsheet_id(input).unwrap(), "sheet_1");
    }
    for input in [
        "",
        "http://docs.google.com/spreadsheets/d/a/edit",
        "https://docs.google.com.evil.invalid/spreadsheets/d/a/edit",
        "https://evil@docs.google.com/spreadsheets/d/a",
        "https://docs.google.com/spreadsheets/d/a/../b",
        "https://docs.google.com/spreadsheets/d/%61",
        "https://docs.google.com/spreadsheets/d/a/export",
        "https://127.0.0.1/a",
        "https://docs.google.com/spreadsheets/d/a\\b",
    ] {
        assert!(parse_spreadsheet_id(input).is_err(), "{input}");
    }
}
#[test]
fn metadata_preserves_all_tabs_and_rejects_wrong_or_missing_data() {
    let a = api(vec![response(200, metadata())]);
    let result = a.sheet_metadata("sheet_1").unwrap();
    assert_eq!(result.sheets.len(), 2);
    assert!(result.sheets[1].hidden);
    assert_eq!(result.sheets[0].row_count, Some(100));
    assert!(a.transport.urls.borrow()[0]
        .query()
        .unwrap()
        .contains("includeGridData=false"));
    for body in [
        json!({}),
        json!({"spreadsheetId":"wrong","properties":{"title":"Fixture"},"sheets":[]}),
    ] {
        assert_eq!(
            api(vec![response(200, body)])
                .sheet_metadata("sheet_1")
                .unwrap_err(),
            ApiError::InvalidResponse
        );
    }
}
#[test]
fn batch_get_calculated_formulas_and_empty_ranges_are_distinct() {
    let values = json!({"spreadsheetId":"sheet_1","valueRanges":[{"range":"Dados!A1:B2","values":[["Total",2]]},{"range":"Dados!C1:C2"}]});
    let formulas = json!({"spreadsheetId":"sheet_1","valueRanges":[{"range":"Dados!A1:B2","values":[["Total","=1+1"]]},{"range":"Dados!C1:C2"}]});
    let a = api(vec![response(200, values), response(200, formulas)]);
    let result = a
        .sheet_values("sheet_1", &["Dados!A1:B2".into(), "Dados!C1:C2".into()])
        .unwrap();
    assert_eq!(result.values[0].values[0][1], 2);
    assert_eq!(result.formulas[0].values[0][1], "=1+1");
    assert!(result.values[1].values.is_empty());
    let urls = a.transport.urls.borrow();
    assert_eq!(urls.len(), 2);
    assert!(urls[0].query().unwrap().contains("UNFORMATTED_VALUE"));
    assert!(urls[1].query().unwrap().contains("FORMULA"));
    assert_eq!(
        urls[0].query_pairs().filter(|(k, _)| k == "ranges").count(),
        2
    );
}
#[test]
fn invalid_ranges_or_non_scalar_cells_fail() {
    assert!(api(vec![]).sheet_values("sheet_1", &[]).is_err());
    let a = api(vec![response(
        200,
        json!({"spreadsheetId":"sheet_1","valueRanges":[{"range":"A1","values":[[{"bad":true}]]}]}),
    )]);
    assert!(matches!(
        a.sheet_values("sheet_1", &["A1".into()]),
        Err(ApiError::InvalidResponse)
    ));
}
#[test]
fn drive_removed_metadata_and_invalid_response() {
    let a = api(vec![response(
        200,
        json!({"id":"sheet_1","name":"Fixture","mimeType":"application/vnd.google-apps.spreadsheet","trashed":true,"modifiedTime":"2026-11-02T00:00:00Z","version":"42"}),
    )]);
    let result = a.drive_metadata("sheet_1").unwrap();
    assert!(result.trashed);
    assert_eq!(result.version.as_deref(), Some("42"));
    assert_eq!(
        api(vec![response(
            200,
            json!({"id":"other","name":"Fixture","mimeType":"x"})
        )])
        .drive_metadata("sheet_1")
        .unwrap_err(),
        ApiError::InvalidResponse
    );
}
fn project(files: serde_json::Value) -> ScriptProject {
    api(vec![
        response(
            200,
            json!({"scriptId":"script_1","title":"Fixture","parentId":"different_sheet"}),
        ),
        response(200, json!({"scriptId":"script_1","files":files})),
    ])
    .script_project("script_1")
    .unwrap()
}
#[test]
fn apps_script_hash_is_order_stable_but_changes_with_source() {
    let gs = json!({"name":"Code","type":"SERVER_JS","source":"function test() { return 1; }"});
    let html = json!({"name":"Index","type":"HTML","source":"<b>Fixture</b>"});
    let manifest = json!({"name":"appsscript","type":"JSON","source":"{}"});
    let a = project(json!([gs.clone(), html.clone(), manifest.clone()]));
    let b = project(json!([manifest, html, gs]));
    assert_eq!(a.source_hash, b.source_hash);
    assert_eq!(a.files.len(), 3);
    assert_eq!(a.metadata.parent_id.as_deref(), Some("different_sheet"));
    assert_ne!(
        a.source_hash,
        project(json!([{ "name":"Code","type":"SERVER_JS","source":"changed"}])).source_hash
    );
    assert!(!format!("{:?}", a.files).contains("return 1"));
}
#[test]
fn apps_script_rejects_unknown_type_and_missing_source() {
    for file in [
        json!({"name":"bad","type":"EXECUTABLE","source":"x"}),
        json!({"name":"bad","type":"SERVER_JS"}),
    ] {
        let a = api(vec![
            response(200, json!({"scriptId":"script_1","title":"Fixture"})),
            response(200, json!({"scriptId":"script_1","files":[file]})),
        ]);
        assert!(matches!(
            a.script_project("script_1"),
            Err(ApiError::InvalidResponse)
        ));
    }
}
#[test]
fn errors_are_closed_and_only_safe_transients_retry() {
    for (status, expected, attempts) in [
        (400, ApiError::BadRequest, 1),
        (401, ApiError::Unauthorized, 1),
        (403, ApiError::Forbidden, 1),
        (404, ApiError::NotFound, 1),
        (429, ApiError::RateLimited, 3),
        (503, ApiError::Server { status: 503 }, 3),
    ] {
        let a = api((0..attempts)
            .map(|_| {
                response(
                    status,
                    json!({"error":{"message":"SENSITIVE_PROVIDER_BODY"}}),
                )
            })
            .collect());
        let err = a.sheet_metadata("sheet_1").unwrap_err();
        assert_eq!(err, expected);
        assert!(!err.to_string().contains("SENSITIVE"));
        assert_eq!(a.transport.urls.borrow().len(), attempts);
        assert_eq!(a.transport.waits.borrow().len(), attempts - 1);
    }
}
#[test]
fn retry_network_timeout_rate_limit_then_success() {
    for error in [ApiError::Network, ApiError::Timeout] {
        let a = api(vec![Err(error), response(200, metadata())]);
        assert!(a.sheet_metadata("sheet_1").is_ok());
        assert_eq!(a.transport.urls.borrow().len(), 2);
    }
    let a = api(vec![
        response(
            403,
            json!({"error":{"errors":[{"reason":"rateLimitExceeded"}]}}),
        ),
        response(200, metadata()),
    ]);
    assert!(a.sheet_metadata("sheet_1").is_ok());
}
#[test]
fn malformed_json_oversize_redirect_and_expired_deadline_fail() {
    let a = api(vec![Ok(HttpReply {
        status: 200,
        body: b"not json".to_vec(),
        retry_after: None,
    })]);
    assert_eq!(
        a.sheet_metadata("sheet_1").unwrap_err(),
        ApiError::InvalidResponse
    );
    let a = api(vec![Ok(HttpReply {
        status: 200,
        body: vec![b' '; MAX_RESPONSE_BYTES as usize + 1],
        retry_after: None,
    })]);
    assert_eq!(
        a.sheet_metadata("sheet_1").unwrap_err(),
        ApiError::ResponseTooLarge
    );
    assert_eq!(
        api(vec![response(302, json!({}))])
            .sheet_metadata("sheet_1")
            .unwrap_err(),
        ApiError::InvalidResponse
    );
    assert_eq!(
        transport::get_bounded(
            &Mock::default(),
            &Url::parse("https://sheets.googleapis.com/v4/spreadsheets/sheet_1").unwrap(),
            &HeaderValue::from_static("Bearer synthetic"),
            Instant::now() - Duration::from_secs(1)
        ),
        Err(ApiError::Timeout)
    );
}
