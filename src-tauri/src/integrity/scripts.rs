//! Conservative JavaScript syntax analysis. No JavaScript is evaluated.
use super::models::*;
use crate::google::{ScriptFile, ScriptFileType};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::{Duration, Instant},
};
use tree_sitter::{Node, ParseOptions, Parser};

const MAX_FILE_BYTES: usize = 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 4 * 1024 * 1024;
const MAX_FILES: usize = 128;
const MAX_NODES: usize = 100_000;
const MAX_DEPTH: usize = 256;

#[derive(Clone, Debug)]
enum Value {
    Unknown,
    Api,
    Spreadsheet,
    Sheet(String),
    Range(String, GridRect),
    Text(String),
    Number(u32),
}

struct Analyzer<'a> {
    source: &'a str,
    file: &'a str,
    spreadsheet: &'a str,
    bound: bool,
    scopes: Vec<BTreeMap<String, Value>>,
    assigned: BTreeSet<String>,
    dependencies: Vec<ScriptDependency>,
    limitations: BTreeSet<String>,
    unsafe_semantics: bool,
}
fn text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    source.get(node.byte_range()).unwrap_or("")
}
fn children(node: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}
fn identifiers(node: Node<'_>, source: &str, names: &mut BTreeSet<String>) {
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if n.kind() == "identifier" {
            names.insert(text(n, source).into());
        } else {
            stack.extend(children(n));
        }
    }
}
fn declarations(node: Node<'_>, source: &str, names: &mut BTreeSet<String>) {
    for n in children(node) {
        match n.kind() {
            "lexical_declaration" | "variable_declaration" => {
                for declaration in children(n) {
                    if let Some(name) = declaration.child_by_field_name("name") {
                        identifiers(name, source, names);
                    }
                }
            }
            "function_declaration" | "generator_function_declaration" | "class_declaration" => {
                if let Some(name) = n.child_by_field_name("name") {
                    names.insert(text(name, source).into());
                }
            }
            _ => {}
        }
    }
}
fn hoisted_vars(node: Node<'_>, source: &str, names: &mut BTreeSet<String>) {
    let mut stack = children(node);
    while let Some(n) = stack.pop() {
        if matches!(
            n.kind(),
            "function_declaration"
                | "generator_function_declaration"
                | "function_expression"
                | "generator_function"
                | "arrow_function"
                | "method_definition"
        ) {
            continue;
        }
        if n.kind() == "variable_declaration" {
            for declaration in children(n) {
                if let Some(name) = declaration.child_by_field_name("name") {
                    identifiers(name, source, names);
                }
            }
        }
        stack.extend(children(n));
    }
}
fn relevant_method(name: &str) -> Option<DependencyKind> {
    Some(match name {
        "getSheetByName" => DependencyKind::SheetByName,
        "getRange" => DependencyKind::RangeA1,
        "getColumn" => DependencyKind::Column,
        "getLastRow" => DependencyKind::LastRow,
        "getLastColumn" => DependencyKind::LastColumn,
        "getValues" => DependencyKind::ReadValues,
        "getDisplayValues" => DependencyKind::ReadDisplayValues,
        "getFormulas" => DependencyKind::ReadFormulas,
        _ => return None,
    })
}
impl Analyzer<'_> {
    fn lookup(&self, name: &str) -> Value {
        if self.assigned.contains(name) {
            return Value::Unknown;
        }
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.get(name))
            .cloned()
            .unwrap_or_else(|| {
                if name == "SpreadsheetApp" {
                    Value::Api
                } else {
                    Value::Unknown
                }
            })
    }
    fn unknown(&mut self, code: &str) {
        self.limitations.insert(code.into());
    }
    fn dependency(
        &mut self,
        node: Node<'_>,
        kind: DependencyKind,
        sheet: Option<String>,
        range: Option<GridRect>,
        coverage: DependencyCoverage,
        code: &str,
    ) {
        if coverage != DependencyCoverage::Static {
            self.unknown(code);
        }
        self.dependencies.push(ScriptDependency {
            file: self.file.into(),
            line: node.start_position().row as u32 + 1,
            kind,
            sheet_name: sheet,
            range,
            coverage,
            detail_code: code.into(),
        });
    }
    fn scope(&mut self, node: Node<'_>, depth: usize) -> Value {
        let mut names = BTreeSet::new();
        declarations(node, self.source, &mut names);
        if node.kind() == "program" {
            hoisted_vars(node, self.source, &mut names);
        }
        self.scopes
            .push(names.into_iter().map(|n| (n, Value::Unknown)).collect());
        for child in children(node) {
            self.eval(child, depth + 1);
        }
        self.scopes.pop();
        Value::Unknown
    }
    fn eval(&mut self, node: Node<'_>, depth: usize) -> Value {
        if depth > MAX_DEPTH {
            self.unknown("ast_depth_limit");
            return Value::Unknown;
        }
        match node.kind() {
            "program" | "statement_block" | "switch_body" => self.scope(node, depth),
            "comment" | "regex" => Value::Unknown,
            "identifier" => self.lookup(text(node, self.source)),
            "string" | "template_string" => decode_string(text(node, self.source))
                .map(Value::Text)
                .unwrap_or(Value::Unknown),
            "number" => text(node, self.source)
                .parse::<u32>()
                .map(Value::Number)
                .unwrap_or(Value::Unknown),
            "parenthesized_expression" => children(node)
                .first()
                .map(|n| self.eval(*n, depth + 1))
                .unwrap_or(Value::Unknown),
            "lexical_declaration" | "variable_declaration" => {
                let immutable = node
                    .child_by_field_name("kind")
                    .is_some_and(|n| text(n, self.source) == "const");
                for declaration in children(node) {
                    if declaration.kind() != "variable_declarator" {
                        continue;
                    }
                    let value = declaration
                        .child_by_field_name("value")
                        .map(|n| self.eval(n, depth + 1))
                        .unwrap_or(Value::Unknown);
                    if let Some(name) = declaration.child_by_field_name("name") {
                        if name.kind() == "identifier" {
                            let name = text(name, self.source).to_owned();
                            let value = if immutable && !self.assigned.contains(&name) {
                                value
                            } else {
                                Value::Unknown
                            };
                            if let Some(scope) = self.scopes.last_mut() {
                                scope.insert(name, value);
                            }
                        }
                    }
                }
                Value::Unknown
            }
            "function_declaration"
            | "generator_function_declaration"
            | "function_expression"
            | "generator_function"
            | "arrow_function"
            | "method_definition" => {
                let mut names = BTreeSet::new();
                for field in ["parameters", "parameter", "name"] {
                    if let Some(params) = node.child_by_field_name(field) {
                        identifiers(params, self.source, &mut names);
                    }
                }
                if let Some(body) = node.child_by_field_name("body") {
                    hoisted_vars(body, self.source, &mut names);
                }
                self.scopes
                    .push(names.into_iter().map(|n| (n, Value::Unknown)).collect());
                if let Some(body) = node.child_by_field_name("body") {
                    self.eval(body, depth + 1);
                }
                self.scopes.pop();
                Value::Unknown
            }
            "for_statement" | "for_in_statement" | "catch_clause" => {
                self.scopes.push(BTreeMap::new());
                if let Some(parameter) = node.child_by_field_name("parameter") {
                    let mut names = BTreeSet::new();
                    identifiers(parameter, self.source, &mut names);
                    for name in names {
                        self.scopes.last_mut().unwrap().insert(name, Value::Unknown);
                    }
                }
                for child in children(node) {
                    self.eval(child, depth + 1);
                }
                self.scopes.pop();
                Value::Unknown
            }
            "assignment_expression" | "augmented_assignment_expression" | "update_expression" => {
                for child in children(node) {
                    self.eval(child, depth + 1);
                }
                Value::Unknown
            }
            "call_expression" => self.call(node, depth),
            "member_expression" => {
                if let Some(object) = node.child_by_field_name("object") {
                    let receiver = self.eval(object, depth + 1);
                    let method = node
                        .child_by_field_name("property")
                        .map(|n| text(n, self.source))
                        .unwrap_or("");
                    if let Some(kind) = relevant_method(method) {
                        self.dependency(
                            node,
                            kind,
                            None,
                            None,
                            DependencyCoverage::Partial,
                            "detached_method_reference_unanalyzed",
                        );
                    } else if matches!(receiver, Value::Api) {
                        self.dependency(
                            node,
                            DependencyKind::Dynamic,
                            None,
                            None,
                            DependencyCoverage::Dynamic,
                            "detached_api_method_reference",
                        );
                    }
                }
                Value::Unknown
            }
            "with_statement" => {
                self.unknown("dynamic_with_scope");
                self.unsafe_semantics = true;
                for child in children(node) {
                    self.eval(child, depth + 1);
                }
                Value::Unknown
            }
            _ => {
                for child in children(node) {
                    self.eval(child, depth + 1);
                }
                Value::Unknown
            }
        }
    }
    fn call(&mut self, node: Node<'_>, depth: usize) -> Value {
        let Some(function) = node.child_by_field_name("function") else {
            return Value::Unknown;
        };
        let Some(arguments) = node.child_by_field_name("arguments") else {
            return Value::Unknown;
        };
        let values: Vec<_> = children(arguments)
            .into_iter()
            .filter(|n| n.kind() != "comment")
            .map(|n| self.eval(n, depth + 1))
            .collect();
        if function.kind() != "member_expression" {
            if function.kind() == "subscript_expression" {
                let receiver = function
                    .child_by_field_name("object")
                    .map(|n| self.eval(n, depth + 1))
                    .unwrap_or(Value::Unknown);
                let method = function
                    .child_by_field_name("index")
                    .and_then(|n| decode_string(text(n, self.source)));
                if let Some(kind) = method.as_deref().and_then(relevant_method) {
                    self.dependency(
                        node,
                        kind,
                        None,
                        None,
                        DependencyCoverage::Partial,
                        "computed_method_unanalyzed",
                    );
                } else if matches!(
                    receiver,
                    Value::Api | Value::Spreadsheet | Value::Sheet(_) | Value::Range(..)
                ) {
                    self.dependency(
                        node,
                        DependencyKind::Dynamic,
                        None,
                        None,
                        DependencyCoverage::Dynamic,
                        "computed_method_unanalyzed",
                    );
                }
                return Value::Unknown;
            }
            if matches!(text(function, self.source), "eval" | "Function") {
                self.unsafe_semantics = true;
                self.unknown("dynamic_code_execution_unanalyzed");
            }
            self.eval(function, depth + 1);
            return Value::Unknown;
        }
        let Some(object) = function.child_by_field_name("object") else {
            return Value::Unknown;
        };
        let receiver = self.eval(object, depth + 1);
        let method = function
            .child_by_field_name("property")
            .map(|n| text(n, self.source))
            .unwrap_or("");
        if matches!(receiver, Value::Api) {
            return match method {
                "getActiveSpreadsheet" if values.is_empty() && self.bound => Value::Spreadsheet,
                "openById" if matches!(values.as_slice(),[Value::Text(id)] if id==self.spreadsheet) => {
                    Value::Spreadsheet
                }
                _ => {
                    self.dependency(
                        node,
                        DependencyKind::Dynamic,
                        None,
                        None,
                        DependencyCoverage::Dynamic,
                        if method == "getActiveSpreadsheet" {
                            "active_spreadsheet_unproven"
                        } else {
                            "external_or_dynamic_spreadsheet"
                        },
                    );
                    Value::Unknown
                }
            };
        }
        let Some(kind) = relevant_method(method) else {
            if matches!(
                receiver,
                Value::Spreadsheet | Value::Sheet(_) | Value::Range(..)
            ) {
                self.dependency(
                    node,
                    DependencyKind::Dynamic,
                    None,
                    None,
                    DependencyCoverage::Partial,
                    "unsupported_spreadsheet_method",
                );
            }
            return Value::Unknown;
        };
        match (method, receiver) {
            ("getSheetByName", Value::Spreadsheet) => {
                if let [Value::Text(name)] = values.as_slice() {
                    if !name.is_empty() && name.len() <= 512 && !name.chars().any(char::is_control)
                    {
                        self.dependency(
                            node,
                            kind,
                            Some(name.clone()),
                            None,
                            DependencyCoverage::Static,
                            "static_sheet_name",
                        );
                        return Value::Sheet(name.clone());
                    }
                }
                self.dependency(
                    node,
                    kind,
                    None,
                    None,
                    DependencyCoverage::Dynamic,
                    "dynamic_sheet_name",
                );
                Value::Unknown
            }
            ("getRange", Value::Sheet(name)) => self.range_call(node, values, Some(name)),
            ("getRange", Value::Spreadsheet) => self.range_call(node, values, None),
            ("getLastRow" | "getLastColumn", Value::Sheet(name)) if values.is_empty() => {
                self.dependency(
                    node,
                    kind,
                    Some(name),
                    None,
                    DependencyCoverage::Static,
                    "content_extent_query",
                );
                Value::Unknown
            }
            ("getColumn", Value::Range(name, range)) if values.is_empty() => {
                self.dependency(
                    node,
                    kind,
                    Some(name),
                    Some(range.clone()),
                    DependencyCoverage::Static,
                    "static_range_column",
                );
                Value::Number(range.start_col + 1)
            }
            ("getValues" | "getDisplayValues" | "getFormulas", Value::Range(name, range))
                if values.is_empty() =>
            {
                self.dependency(
                    node,
                    kind,
                    Some(name),
                    Some(range),
                    DependencyCoverage::Static,
                    "static_range_read",
                );
                Value::Unknown
            }
            _ => {
                self.dependency(
                    node,
                    kind,
                    None,
                    None,
                    DependencyCoverage::Partial,
                    "receiver_or_arguments_unproven",
                );
                Value::Unknown
            }
        }
    }
    fn range_call(&mut self, node: Node<'_>, values: Vec<Value>, sheet: Option<String>) -> Value {
        let kind = if matches!(values.as_slice(), [Value::Text(_)]) {
            DependencyKind::RangeA1
        } else {
            DependencyKind::RangeNumeric
        };
        let resolved = if let [Value::Text(raw)] = values.as_slice() {
            parse_a1(raw)
                .and_then(|(qualified, rect)| qualified.or(sheet.clone()).map(|name| (name, rect)))
        } else if (2..=4).contains(&values.len())
            && values.iter().all(|v| matches!(v, Value::Number(_)))
        {
            let numbers: Vec<_> = values
                .iter()
                .map(|v| if let Value::Number(n) = v { *n } else { 0 })
                .collect();
            let rect = (|| {
                let row = numbers[0].checked_sub(1)?;
                let col = numbers[1].checked_sub(1)?;
                let rows = numbers.get(2).copied().unwrap_or(1);
                let cols = numbers.get(3).copied().unwrap_or(1);
                let rect = GridRect {
                    start_row: row,
                    end_row: row.checked_add(rows)?,
                    start_col: col,
                    end_col: col.checked_add(cols)?,
                };
                rect.valid().then_some(rect)
            })();
            if rect.is_none() {
                self.dependency(
                    node,
                    kind,
                    sheet,
                    None,
                    DependencyCoverage::Static,
                    "invalid_numeric_range",
                );
                return Value::Unknown;
            }
            sheet.clone().zip(rect)
        } else {
            None
        };
        if let Some((name, rect)) = resolved {
            self.dependency(
                node,
                kind,
                Some(name.clone()),
                Some(rect.clone()),
                DependencyCoverage::Static,
                "static_range",
            );
            Value::Range(name, rect)
        } else {
            self.dependency(
                node,
                kind,
                sheet,
                None,
                DependencyCoverage::Dynamic,
                "dynamic_or_unsupported_range",
            );
            Value::Unknown
        }
    }
}

