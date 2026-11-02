use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Finding {
    pub rule: String,
    pub file: String,
    pub line: usize,
    pub severity: String,
    pub message: String,
}

/// Heuristic static indicators only. Never executes, compiles or sends source.
/// Matches inside strings/comments remain indicators; absence is not a safety claim.
pub fn analyze_project(project: &Value) -> Vec<Finding> {
    let mut result = Vec::new();
    if let Some(files) = project.get("files").and_then(Value::as_array) {
        for file in files {
            let name = file.get("name").and_then(Value::as_str).unwrap_or("unnamed");
            let source = file.get("source").and_then(Value::as_str).unwrap_or("");
            for (line, text) in source.lines().enumerate() {
                for (pattern, rule, severity, message) in [
                    ("UrlFetchApp", "external_http", "review", "Indicador de requisição HTTP externa; revisar destinos e dados enviados."),
                    ("eval(", "dynamic_execution", "review", "Indicador de execução dinâmica; análise estática não determina o código executado."),
                    ("new Function", "dynamic_function", "review", "Indicador de geração dinâmica de função."),
                    ("deleteFile", "file_deletion", "review", "Indicador de operação de remoção de arquivo."),
                    ("setTrashed", "file_trash", "review", "Indicador de movimentação de arquivo para lixeira."),
                    ("MailApp", "email", "info", "Indicador de acesso a envio de e-mail."),
                    ("GmailApp", "gmail", "review", "Indicador de acesso ao Gmail."),
                    ("ScriptApp.newTrigger", "trigger_creation", "info", "Indicador de criação de gatilho."),
                    ("setValue", "spreadsheet_write", "info", "Indicador de escrita em planilha."),
                    ("clear(", "spreadsheet_clear", "review", "Indicador de limpeza de conteúdo."),
                ] {
                    if text.contains(pattern) {
                        result.push(Finding { rule:rule.into(), file:name.into(), line:line+1,
                            severity:severity.into(), message:message.into() });
                    }
                }
            }
            if file.get("type").and_then(Value::as_str)==Some("JSON") {
                match serde_json::from_str::<Value>(source) {
                    Ok(manifest) => {
                        if let Some(scopes) = manifest.get("oauthScopes").and_then(Value::as_array) {
                            for scope in scopes {
                                result.push(Finding {rule:"manifest_scope".into(),file:name.into(),line:1,severity:"info".into(),
                                    message:format!("Escopo declarado: {}", scope.as_str().unwrap_or("inválido"))});
                            }
                        }
                        if manifest.get("webapp").is_some() {
                            result.push(Finding {rule:"webapp_manifest".into(),file:name.into(),line:1,severity:"review".into(),message:"Manifest contém configuração de aplicativo web; revisar acesso e identidade de execução.".into()});
                        }
                    }
                    Err(_) => result.push(Finding {rule:"invalid_manifest".into(),file:name.into(),line:1,severity:"review".into(),message:"Manifest JSON não pôde ser analisado.".into()}),
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn reviews_source_and_manifest_without_execution() {
        let findings=analyze_project(&json!({"files":[{"name":"Code","type":"SERVER_JS","source":"UrlFetchApp.fetch('https://example.com');\n eval('1')"},{"name":"appsscript","type":"JSON","source":"{\"oauthScopes\":[\"scope\"],\"webapp\":{}}"}]}));
        assert_eq!(findings.len(),4);
        assert!(findings.iter().any(|f|f.rule=="dynamic_execution"&&f.line==2));
    }
    #[test]
    fn reports_invalid_manifest_and_missing_files_is_empty() {
        assert!(analyze_project(&json!({})).is_empty());
        assert_eq!(analyze_project(&json!({"files":[{"name":"appsscript","type":"JSON","source":"{"}]}))[0].rule,"invalid_manifest");
    }
}
