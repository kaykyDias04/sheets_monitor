use super::*;
use crate::google::HttpReply;
use reqwest::header::HeaderValue;
use serde_json::{json, Value};
use std::{cell::RefCell, collections::VecDeque, time::Duration};
use url::Url;

#[derive(Default)]
struct Mock {
    responses: RefCell<VecDeque<HttpReply>>,
    urls: RefCell<Vec<Url>>,
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
        Ok(self
            .responses
            .borrow_mut()
            .pop_front()
            .expect("unexpected fixture request"))
    }
    fn wait(&self, _: Duration) {}
    fn jitter(&self) -> Duration {
        Duration::ZERO
    }
}
fn api(responses: Vec<(u16, Value)>) -> GoogleApi<Mock> {
    GoogleApi::with_transport(
        "synthetic-access".into(),
        Mock {
            responses: RefCell::new(
                responses
                    .into_iter()
                    .map(|(status, body)| HttpReply {
                        status,
                        body: serde_json::to_vec(&body).unwrap(),
                        retry_after: None,
                    })
                    .collect(),
            ),
            ..Default::default()
        },
    )
    .unwrap()
}
fn file(id: &str) -> Value {
    json!({"id":id,"name":"Fixture","mimeType":SHEET_MIME,"modifiedTime":"2026-11-02T10:00:00Z"})
}
fn params(a: &GoogleApi<Mock>, index: usize) -> std::collections::BTreeMap<String, String> {
    a.transport.urls.borrow()[index]
        .query_pairs()
        .into_owned()
        .collect()
}

#[test]
fn discovery_queries_metadata_only_and_escapes_search_without_operator_injection() {
    let a = api(vec![(200, json!({"files":[file("sheet_a")]}))]);
    let query = DiscoveryQuery {
        search: "O'Brien\\Docs' or trashed = true".into(),
        order: DiscoveryOrder::ModifiedDesc,
        ..Default::default()
    };
    let page = a.discover_spreadsheets(&query).unwrap();
    assert!(page.complete);
    assert_eq!(page.files.len(), 1);
    let p = params(&a, 0);
    assert_eq!(p["q"],format!("mimeType = '{SHEET_MIME}' and trashed = false and name contains 'O\\'Brien\\\\Docs\\' or trashed = true'"));
    assert_eq!(p["corpora"], "allDrives");
    assert_eq!(p["supportsAllDrives"], "true");
    assert_eq!(p["includeItemsFromAllDrives"], "true");
    assert_eq!(p["spaces"], "drive");
    assert_eq!(p["pageSize"], "50");
    assert_eq!(p["orderBy"], "modifiedTime desc,name_natural");
    assert!(p["fields"].contains("incompleteSearch"));
    assert!(!p.contains_key("alt"));
    assert_eq!(a.transport.urls.borrow()[0].path(), "/drive/v3/files");
}

