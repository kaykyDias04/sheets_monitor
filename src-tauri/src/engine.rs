use crate::domain::{Difference, DifferenceKind};
use serde_json::Value;
use std::collections::BTreeSet;

/// Lossless structural comparison. No value, formula, formatting or metadata
/// is silently filtered. Arrays preserve API order; every index is compared.
pub fn diff(before: &Value, after: &Value) -> Vec<Difference> {
    let mut result = Vec::new();
    compare("", Some(before), Some(after), &mut result);
    result
}

fn escape(key: &str) -> String { key.replace('~', "~0").replace('/', "~1") }

fn compare(path: &str, before: Option<&Value>, after: Option<&Value>, out: &mut Vec<Difference>) {
    if before == after { return; }
    match (before, after) {
        (Some(Value::Object(a)), Some(Value::Object(b))) => {
            for key in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
                compare(&format!("{path}/{}", escape(key)), a.get(key), b.get(key), out);
            }
        }
        (Some(Value::Array(a)), Some(Value::Array(b))) => {
            for i in 0..a.len().max(b.len()) {
                compare(&format!("{path}/{i}"), a.get(i), b.get(i), out);
            }
        }
        // Addition/removal carries the entire original subtree, so no data is lost.
        _ => out.push(Difference {
            path: path.into(),
            kind: match (before, after) {
                (None, _) => DifferenceKind::Added,
                (_, None) => DifferenceKind::Removed,
                _ => DifferenceKind::Changed,
            },
            before: before.cloned(), after: after.cloned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_cells_formulas_formats_permissions_scripts_and_escaped_paths() {
        let a = json!({"sheets":{"cells":[{"formula":"=1","format":{"bold":false}}]},"permissions":[{"role":"reader"}],"scripts":{"x":"old"},"a/b~":null});
        let b = json!({"sheets":{"cells":[{"formula":"=2","format":{"bold":true}}]},"permissions":[{"role":"writer"}],"scripts":{"x":"new"},"a/b~":1});
        let changes = diff(&a,&b);
        assert_eq!(changes.len(),5);
        assert!(changes.iter().any(|d| d.path=="/a~1b~0"));
    }

    #[test]
    fn distinguishes_absent_and_null_and_keeps_removed_subtrees() {
        let changes = diff(&json!({"removed":{"all":[1,2]}}),&json!({"new":null}));
        assert_eq!(changes.len(),2);
        assert_eq!(changes[0].after,Some(Value::Null));
        assert_eq!(changes[1].before,Some(json!({"all":[1,2]})));
    }

    #[test]
    fn ignores_object_key_order_but_preserves_array_order() {
        assert!(diff(&json!({"x":1,"y":2}),&json!({"y":2,"x":1})).is_empty());
        assert_eq!(diff(&json!([1,2,3]),&json!([2,1])).len(),3);
    }
}
