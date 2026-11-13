//! Centralized event normalization, pt-BR descriptions, local timestamp formatting,
//! and streaming export helpers shared by notifications, history, and exports.

use chrono::{DateTime, Local, TimeZone};
use std::io::{self, Write};

pub fn format_local_datetime(value: &str) -> String {
    let trimmed = value.trim();
    if let Ok(parsed) = DateTime::parse_from_rfc3339(trimmed) {
        return parsed
            .with_timezone(&Local)
            .format("%d/%m/%Y %H:%M:%S")
            .to_string();
    }
    if let Ok(ms) = trimmed.parse::<i64>() {
        if let Some(formatted) = try_format_local_millis(ms) {
            return formatted;
        }
    }
    trimmed
        .chars()
        .filter(|c| !c.is_control())
        .take(32)
        .collect()
}

pub fn format_local_millis(ms: i64) -> String {
    try_format_local_millis(ms).unwrap_or_else(|| ms.to_string())
}

fn try_format_local_millis(ms: i64) -> Option<String> {
    Local
        .timestamp_millis_opt(ms)
        .single()
        .map(|dt| dt.format("%d/%m/%Y %H:%M:%S").to_string())
}

pub fn infer_kind(kind: Option<&str>, raw_diagnosis: &str) -> &'static str {
    let (base, _) = split_count(raw_diagnosis.trim());
    match kind.map(str::trim).filter(|k| !k.is_empty()) {
        Some("recovered") => "recovered",
        Some("analysis_concluded") => "analysis_concluded",
        Some("violation") => "violation",
        Some("analysis_inconclusive") => "analysis_inconclusive",
        Some("unavailable") => "unavailable",
        Some("value_changed") => "value_changed",
        Some("record_added") => "record_added",
        Some("record_removed") => "record_removed",
        Some("record_order_changed") => "record_order_changed",
        Some("content_changed_unspecified") => "content_changed_unspecified",
        Some("header_changed") => "header_changed",
        Some("sheet_title_changed") => "sheet_title_changed",
        Some("sheet_position_changed") => "sheet_position_changed",
        Some("region_changed") => "region_changed",
        Some("named_range_changed") => "named_range_changed",
        Some("script_file_added") => "script_file_added",
        Some("script_file_removed") => "script_file_removed",
        Some("script_file_changed") => "script_file_changed",
        Some("script_code_changed") => "script_code_changed",
        Some(_) => "other",
        None => match base {
            "critical_rule_recovered" | "operational_recovered" | "recovered" => "recovered",
            "analysis_concluded" => "analysis_concluded",
            "value_changed" => "value_changed",
            "record_added" => "record_added",
            "record_removed" => "record_removed",
            "record_order_changed" => "record_order_changed",
            "content_changed_unspecified" | "content_changed_operation_ambiguous" => {
                "content_changed_unspecified"
            }
            "header_changed" => "header_changed",
            "sheet_title_changed" => "sheet_title_changed",
            "sheet_position_changed" => "sheet_position_changed",
            "region_changed" => "region_changed",
            "script_file_added" => "script_file_added",
            "script_file_removed" => "script_file_removed",
            "script_file_changed" => "script_file_changed",
            "script_code_changed" => "script_code_changed",
            _ => "",
        },
    }
}

/// Returns `(event_type, tone)`:
/// - `event_type`: `"failure" | "modification" | "resolution"`
/// - `tone`: `"failure" | "resolved" | "neutral"`
pub fn classify_event(
    source: &str,
    kind: Option<&str>,
    severity: &str,
    raw_diagnosis: &str,
) -> (&'static str, &'static str) {
    let resolved_kind = infer_kind(kind, raw_diagnosis);
    if source == "operational" {
        return if resolved_kind == "recovered" || severity == "info" {
            ("resolution", "resolved")
        } else {
            ("failure", "failure")
        };
    }
    match resolved_kind {
        "recovered" | "analysis_concluded" => ("resolution", "resolved"),
        "violation" | "analysis_inconclusive" | "unavailable" => ("failure", "failure"),
        _ if matches!(severity, "critical" | "warning") => ("failure", "failure"),
        _ => ("modification", "neutral"),
    }
}

pub fn event_type_pt_br(event_type: &str) -> &'static str {
    match event_type {
        "failure" => "Falha",
        "resolution" => "Resolução",
        _ => "Modificação",
    }
}