/// Decode supported JS string literals, never eval/regex-evaluate source.
fn decode_string(raw: &str) -> Option<String> {
    let mut chars = raw.chars();
    let quote = chars.next()?;
    if !matches!(quote, '\'' | '"' | '`') || !raw.ends_with(quote) {
        return None;
    }
    let end = raw.len().checked_sub(quote.len_utf8())?;
    let inner = raw.get(quote.len_utf8()..end)?;
    if quote == '`' && inner.contains("${") {
        return None;
    }
    let mut result = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        match chars.next()? {
            'n' => result.push('\n'),
            'r' => result.push('\r'),
            't' => result.push('\t'),
            'b' => result.push('\u{8}'),
            'f' => result.push('\u{c}'),
            'v' => result.push('\u{b}'),
            '0' => result.push('\0'),
            '\n' => {}
            '\r' => {
                if chars.clone().next() == Some('\n') {
                    chars.next();
                }
            }
            '\'' => result.push('\''),
            '"' => result.push('"'),
            '`' => result.push('`'),
            '\\' => result.push('\\'),
            'x' | 'u' => return None,
            _ => return None,
        }
    }
    Some(result)
}
fn parse_cell(raw: &str) -> Option<(u32, u32)> {
    let mut chars = raw.chars().peekable();
    if chars.peek() == Some(&'$') {
        chars.next();
    }
    let mut col = 0u32;
    let mut letters = 0;
    while chars.peek().is_some_and(char::is_ascii_alphabetic) {
        let c = chars.next()?;
        col = col
            .checked_mul(26)?
            .checked_add(c.to_ascii_uppercase() as u32 - 'A' as u32 + 1)?;
        letters += 1;
    }
    if letters == 0 || letters > 3 {
        return None;
    }
    if chars.peek() == Some(&'$') {
        chars.next();
    }
    let digits: String = chars.collect();
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let row = digits.parse::<u32>().ok()?;
    Some((row.checked_sub(1)?, col.checked_sub(1)?))
}
fn parse_a1(raw: &str) -> Option<(Option<String>, GridRect)> {
    if raw.len() > 1024 || raw.chars().any(char::is_control) {
        return None;
    }
    let (sheet, range) = if let Some((name, range)) = raw.rsplit_once('!') {
        let name = if name.starts_with('\'') && name.ends_with('\'') && name.len() >= 2 {
            name[1..name.len() - 1].replace("''", "'")
        } else {
            if !name
                .chars()
                .all(|c| c.is_alphanumeric() || matches!(c, '_' | '.'))
            {
                return None;
            }
            name.into()
        };
        if name.is_empty() {
            return None;
        }
        (Some(name), range)
    } else {
        (None, raw)
    };
    let (start, end) = range
        .split_once(':')
        .map_or((range, range), |(a, b)| (a, b));
    let (start_row, start_col) = parse_cell(start)?;
    let (last_row, last_col) = parse_cell(end)?;
    let rect = GridRect {
        start_row,
        end_row: last_row.checked_add(1)?,
        start_col,
        end_col: last_col.checked_add(1)?,
    };
    rect.valid().then_some((sheet, rect))
}

