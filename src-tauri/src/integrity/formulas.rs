//! Conservative formula comparison. No evaluation or general algebraic equivalence.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FormulaComparison {
    Equivalent,
    Changed,
    Inconclusive,
    BrokenReference,
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Literal(String),
    Reference(Reference),
    Symbol(char),
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Reference {
    sheet: Option<String>,
    row: u32,
    column: u32,
    absolute_row: bool,
    absolute_column: bool,
}

/// Proves only unchanged syntax or identical supported tokens after resolving
/// target sheets, logical columns and uniquely keyed records. Different reference
/// targets are inconclusive because Sheets can adjust references on row edits.
pub(crate) struct FormulaContext<'a> {
    pub sheet_id: u32,
    pub before: (u32, u32),
    pub after: (u32, u32),
    pub mode: FormulaMode,
    pub allow_shift: bool,
    pub baseline: &'a StructuralSnapshot,
    pub current: &'a StructuralSnapshot,
}
pub(crate) fn compare_formula(
    expected: &str,
    found: &str,
    context: FormulaContext<'_>,
) -> FormulaComparison {
    let FormulaContext {
        sheet_id,
        before,
        after,
        mode,
        allow_shift,
        baseline,
        current,
    } = context;
    if before != after && !allow_shift {
        return FormulaComparison::Changed;
    }
    let Some(old) = tokenize(expected, mode) else {
        return FormulaComparison::Inconclusive;
    };
    let Some(new) = tokenize(found, mode) else {
        return FormulaComparison::Inconclusive;
    };
    if references_missing(&new, sheet_id, current) {
        return FormulaComparison::BrokenReference;
    }
    if matches!(mode, FormulaMode::Exact) && before == after && expected == found {
        return FormulaComparison::Equivalent;
    }
    if matches!(mode, FormulaMode::Structural) {
        if let (Some(a), Some(b)) = (
            affine(&old, sheet_id, baseline),
            affine(&new, sheet_id, current),
        ) {
            if a.0 == b.0 {
                return if a.1 == b.1 && a.2 == b.2 {
                    FormulaComparison::Equivalent
                } else {
                    FormulaComparison::Changed
                };
            }
        }
    }
    if old.len() != new.len() {
        return FormulaComparison::Inconclusive;
    }
    let mut reference_uncertain = false;
    let mut syntax_changed = false;
    for (a, b) in old.iter().zip(&new) {
        match (a, b) {
            (Token::Reference(a), Token::Reference(b)) => {
                if a.absolute_row != b.absolute_row || a.absolute_column != b.absolute_column {
                    reference_uncertain = true;
                    continue;
                }
                let left = target_identity(a, sheet_id, baseline);
                let right = target_identity(b, sheet_id, current);
                match (left, right) {
                    (Some(a), Some(b)) if a == b => {}
                    _ if before == after
                        && a.row == b.row
                        && a.column == b.column
                        && sheet_for(a, sheet_id, baseline).map(|s| s.sheet_id)
                            == sheet_for(b, sheet_id, current).map(|s| s.sheet_id) => {}
                    _ => reference_uncertain = true,
                }
            }
            (Token::Literal(a), Token::Literal(b)) if a == b => {}
            (Token::Symbol(a), Token::Symbol(b)) if a == b => {}
            // Exact mode protects literal syntax; structural mode does not
            // infer inequivalence from an unsupported algebraic rewrite.
            (Token::Literal(_), Token::Literal(_)) | (Token::Symbol(_), Token::Symbol(_)) => {
                syntax_changed = true
            }
            _ => return FormulaComparison::Inconclusive,
        }
    }
    if reference_uncertain {
        FormulaComparison::Inconclusive
    } else if syntax_changed {
        if matches!(mode, FormulaMode::Exact) {
            FormulaComparison::Changed
        } else {
            FormulaComparison::Inconclusive
        }
    } else {
        FormulaComparison::Equivalent
    }
}
// A tiny, explicitly bounded symbolic proof: one numeric variable and one
// integer constant under +, -, or *. It deliberately excludes division,
// functions, ranges and multiple variables. No floating-point arithmetic.
fn affine(tokens: &[Token], sheet_id: u32, s: &StructuralSnapshot) -> Option<(String, i128, i128)> {
    let constant = |token: &Token| {
        if let Token::Literal(raw) = token {
            raw.parse::<i128>()
                .ok()
                .filter(|n| n.abs() <= 1_000_000_000)
        } else {
            None
        }
    };
    match tokens {
        [Token::Reference(reference), Token::Symbol(operator), number] => {
            let n = constant(number)?;
            let variable = affine_variable(reference, sheet_id, s)?;
            match operator {
                '*' => Some((variable, n, 0)),
                '+' => Some((variable, 1, n)),
                '-' => Some((variable, 1, -n)),
                _ => None,
            }
        }
        [number, Token::Symbol(operator), Token::Reference(reference)] => {
            let n = constant(number)?;
            let variable = affine_variable(reference, sheet_id, s)?;
            match operator {
                '*' => Some((variable, n, 0)),
                '+' => Some((variable, 1, n)),
                '-' => Some((variable, -1, n)),
                _ => None,
            }
        }
        _ => None,
    }
}