pub fn issue_status_pt_br(issue_status: Option<&str>) -> &'static str {
    match issue_status {
        Some("active") => "Ativo",
        Some("resolved") => "Resolvido",
        _ => "Registrado",
    }
}

pub fn notification_event_label(
    source: &str,
    kind: Option<&str>,
    severity: &str,
    raw_diagnosis: &str,
) -> &'static str {
    let resolved_kind = infer_kind(kind, raw_diagnosis);
    if source == "operational" {
        return if resolved_kind == "recovered" || severity == "info" {
            "Monitoramento restabelecido"
        } else {
            "Monitoramento indisponível"
        };
    }
    match resolved_kind {
        "recovered" => "Estrutura normalizada",
        "analysis_concluded" => "Verificação normalizada",
        "analysis_inconclusive" => "Verificação inconclusiva",
        "violation" if severity == "warning" => "Alerta estrutural detectado",
        "violation" => "Alteração estrutural detectada",
        "script_file_added"
        | "script_file_removed"
        | "script_file_changed"
        | "script_code_changed" => "Projeto Apps Script modificado",
        _ if severity == "critical" => "Alteração estrutural detectada",
        _ if severity == "warning" => "Alerta estrutural detectado",
        _ => "Modificação registrada",
    }
}

pub fn notification_title(
    source: &str,
    kind: Option<&str>,
    severity: &str,
    raw_diagnosis: &str,
) -> String {
    format!(
        "Sheets Monitor — {}",
        notification_event_label(source, kind, severity, raw_diagnosis)
    )
}

fn split_count(code: &str) -> (&str, u32) {
    if let Some((base, count_raw)) = code.split_once(":count=") {
        if let Ok(parsed) = count_raw.parse::<u32>() {
            return (base, parsed.max(1));
        }
    }
    (code, 1)
}