#[test]
fn discovery_empty_page_with_cursor_and_next_page_over_thirty_are_distinct() {
    let files: Vec<Value> = (0..40).map(|i| file(&format!("sheet_{i}"))).collect();
    let a = api(vec![
        (200, json!({"files":[],"nextPageToken":"opaque-token"})),
        (200, json!({"files":files})),
    ]);
    let first = a.discover_spreadsheets(&DiscoveryQuery::default()).unwrap();
    assert!(!first.complete);
    assert!(first.files.is_empty());
    let second = a
        .discover_spreadsheets(&DiscoveryQuery {
            page_token: first.next_page_token,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(second.files.len(), 40);
    assert!(second.complete);
    assert_eq!(params(&a, 1)["pageToken"], "opaque-token");
}

#[test]
fn discovery_no_results_is_complete_only_when_search_was_complete() {
    for incomplete in [false, true] {
        let a = api(vec![(
            200,
            json!({"files":[],"incompleteSearch":incomplete}),
        )]);
        let p = a.discover_spreadsheets(&DiscoveryQuery::default()).unwrap();
        assert_eq!(p.complete, !incomplete);
        assert_eq!(p.incomplete_search, incomplete);
    }
}

#[test]
fn discovery_shared_files_and_specific_drive_keep_explicit_coverage() {
    let mut shared = file("shared_a");
    shared["shared"] = json!(true);
    let mut drive = file("drive_a");
    drive["driveId"] = json!("drive_fixture");
    drive["resourceKey"] = json!("key_fixture");
    let a = api(vec![(
        200,
        json!({"files":[shared,drive],"incompleteSearch":true}),
    )]);
    let p = a
        .discover_spreadsheets(&DiscoveryQuery {
            drive_id: Some("drive_fixture".into()),
            ..Default::default()
        })
        .unwrap();
    assert!(p.files.iter().all(|f| f.shared));
    assert!(!p.complete);
    assert_eq!(p.corpus, "drive");
    let q = params(&a, 0);
    assert_eq!(q["corpora"], "drive");
    assert_eq!(q["driveId"], "drive_fixture");
}

#[test]
fn discovery_invalid_queries_fail_before_network_and_order_is_enumerated() {
    let a = api(vec![]);
    for query in [
        DiscoveryQuery {
            page_size: 0,
            ..Default::default()
        },
        DiscoveryQuery {
            page_size: 101,
            ..Default::default()
        },
        DiscoveryQuery {
            search: "a".repeat(257),
            ..Default::default()
        },
        DiscoveryQuery {
            search: "line\nbreak".into(),
            ..Default::default()
        },
        DiscoveryQuery {
            page_token: Some(String::new()),
            ..Default::default()
        },
        DiscoveryQuery {
            page_token: Some("a".repeat(4097)),
            ..Default::default()
        },
        DiscoveryQuery {
            drive_id: Some("../drive".into()),
            ..Default::default()
        },
    ] {
        assert_eq!(
            a.discover_spreadsheets(&query).unwrap_err(),
            ApiError::InvalidInput
        );
    }
    assert!(a.transport.urls.borrow().is_empty());
    assert!(serde_json::from_value::<DiscoveryQuery>(json!({"order":"name,evil"})).is_err());
    let default: DiscoveryQuery = serde_json::from_value(json!({})).unwrap();
    assert_eq!(default.page_size, 50);
}

#[test]
fn discovery_rejects_wrong_mime_trashed_duplicate_ids_and_malformed_metadata() {
    let mut wrong = file("sheet_a");
    wrong["mimeType"] = json!("text/plain");
    let mut trashed = file("sheet_a");
    trashed["trashed"] = json!(true);
    let mut invalid = file("sheet_a");
    invalid["id"] = json!("bad/id");
    let mut time = file("sheet_a");
    time["modifiedTime"] = json!("not-a-time");
    for body in [
        json!({}),
        json!({"files":[wrong]}),
        json!({"files":[trashed]}),
        json!({"files":[invalid]}),
        json!({"files":[time]}),
        json!({"files":[file("same"),file("same")]}),
        json!({"files":[],"nextPageToken":""}),
    ] {
        assert_eq!(
            api(vec![(200, body)])
                .discover_spreadsheets(&DiscoveryQuery::default())
                .unwrap_err(),
            ApiError::InvalidResponse
        );
    }
}

#[test]
fn discovery_rejects_repeating_cursor_and_excess_page_size_from_provider() {
    assert_eq!(
        api(vec![(200, json!({"files":[],"nextPageToken":"same"}))])
            .discover_spreadsheets(&DiscoveryQuery {
                page_token: Some("same".into()),
                ..Default::default()
            })
            .unwrap_err(),
        ApiError::InvalidResponse
    );
    assert_eq!(
        api(vec![(200, json!({"files":[file("a"),file("b")]}))])
            .discover_spreadsheets(&DiscoveryQuery {
                page_size: 1,
                ..Default::default()
            })
            .unwrap_err(),
        ApiError::InvalidResponse
    );
}

#[test]
fn discovery_authorization_failures_are_errors_not_empty_results() {
    for (status, error) in [
        (401, ApiError::Unauthorized),
        (403, ApiError::Forbidden),
        (400, ApiError::BadRequest),
    ] {
        let a = api(vec![(
            status,
            json!({"error":{"message":"sensitive provider message"}}),
        )]);
        let e = a
            .discover_spreadsheets(&DiscoveryQuery::default())
            .unwrap_err();
        assert_eq!(e, error);
        assert!(!e.to_string().contains("sensitive"));
    }
}