fn affine_variable(reference: &Reference, sheet_id: u32, s: &StructuralSnapshot) -> Option<String> {
    Some(format!(
        "{}:abs_row={}:abs_col={}",
        target_identity(reference, sheet_id, s)?,
        reference.absolute_row,
        reference.absolute_column
    ))
}

fn references_missing(tokens: &[Token], sheet_id: u32, snapshot: &StructuralSnapshot) -> bool {
    snapshot.coverage.metadata_complete
        && tokens.iter().any(|t| match t {
            Token::Reference(r) => sheet_for(r, sheet_id, snapshot).is_none(),
            _ => false,
        })
}
fn sheet_for<'a>(
    r: &Reference,
    sheet_id: u32,
    s: &'a StructuralSnapshot,
) -> Option<&'a SheetSnapshot> {
    if let Some(name) = &r.sheet {
        s.sheets.iter().find(|tab| &tab.title == name)
    } else {
        s.sheets.iter().find(|tab| tab.sheet_id == sheet_id)
    }
}
fn target_identity(r: &Reference, sheet_id: u32, s: &StructuralSnapshot) -> Option<String> {
    let sheet = sheet_for(r, sheet_id, s)?;
    let sheet_rule = s
        .scope
        .sheets
        .iter()
        .find(|tab| tab.sheet_id == sheet.sheet_id)?;
    for region in &sheet.regions {
        if !region.complete || !region.range.contains(r.row, r.column) {
            continue;
        }
        let rule = sheet_rule
            .regions
            .iter()
            .find(|rule| rule.id == region.region_id)?;
        let header = region.headers.iter().find(|c| c.column == r.column)?;
        if header.header.is_empty()
            || region
                .headers
                .iter()
                .filter(|c| c.header == header.header)
                .count()
                != 1
        {
            return None;
        }
        let column = rule
            .columns
            .iter()
            .find(|c| c.header == header.header)
            .map(|c| c.id.clone())
            .unwrap_or_else(|| format!("header:{}", header.header));
        if r.row == rule.header_row {
            return Some(format!(
                "{}:{}:header:{}",
                sheet.sheet_id, region.region_id, column
            ));
        }
        let row = region.rows.iter().find(|row| row.row == r.row)?;
        let key = row.key_hash.as_ref()?;
        if region
            .rows
            .iter()
            .filter(|row| row.key_hash.as_ref() == Some(key))
            .count()
            != 1
        {
            return None;
        }
        return Some(format!(
            "{}:{}:{}:{}",
            sheet.sheet_id, region.region_id, key, column
        ));
    }
    None
}