pub fn diagnosis_pt_br(code: &str, kind: Option<&str>) -> String {
    let trimmed = code.trim();
    if let Some(detail) = trimmed.strip_prefix("rate_limited|") {
        let detail: String = detail
            .chars()
            .filter(|ch| !ch.is_control())
            .take(512)
            .collect();
        return format!(
            "{} Detalhes: {detail}",
            diagnosis_pt_br("rate_limited", kind)
        );
    }
    if trimmed.is_empty() {
        return match infer_kind(kind, "") {
            "recovered" => {
                "A estrutura protegida voltou a corresponder à referência aprovada.".into()
            }
            "analysis_concluded" => {
                "Leitura completa restabelecida; pendência inconclusiva encerrada.".into()
            }
            "violation" => "Alteração estrutural detectada em regra protegida.".into(),
            "analysis_inconclusive" => {
                "Verificação inconclusiva; uma nova tentativa será realizada.".into()
            }
            _ => "Ocorrência registrada no monitoramento.".into(),
        };
    }
    // Preserve already-localized human sentences (e.g. operational or test messages).
    if trimmed.contains(' ') {
        return trimmed.chars().filter(|c| !c.is_control()).collect();
    }
    let (base, count) = split_count(trimmed);
    if matches!(kind, Some("recovered")) && base != "operational_recovered" {
        return "A estrutura protegida voltou a corresponder à referência aprovada.".into();
    }
    if matches!(kind, Some("analysis_concluded")) {
        return "Leitura completa restabelecida; pendência inconclusiva encerrada.".into();
    }
    match base {
        "critical_rule_recovered" | "recovered" => {
            "A estrutura protegida voltou a corresponder à referência aprovada.".into()
        }
        "analysis_concluded" => {
            "Leitura completa restabelecida; pendência inconclusiva encerrada.".into()
        }
        "operational_recovered" => {
            "Monitoramento restabelecido; novas verificações voltaram a concluir normalmente."
                .into()
        }
        "monitored_sheet_missing" => "Aba monitorada ausente na planilha.".into(),
        "sheet_not_observed" => "Aba monitorada não pôde ser observada nesta leitura.".into(),
        "protected_sheet_renamed" => "Nome de aba protegida foi alterado.".into(),
        "protected_sheet_reordered" => "Posição de aba protegida foi alterada.".into(),
        "region_not_observed" => "Intervalo protegido não pôde ser observado completamente.".into(),
        "protected_region_displaced" => {
            "Intervalo protegido foi deslocado ou redimensionado.".into()
        }
        "column_identity_ambiguous" => {
            "Cabeçalho duplicado impediu identificar coluna protegida.".into()
        }
        "protected_column_missing" => "Coluna obrigatória foi removida da aba.".into(),
        "protected_header_changed_or_column_missing" => {
            "Cabeçalho protegido alterado ou coluna obrigatória ausente.".into()
        }
        "protected_column_reordered" => "Ordem de coluna protegida foi alterada.".into(),
        "named_range_changed" => {
            if matches!(kind, Some("violation")) {
                "Intervalo nomeado protegido foi alterado.".into()
            } else {
                "Intervalo nomeado foi atualizado.".into()
            }
        }
        "named_range_missing" => "Intervalo nomeado protegido foi removido.".into(),
        "named_range_not_observed" => "Intervalo nomeado não pôde ser observado.".into(),
        "baseline_formula_not_observed" => "Fórmula da referência não pôde ser confirmada.".into(),
        "formula_column_or_key_not_resolved" => {
            "Coluna ou chave da fórmula protegida não pôde ser resolvida.".into()
        }
        "formula_anchor_ambiguous" => {
            "Linha de referência da fórmula protegida ficou ambígua.".into()
        }
        "fixed_formula_anchor_changed" => "Chave da linha com fórmula fixa foi alterada.".into(),
        "formula_cell_not_observed" => {
            "Célula de fórmula protegida fora da janela observada.".into()
        }
        "formula_loading" => {
            "Fórmula protegida ainda estava em carregamento no Google Sheets.".into()
        }
        "formula_effective_error" => {
            "Fórmula protegida apresentou erro de cálculo na planilha.".into()
        }
        "formula_error_unknown" => {
            "Fórmula protegida retornou estado de erro não reconhecido.".into()
        }
        "protected_formula_removed_or_replaced" => {
            "Fórmula protegida foi removida ou substituída por valor fixo.".into()
        }
        "protected_record_removed" => "Um registro da referência foi removido, contrariando a regra de preservação de registros.".into(),
        "record_deletion_unproven" => "Não foi possível comprovar a preservação dos registros sem uma chave única estável.".into(),
        "protected_formula_changed" => "Expressão de fórmula protegida foi alterada.".into(),
        "formula_sheet_reference_missing" => "Fórmula protegida referencia aba inexistente.".into(),
        "formula_equivalence_unproven" => {
            "Não foi possível comprovar equivalência da fórmula protegida.".into()
        }
        "metadata_not_observed" => {
            "Metadados estruturais da planilha não puderam ser lidos.".into()
        }
        "capture_incomplete" => "Leitura parcial impediu concluir toda a verificação.".into(),
        "network" | "network_failed" => "Causa: falha de conexão com o Google. Impacto: estado atual não confirmado. Solução: confira a internet e aguarde a próxima tentativa.".into(),
        "timeout" => "Causa: consulta Google excedeu o tempo limite. Impacto: verificação não concluída. Solução: confira a conexão e aguarde a próxima tentativa.".into(),
        "rate_limited" | "quota_exceeded" => "Causa: limite temporário de consultas Google atingido. Impacto: leituras adiadas, sem confirmação do estado atual. Solução: aguarde o prazo de nova tentativa e evite consultas manuais repetidas.".into(),
        "auth_required" | "authentication_required" | "token_expired" | "invalid_grant" | "unauthorized" | "reconnect_required" => "Causa: conta Google desconectada ou autorização expirada. Impacto: leitura suspensa, sem confirmação do estado atual. Solução: conecte ou reautorize a conta em Configurações.".into(),
        "forbidden" | "permission_denied" | "insufficient_scope" => "Causa: permissão de leitura insuficiente. Impacto: recurso não verificado; não comprova violação estrutural. Solução: confira o compartilhamento e reautorize as permissões da conta.".into(),
        "not_found" => "Causa: arquivo não encontrado ou acesso revogado. Impacto: recurso não verificado. Solução: confira o link e o compartilhamento com a conta conectada.".into(),
        "server" | "server_error" | "service_unavailable" => "Causa: serviço Google temporariamente indisponível. Impacto: verificação não concluída. Solução: aguarde a próxima tentativa.".into(),
        "configuration_conflict" | "monitoring_conflict" | "sync_conflict" | "sync_failed" => "Causa: sincronização não confirmou a atualização. Impacto: dados locais podem estar desatualizados. Solução: atualize os dados e revise a versão atual antes de repetir a operação.".into(),
        "script_not_observed" => "Projeto Apps Script protegido não pôde ser lido.".into(),
        "script_analysis_incomplete" | "script_analysis_partial" => {
            "Análise estática do projeto Apps Script ficou parcial.".into()
        }
        "script_dependency_unproven" => {
            "Script utiliza referência dinâmica que não pôde ser comprovada.".into()
        }
        "script_range_invalid" => "Script referencia intervalo numérico inválido.".into(),
        "script_sheet_missing" => "Script referencia aba ausente na planilha.".into(),
        "script_sheet_unproven" => {
            "Não foi possível confirmar aba referenciada pelo script.".into()
        }
        "script_range_outside_grid" => {
            "Script referencia células fora dos limites atuais da aba.".into()
        }
        "script_range_outside_monitored_scope" => {
            "Script acessa aba fora do escopo monitorado.".into()
        }
        "script_baseline_sheet_unobserved" => {
            "Aba utilizada pelo script não constava na referência.".into()
        }
        "script_column_identity_ambiguous" => {
            "Coluna acessada pelo script ficou ambígua por cabeçalho duplicado.".into()
        }
        "script_logical_column_changed" => {
            "Coluna utilizada pelo script mudou de posição ou cabeçalho.".into()
        }
        "script_range_coverage_partial" => {
            "Intervalo acessado pelo script está parcialmente fora da área monitorada.".into()
        }
        "value_changed" => {
            if count > 1 {
                format!("{count} células tiveram valores alterados na mesma verificação.")
            } else {
                "Valor de célula alterado na área monitorada.".into()
            }
        }
        "record_added" => {
            if count > 1 {
                format!("{count} registros adicionados na área monitorada.")
            } else {
                "Novo registro adicionado na área monitorada.".into()
            }
        }
        "record_removed" => {
            if count > 1 {
                format!("{count} registros removidos na área monitorada.")
            } else {
                "Registro removido na área monitorada.".into()
            }
        }
        "record_order_changed" => "Ordem dos registros foi alterada na área monitorada.".into(),
        "content_changed_unspecified" | "content_changed_operation_ambiguous" => {
            if count > 1 {
                format!("{count} blocos de conteúdo foram atualizados na área monitorada.")
            } else {
                "Conteúdo da área monitorada foi atualizado.".into()
            }
        }
        "header_changed" => {
            if count > 1 {
                format!("{count} cabeçalhos não bloqueados foram atualizados.")
            } else {
                "Cabeçalho não bloqueado foi atualizado.".into()
            }
        }
        "sheet_title_changed" => "Título da aba foi atualizado.".into(),
        "sheet_position_changed" => "Ordem da aba foi atualizada.".into(),
        "region_changed" => "Dimensões da área observada foram atualizadas.".into(),
        "script_file_added" => {
            if count > 1 {
                format!("{count} arquivos adicionados ao projeto Apps Script.")
            } else {
                "Arquivo adicionado ao projeto Apps Script.".into()
            }
        }
        "script_file_removed" => {
            if count > 1 {
                format!("{count} arquivos removidos do projeto Apps Script.")
            } else {
                "Arquivo removido do projeto Apps Script.".into()
            }
        }
        "script_file_changed" => {
            if count > 1 {
                format!("{count} arquivos modificados no projeto Apps Script.")
            } else {
                "Arquivo modificado no projeto Apps Script.".into()
            }
        }
        "script_code_changed" => "Código do projeto Apps Script foi atualizado.".into(),
        _ => match infer_kind(kind, base) {
            "recovered" => {
                "A estrutura protegida voltou a corresponder à referência aprovada.".into()
            }
            "analysis_concluded" => {
                "Leitura completa restabelecida; pendência inconclusiva encerrada.".into()
            }
            "violation" => "Alteração estrutural detectada em regra protegida.".into(),
            "analysis_inconclusive" => {
                "Verificação inconclusiva; uma nova tentativa será realizada.".into()
            }
            "unavailable" => {
                "Não foi possível concluir a verificação; nova tentativa será agendada.".into()
            }
            _ => "Modificação registrada durante a verificação.".into(),
        },
    }
}