pub fn analyze_sources(
    files: &[ScriptFile],
    spreadsheet_id: &str,
    bound_to_monitored: bool,
) -> EngineResult<ScriptAnalysis> {
    if files.len() > MAX_FILES
        || files.iter().any(|f| f.source.len() > MAX_FILE_BYTES)
        || files.iter().map(|f| f.source.len()).sum::<usize>() > MAX_TOTAL_BYTES
    {
        return Err(EngineError::Limit);
    }
    crate::google::validate_google_id(spreadsheet_id).map_err(|_| EngineError::InvalidScope)?;
    let mut analysis = ScriptAnalysis {
        parser: "tree-sitter-javascript-0.25".into(),
        complete: true,
        dependencies: Vec::new(),
        limitations: Vec::new(),
    };
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_javascript::LANGUAGE.into())
        .map_err(|_| EngineError::InvalidState)?;
    let mut parsed = Vec::new();
    let mut globals = BTreeSet::new();
    let mut assigned = BTreeSet::new();
    let mut nodes = 0usize;
    let mut limitations = BTreeSet::new();
    let mut unsafe_semantics = false;
    for file in files {
        match file.file_type {
            ScriptFileType::Html => {
                limitations.insert("html_inline_javascript_unanalyzed".into());
                continue;
            }
            ScriptFileType::Json => {
                continue;
            }
            ScriptFileType::ServerJs => {}
        }
        let started = Instant::now();
        let mut progress =
            |_: &tree_sitter::ParseState| started.elapsed() > Duration::from_millis(250);
        let source = file.source.as_bytes();
        let tree = parser.parse_with_options(
            &mut |offset, _| source.get(offset..).unwrap_or(&[]),
            None,
            Some(ParseOptions::new().progress_callback(&mut progress)),
        );
        let Some(tree) = tree else {
            parser.reset();
            limitations.insert("javascript_parse_deadline".into());
            continue;
        };
        if tree.root_node().has_error() {
            limitations.insert("javascript_parse_error".into());
            continue;
        }
        let mut stack = vec![(tree.root_node(), 0usize)];
        let mut bounded = true;
        while let Some((node, depth)) = stack.pop() {
            nodes += 1;
            if nodes > MAX_NODES || depth > MAX_DEPTH {
                bounded = false;
                break;
            }
            if matches!(
                node.kind(),
                "assignment_expression" | "augmented_assignment_expression" | "update_expression"
            ) {
                if let Some(left) = node
                    .child_by_field_name("left")
                    .or_else(|| node.child_by_field_name("argument"))
                {
                    if left.kind() == "identifier" {
                        assigned.insert(text(left, &file.source).into());
                    }
                    if left.kind() == "member_expression"
                        && left.child_by_field_name("property").is_some_and(|p| {
                            relevant_method(text(p, &file.source)).is_some()
                                || matches!(
                                    text(p, &file.source),
                                    "__proto__" | "prototype" | "getActiveSpreadsheet" | "openById"
                                )
                        })
                    {
                        unsafe_semantics = true;
                    }
                }
            }
            if node.kind() == "with_statement"
                || node.kind() == "call_expression"
                    && node
                        .child_by_field_name("function")
                        .is_some_and(|n| matches!(text(n, &file.source), "eval" | "Function"))
            {
                unsafe_semantics = true;
            }
            if node.kind() == "new_expression"
                && node
                    .child_by_field_name("constructor")
                    .is_some_and(|n| text(n, &file.source) == "Function")
            {
                unsafe_semantics = true;
            }
            stack.extend(children(node).into_iter().map(|n| (n, depth + 1)));
        }
        if !bounded {
            limitations.insert("javascript_ast_budget_exceeded".into());
            continue;
        }
        declarations(tree.root_node(), &file.source, &mut globals);
        parsed.push((file, tree));
    }
    for (file, tree) in parsed {
        let mut analyzer = Analyzer {
            source: &file.source,
            file: &file.name,
            spreadsheet: spreadsheet_id,
            bound: bound_to_monitored,
            scopes: vec![globals
                .iter()
                .map(|n| (n.clone(), Value::Unknown))
                .collect()],
            assigned: assigned.clone(),
            dependencies: Vec::new(),
            limitations: BTreeSet::new(),
            unsafe_semantics,
        };
        analyzer.eval(tree.root_node(), 0);
        if analyzer.unsafe_semantics {
            analyzer.unknown("dynamic_mutation_or_execution_unanalyzed");
            for dep in &mut analyzer.dependencies {
                dep.coverage = DependencyCoverage::Partial;
                dep.detail_code = "dynamic_mutation_or_execution_unanalyzed".into();
            }
        }
        analysis.dependencies.extend(analyzer.dependencies);
        limitations.extend(analyzer.limitations);
    }
    analysis.complete = limitations.is_empty();
    analysis.limitations = limitations.into_iter().collect();
    Ok(analysis)
}