fn tokenize(formula: &str, mode: FormulaMode) -> Option<Vec<Token>> {
    if formula.len() > 16_384 || !formula.starts_with('=') {
        return None;
    }
    let chars: Vec<char> = formula[1..].chars().collect();
    let mut i = 0;
    let mut result = Vec::new();
    while i < chars.len() {
        let c = chars[i];
        if c.is_ascii_whitespace() {
            if matches!(mode, FormulaMode::Exact) {
                result.push(Token::Symbol(c));
            }
            i += 1;
            continue;
        }
        if c == '"' {
            let start = i;
            i += 1;
            let mut closed = false;
            while i < chars.len() {
                if chars[i] == '"' {
                    i += 1;
                    if i < chars.len() && chars[i] == '"' {
                        i += 1;
                    } else {
                        closed = true;
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            if !closed {
                return None;
            }
            result.push(Token::Literal(chars[start..i].iter().collect()));
            continue;
        }
        let (sheet, start) = if c == '\'' {
            let mut name = String::new();
            let mut j = i + 1;
            let mut closed = false;
            while j < chars.len() {
                if chars[j] == '\'' {
                    j += 1;
                    if j < chars.len() && chars[j] == '\'' {
                        name.push('\'');
                        j += 1;
                    } else {
                        closed = true;
                        break;
                    }
                } else {
                    name.push(chars[j]);
                    j += 1;
                }
            }
            if !closed || chars.get(j) != Some(&'!') {
                return None;
            }
            (Some(name), j + 1)
        } else {
            let mut j = i;
            while j < chars.len()
                && (chars[j].is_ascii_alphanumeric() || matches!(chars[j], '_' | '.'))
            {
                j += 1;
            }
            if j > i && chars.get(j) == Some(&'!') {
                (Some(chars[i..j].iter().collect()), j + 1)
            } else {
                (None, i)
            }
        };
        if let Some((mut reference, end)) = parse_reference(&chars, start) {
            // A token such as LOG10(...) is a function, not cell LOG10.
            if chars.get(end) != Some(&'(') {
                reference.sheet = sheet;
                // A range endpoint inherits the qualifier of its first endpoint.
                // Whitespace is not accepted between tokens in this subset.
                if reference.sheet.is_none() {
                    let mut significant = result
                        .iter()
                        .rev()
                        .filter(|t| !matches!(t,Token::Symbol(c) if c.is_ascii_whitespace()));
                    if matches!(significant.next(), Some(Token::Symbol(':'))) {
                        if let Some(Token::Reference(start)) = significant.next() {
                            reference.sheet = start.sheet.clone();
                        }
                    }
                }
                result.push(Token::Reference(reference));
                i = end;
                continue;
            }
        }
        if sheet.is_some() {
            return None;
        }
        if c.is_ascii_digit() || c == '.' && chars.get(i + 1).is_some_and(char::is_ascii_digit) {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let literal: String = chars[start..i].iter().collect();
            if literal.parse::<f64>().is_err() {
                return None;
            }
            result.push(Token::Literal(literal));
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || matches!(chars[i], '_' | '.'))
            {
                i += 1;
            }
            let name: String = chars[start..i]
                .iter()
                .collect::<String>()
                .to_ascii_uppercase();
            if matches!(
                name.as_str(),
                "INDIRECT"
                    | "OFFSET"
                    | "IMPORTRANGE"
                    | "GOOGLEFINANCE"
                    | "IMPORTXML"
                    | "IMPORTDATA"
                    | "IMPORTHTML"
                    | "IMPORTFEED"
            ) {
                return None;
            }
            if chars.get(i) != Some(&'(') && !matches!(name.as_str(), "TRUE" | "FALSE") {
                return None;
            }
            result.push(Token::Literal(if matches!(mode, FormulaMode::Exact) {
                chars[start..i].iter().collect()
            } else {
                name
            }));
            continue;
        }
        if "=+-*/^&(),:;<>%{}".contains(c) {
            result.push(Token::Symbol(c));
            i += 1;
            continue;
        }
        return None;
    }
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}
fn parse_reference(chars: &[char], start: usize) -> Option<(Reference, usize)> {
    let mut i = start;
    let absolute_column = chars.get(i) == Some(&'$');
    if absolute_column {
        i += 1;
    }
    let column_start = i;
    let mut column = 0u32;
    while i < chars.len() && chars[i].is_ascii_alphabetic() {
        column = column
            .checked_mul(26)?
            .checked_add(chars[i].to_ascii_uppercase() as u32 - 'A' as u32 + 1)?;
        i += 1;
    }
    if i == column_start || column > 18_278 {
        return None;
    }
    let absolute_row = chars.get(i) == Some(&'$');
    if absolute_row {
        i += 1;
    }
    let row_start = i;
    let mut row = 0u32;
    while i < chars.len() && chars[i].is_ascii_digit() {
        row = row
            .checked_mul(10)?
            .checked_add(chars[i] as u32 - '0' as u32)?;
        i += 1;
    }
    if i == row_start
        || row == 0
        || row > 1_000_000
        || chars
            .get(i)
            .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
    {
        return None;
    }
    Some((
        Reference {
            sheet: None,
            row: row - 1,
            column: column - 1,
            absolute_row,
            absolute_column,
        },
        i,
    ))
}

#[cfg(test)]
mod qualifier_tests {
    use super::*;
    #[test]
    fn ranges_inherit_sheet_qualifier_on_both_endpoints() {
        for (formula, mode) in [
            ("='Other'!A1:B2", FormulaMode::Structural),
            ("='Other'!A1 : B2", FormulaMode::Exact),
        ] {
            let tokens = tokenize(formula, mode).unwrap();
            let refs: Vec<_> = tokens
                .iter()
                .filter_map(|t| {
                    if let Token::Reference(r) = t {
                        Some(r)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(refs.len(), 2);
            assert!(refs.iter().all(|r| r.sheet.as_deref() == Some("Other")));
        }
    }
}