pub fn normalize_notification_diagnosis(
    source: &str,
    kind: Option<&str>,
    operational_code: Option<&str>,
    raw_diagnosis: &str,
) -> String {
    if source == "operational" {
        if matches!(kind, Some("recovered")) {
            return "Monitoramento restabelecido; alterações intermediárias podem não ter sido observadas.".into();
        }
        if let Some(code) = operational_code.filter(|c| !c.trim().is_empty()) {
            let reason = diagnosis_pt_br(code, Some("unavailable"));
            return format!("{reason} Uma nova tentativa será realizada automaticamente.");
        }
    }
    diagnosis_pt_br(raw_diagnosis, kind)
}

pub fn sanitize_csv_cell(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .map(|c| match c {
            '\r' | '\n' | '\t' => ' ',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.starts_with(['=', '+', '-', '@']) {
        format!("'{trimmed}")
    } else {
        trimmed.to_owned()
    }
}

pub fn write_csv_row(writer: &mut impl Write, cells: &[&str]) -> io::Result<()> {
    for (index, raw) in cells.iter().enumerate() {
        if index > 0 {
            writer.write_all(b";")?;
        }
        let safe = sanitize_csv_cell(raw);
        if safe.contains([';', ',', '"']) {
            writer.write_all(b"\"")?;
            for ch in safe.chars() {
                if ch == '"' {
                    writer.write_all(b"\"\"")?;
                } else {
                    let mut buf = [0u8; 4];
                    writer.write_all(ch.encode_utf8(&mut buf).as_bytes())?;
                }
            }
            writer.write_all(b"\"")?;
        } else {
            writer.write_all(safe.as_bytes())?;
        }
    }
    writer.write_all(b"\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operational_errors_explain_cause_impact_and_solution() {
        for code in [
            "auth_required",
            "network",
            "timeout",
            "forbidden",
            "sync_failed",
            "quota_exceeded",
        ] {
            let text = diagnosis_pt_br(code, None);
            for label in ["Causa:", "Impacto:", "Solução:"] {
                assert!(text.contains(label), "{code}: {text}");
            }
        }
        let detail = diagnosis_pt_br(
            "rate_limited|Sheets API / spreadsheets.get: cota por minuto",
            None,
        );
        assert!(detail.contains("Impacto:"));
        assert!(detail.contains("Sheets API / spreadsheets.get: cota por minuto"));
    }

    #[test]
    fn formats_rfc3339_into_local_dd_mm_yyyy_hh_mm_ss() {
        let formatted = format_local_datetime("2026-11-04T15:30:45Z");
        assert_eq!(formatted.len(), 19);
        assert_eq!(&formatted[2..3], "/");
        assert_eq!(&formatted[5..6], "/");
        assert_eq!(&formatted[10..11], " ");
        assert_eq!(&formatted[13..14], ":");
        assert_eq!(&formatted[16..17], ":");
    }

    #[test]
    fn translates_technical_identifiers_without_leaking_snake_case() {
        for (code, kind) in [
            (
                "protected_header_changed_or_column_missing",
                Some("violation"),
            ),
            ("critical_rule_recovered", Some("recovered")),
            ("monitored_sheet_missing", Some("violation")),
            ("value_changed:count=4", Some("value_changed")),
            ("unknown_future_code_xyz", Some("violation")),
        ] {
            let text = diagnosis_pt_br(code, kind);
            assert!(!text.contains('_'), "leaked snake_case in {text}");
            assert!(!text.is_empty());
        }
        assert!(
            diagnosis_pt_br("value_changed:count=4", Some("value_changed")).contains("4 células")
        );
    }

    #[test]
    fn classifies_failures_resolutions_and_neutral_modifications() {
        assert_eq!(
            classify_event(
                "integrity",
                Some("violation"),
                "critical",
                "protected_column_missing"
            ),
            ("failure", "failure")
        );
        assert_eq!(
            classify_event(
                "integrity",
                Some("recovered"),
                "info",
                "critical_rule_recovered"
            ),
            ("resolution", "resolved")
        );
        assert_eq!(
            classify_event(
                "operational",
                Some("unavailable"),
                "warning",
                "network_failed"
            ),
            ("failure", "failure")
        );
        assert_eq!(
            classify_event(
                "operational",
                Some("recovered"),
                "info",
                "operational_recovered"
            ),
            ("resolution", "resolved")
        );
        assert_eq!(
            classify_event("integrity", Some("value_changed"), "info", "value_changed"),
            ("modification", "neutral")
        );
    }

    #[test]
    fn csv_writer_escapes_delimiters_and_neutralizes_formulas() {
        let mut out = Vec::new();
        out.extend_from_slice("\u{FEFF}".as_bytes());
        write_csv_row(&mut out, &["=SUM(A1:A2)", "Planilha; \"A\"", "Ativo"]).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with('\u{FEFF}'));
        assert!(text.contains("'=SUM(A1:A2);\"Planilha; \"\"A\"\"\";Ativo\r\n"));
    }
}