fn finding(
    dep: Option<&ScriptDependency>,
    critical: bool,
    outcome: FindingOutcome,
    code: &str,
    sheet_id: Option<u32>,
    expected: Option<String>,
    found: Option<String>,
) -> Finding {
    let location = dep.map(|d| format!("{}:{}", d.file, d.line));
    // Identity belongs to the dependency/rule, not its latest observed state.
    // Values such as current grid size or a field's new column stay in the
    // finding fingerprint so a changed violation cannot falsely recover itself.
    let logical_column = matches!(
        code,
        "script_logical_column_changed" | "script_column_identity_ambiguous"
    )
    .then(|| {
        expected
            .as_deref()
            .map(|value| value.split('@').next().unwrap_or(value))
    })
    .flatten();
    let dependency = dep.map(|d| (&d.file, d.line, &d.sheet_name));
    let key = fingerprint(&(dependency, code, sheet_id, logical_column));
    Finding {
        key: format!("script-{key}"),
        sheet_id,
        rule_id: "script_dependency".into(),
        code: code.into(),
        severity: if critical {
            Severity::Critical
        } else {
            Severity::Warning
        },
        outcome,
        expected,
        found,
        location,
    }
}
/// Validates declared static coverage. Names/headers are private snapshot metadata;
/// source text never enters findings or diagnostics.
pub fn validate_dependencies(
    analysis: &ScriptAnalysis,
    baseline: &StructuralSnapshot,
    current: &StructuralSnapshot,
    critical: bool,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut seen = BTreeSet::new();
    if !analysis.complete {
        findings.push(finding(
            None,
            critical,
            FindingOutcome::Inconclusive,
            "script_analysis_partial",
            None,
            None,
            None,
        ));
    }
    for dep in &analysis.dependencies {
        if dep.coverage != DependencyCoverage::Static {
            findings.push(finding(
                Some(dep),
                critical,
                FindingOutcome::Inconclusive,
                "script_dependency_unproven",
                None,
                None,
                None,
            ));
            continue;
        }
        if dep.detail_code == "invalid_numeric_range" {
            findings.push(finding(
                Some(dep),
                critical,
                FindingOutcome::Violation,
                "script_range_invalid",
                None,
                Some("positive_valid_bounds".into()),
                None,
            ));
            continue;
        }
        let Some(name) = &dep.sheet_name else {
            continue;
        };
        let matches: Vec<_> = current.sheets.iter().filter(|s| &s.title == name).collect();
        if matches.len() != 1 {
            let confirmed = matches.is_empty() && current.coverage.metadata_complete;
            findings.push(finding(
                Some(dep),
                critical,
                if confirmed {
                    FindingOutcome::Violation
                } else {
                    FindingOutcome::Inconclusive
                },
                if confirmed {
                    "script_sheet_missing"
                } else {
                    "script_sheet_unproven"
                },
                None,
                Some(name.clone()),
                None,
            ));
            continue;
        }
        let sheet = matches[0];
        let Some(range) = &dep.range else {
            continue;
        };
        let range_key = (
            dep.file.clone(),
            dep.line,
            sheet.sheet_id,
            range.start_row,
            range.end_row,
            range.start_col,
            range.end_col,
        );
        if !seen.insert(range_key) {
            continue;
        }
        if range.end_row > sheet.row_count || range.end_col > sheet.column_count {
            findings.push(finding(
                Some(dep),
                critical,
                if current.coverage.metadata_complete {
                    FindingOutcome::Violation
                } else {
                    FindingOutcome::Inconclusive
                },
                "script_range_outside_grid",
                Some(sheet.sheet_id),
                Some(format!("{}x{}", sheet.row_count, sheet.column_count)),
                Some(format!("{}x{}", range.end_row, range.end_col)),
            ));
            continue;
        }
        let Some(rule) = baseline
            .scope
            .sheets
            .iter()
            .find(|r| r.sheet_id == sheet.sheet_id)
        else {
            findings.push(finding(
                Some(dep),
                critical,
                FindingOutcome::Inconclusive,
                "script_range_outside_monitored_scope",
                Some(sheet.sheet_id),
                None,
                None,
            ));
            continue;
        };
        let Some(old_sheet) = baseline
            .sheets
            .iter()
            .find(|s| s.sheet_id == sheet.sheet_id)
        else {
            findings.push(finding(
                Some(dep),
                critical,
                FindingOutcome::Inconclusive,
                "script_baseline_sheet_unobserved",
                Some(sheet.sheet_id),
                None,
                None,
            ));
            continue;
        };
        let mut covered = false;
        for region in &rule.regions {
            // An unobserved part makes coverage partial, but it cannot hide a
            // confirmed logical-field violation in the observed intersection.
            let overlaps = range.start_col < region.range.end_col
                && range.end_col > region.range.start_col
                && range.start_row < region.range.end_row
                && range.end_row > region.range.start_row;
            if !overlaps {
                continue;
            }
            let old = old_sheet.regions.iter().find(|r| r.region_id == region.id);
            let now = sheet.regions.iter().find(|r| r.region_id == region.id);
            let (Some(old), Some(now)) = (old, now) else {
                continue;
            };
            if !old.complete || !now.complete {
                continue;
            }
            covered |= range.start_col >= region.range.start_col
                && range.end_col <= region.range.end_col
                && range.start_row >= region.range.start_row
                && range.end_row <= region.range.end_row;
            for column in &region.columns {
                if column.position < range.start_col || column.position >= range.end_col {
                    continue;
                }
                let expected: Vec<_> = old
                    .headers
                    .iter()
                    .filter(|h| h.header == column.header)
                    .collect();
                let actual: Vec<_> = now
                    .headers
                    .iter()
                    .filter(|h| h.header == column.header)
                    .collect();
                if expected.len() != 1 || actual.len() > 1 {
                    findings.push(finding(
                        Some(dep),
                        critical,
                        FindingOutcome::Inconclusive,
                        "script_column_identity_ambiguous",
                        Some(sheet.sheet_id),
                        Some(column.id.clone()),
                        None,
                    ));
                    continue;
                }
                let expected_col = expected[0].column;
                if actual.len() != 1 || actual[0].column != expected_col {
                    findings.push(finding(
                        Some(dep),
                        critical,
                        FindingOutcome::Violation,
                        "script_logical_column_changed",
                        Some(sheet.sheet_id),
                        Some(format!("{}@{}", column.id, expected_col)),
                        actual
                            .first()
                            .map(|c| format!("{}@{}", column.id, c.column)),
                    ));
                }
            }
        }
        if !covered {
            findings.push(finding(
                Some(dep),
                critical,
                FindingOutcome::Inconclusive,
                "script_range_coverage_partial",
                Some(sheet.sheet_id),
                None,
                None,
            ));
        }
    }
    let mut keys = BTreeSet::new();
    findings.retain(|f| keys.insert(f.key.clone()));
    findings
}

#[cfg(test)]
#[path = "script_tests.rs"]
mod tests;
