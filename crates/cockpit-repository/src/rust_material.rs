use std::collections::{BTreeMap, BTreeSet};

use cockpit_git::ChangeKind;
use serde::Serialize;

const MAX_RUST_SOURCE_BYTES: usize = 4 * 1024 * 1024;
const MAX_RUST_LEXICAL_TOKENS: usize = 262_144;
const MAX_RUST_CHANGED_CONTEXT_TOKENS: usize = 16_384;
const MAX_RUST_COMMENT_CONTEXT_BYTES: usize = 16 * 1024;

fn contains_instruction_marker(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    [
        "ignore previous instructions",
        "ignore all previous instructions",
        "ignore the contract",
        "override policy",
        "bypass policy",
        "disable governance",
        "system message",
    ]
    .iter()
    .any(|pattern| text.contains(pattern))
}

fn contains_risky_operation(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    [
        "delete",
        "rm -rf",
        "execute",
        "run ",
        "curl ",
        "secret",
        "token",
        "upload",
        "exfil",
        "disable test",
        "skip test",
        "publish",
        "push main",
    ]
    .iter()
    .any(|pattern| text.contains(pattern))
}

pub(super) fn contains_strong_instruction_injection(text: &str) -> bool {
    contains_instruction_marker(text) && contains_risky_operation(text)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RustMaterialAssessment {
    Clean,
    Finding,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialUnknownCause {
    MissingCompleteSource,
    NonTextMaterial,
    SourceOverBudget,
    InvalidProvenance,
    ParseUnavailable,
    AnalysisIncomplete,
    ReadableCommittedRustSyntaxUnknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RustMaterialDiagnosis {
    pub assessment: RustMaterialAssessment,
    pub unknown_cause: Option<MaterialUnknownCause>,
}

impl RustMaterialDiagnosis {
    const CLEAN: Self = Self {
        assessment: RustMaterialAssessment::Clean,
        unknown_cause: None,
    };
    const FINDING: Self = Self {
        assessment: RustMaterialAssessment::Finding,
        unknown_cause: None,
    };

    const fn unknown(cause: MaterialUnknownCause) -> Self {
        Self {
            assessment: RustMaterialAssessment::Unknown,
            unknown_cause: Some(cause),
        }
    }
}

fn rust_literal_text(literal: &syn::Lit) -> Option<String> {
    match literal {
        syn::Lit::Str(value) => Some(value.value()),
        syn::Lit::ByteStr(value) => Some(String::from_utf8_lossy(&value.value()).into_owned()),
        _ => None,
    }
}

fn resolve_rust_text(
    expr: &syn::Expr,
    locals: &BTreeMap<String, String>,
    functions: &BTreeMap<String, String>,
    depth: usize,
) -> Option<String> {
    if depth > 8 {
        return None;
    }
    let value = match expr {
        syn::Expr::Lit(value) => rust_literal_text(&value.lit),
        syn::Expr::Path(value) if value.path.segments.len() == 1 => locals
            .get(&value.path.segments[0].ident.to_string())
            .cloned(),
        syn::Expr::Call(value) if value.args.is_empty() => {
            if let syn::Expr::Path(path) = value.func.as_ref() {
                if path.path.segments.len() == 1 {
                    functions
                        .get(&path.path.segments[0].ident.to_string())
                        .cloned()
                } else {
                    None
                }
            } else {
                None
            }
        }
        syn::Expr::Paren(value) => resolve_rust_text(&value.expr, locals, functions, depth + 1),
        syn::Expr::Group(value) => resolve_rust_text(&value.expr, locals, functions, depth + 1),
        syn::Expr::Binary(value) if matches!(value.op, syn::BinOp::Add(_)) => {
            let mut left = resolve_rust_text(&value.left, locals, functions, depth + 1)?;
            left.push_str(&resolve_rust_text(
                &value.right,
                locals,
                functions,
                depth + 1,
            )?);
            Some(left)
        }
        syn::Expr::Macro(value) => {
            use syn::parse::Parser;
            let name = value.mac.path.get_ident()?.to_string();
            let args = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
                .parse2(value.mac.tokens.clone())
                .ok()?;
            let mut parts = args
                .iter()
                .map(|arg| resolve_rust_text(arg, locals, functions, depth + 1));
            match name.as_str() {
                "concat" => {
                    let mut result = String::new();
                    for part in parts {
                        result.push_str(&part?);
                    }
                    Some(result)
                }
                "format" => {
                    let template = parts.next()??;
                    let values = parts.collect::<Option<Vec<_>>>()?;
                    if template.matches("{}").count() != values.len()
                        || template.replace("{}", "").contains(['{', '}'])
                    {
                        None
                    } else {
                        let mut result = String::new();
                        let mut segments = template.split("{}");
                        result.push_str(segments.next().unwrap_or_default());
                        for (value, suffix) in values.iter().zip(segments) {
                            result.push_str(value);
                            result.push_str(suffix);
                        }
                        Some(result)
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }?;
    (value.len() <= 16 * 1024).then_some(value)
}

fn count_token_tree(tokens: proc_macro2::TokenStream, depth: usize, remaining: &mut usize) -> bool {
    if depth > 64 {
        return false;
    }
    for token in tokens {
        if *remaining == 0 {
            return false;
        }
        *remaining -= 1;
        if let proc_macro2::TokenTree::Group(group) = token
            && !count_token_tree(group.stream(), depth + 1, remaining)
        {
            return false;
        }
    }
    true
}

fn source_offset(starts: &[usize], position: proc_macro2::LineColumn) -> Option<usize> {
    starts
        .get(position.line.checked_sub(1)?)
        .map(|line| line + position.column)
}

fn merge_assessments(
    left: RustMaterialAssessment,
    right: RustMaterialAssessment,
) -> RustMaterialAssessment {
    match (left, right) {
        (RustMaterialAssessment::Finding, _) | (_, RustMaterialAssessment::Finding) => {
            RustMaterialAssessment::Finding
        }
        (RustMaterialAssessment::Unknown, _) | (_, RustMaterialAssessment::Unknown) => {
            RustMaterialAssessment::Unknown
        }
        _ => RustMaterialAssessment::Clean,
    }
}

fn split_token_stream_at_commas(tokens: proc_macro2::TokenStream) -> Vec<proc_macro2::TokenStream> {
    let mut entries = Vec::new();
    let mut current = Vec::new();
    for token in tokens {
        let is_comma = match &token {
            proc_macro2::TokenTree::Punct(punct) => punct.as_char() == ',',
            _ => false,
        };
        if is_comma {
            if !current.is_empty() {
                entries.push(current.drain(..).collect());
            }
        } else {
            current.push(token);
        }
    }
    if !current.is_empty() {
        entries.push(current.into_iter().collect());
    }
    entries
}

fn split_json_object_entry(
    tokens: proc_macro2::TokenStream,
) -> Option<(proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    let tokens = tokens.into_iter().collect::<Vec<_>>();
    let mut key = Vec::new();
    let mut value = Vec::new();
    let mut separator_count = 0usize;
    let mut after_separator = false;
    for (index, token) in tokens.iter().cloned().enumerate() {
        let is_colon = match &token {
            proc_macro2::TokenTree::Punct(punct) if punct.as_char() == ':' => {
                let follows_joint_colon = index > 0
                    && match &tokens[index - 1] {
                        proc_macro2::TokenTree::Punct(previous) => {
                            previous.as_char() == ':'
                                && previous.spacing() == proc_macro2::Spacing::Joint
                        }
                        _ => false,
                    };
                punct.spacing() == proc_macro2::Spacing::Alone && !follows_joint_colon
            }
            _ => false,
        };
        if is_colon {
            separator_count += 1;
            if separator_count == 1 {
                after_separator = true;
                continue;
            }
        }
        if after_separator {
            value.push(token);
        } else {
            key.push(token);
        }
    }
    (separator_count == 1 && !key.is_empty() && !value.is_empty())
        .then(|| (key.into_iter().collect(), value.into_iter().collect()))
}

fn assess_json_value_tokens(
    tokens: proc_macro2::TokenStream,
    locals: &BTreeMap<String, String>,
    functions: &BTreeMap<String, String>,
    depth: usize,
) -> RustMaterialAssessment {
    if depth > 32 {
        return RustMaterialAssessment::Unknown;
    }
    let mut token_iter = tokens.clone().into_iter();
    if let (Some(proc_macro2::TokenTree::Group(group)), None) =
        (token_iter.next(), token_iter.next())
    {
        match group.delimiter() {
            proc_macro2::Delimiter::Brace => {
                let mut result = RustMaterialAssessment::Clean;
                for entry in split_token_stream_at_commas(group.stream()) {
                    let Some((key, value)) = split_json_object_entry(entry.clone()) else {
                        if contains_instruction_marker(&entry.to_string())
                            || contains_risky_operation(&entry.to_string())
                        {
                            result = merge_assessments(result, RustMaterialAssessment::Unknown);
                        }
                        continue;
                    };
                    result = merge_assessments(
                        result,
                        assess_json_value_tokens(key, locals, functions, depth + 1),
                    );
                    result = merge_assessments(
                        result,
                        assess_json_value_tokens(value, locals, functions, depth + 1),
                    );
                    if result == RustMaterialAssessment::Finding {
                        return result;
                    }
                }
                result
            }
            proc_macro2::Delimiter::Bracket => split_token_stream_at_commas(group.stream())
                .into_iter()
                .fold(RustMaterialAssessment::Clean, |result, value| {
                    merge_assessments(
                        result,
                        assess_json_value_tokens(value, locals, functions, depth + 1),
                    )
                }),
            proc_macro2::Delimiter::Parenthesis => {
                assess_json_value_tokens(group.stream(), locals, functions, depth + 1)
            }
            proc_macro2::Delimiter::None => RustMaterialAssessment::Unknown,
        }
    } else {
        let Some(expression) = syn::parse2::<syn::Expr>(tokens.clone()).ok() else {
            let text = tokens.to_string();
            return if contains_instruction_marker(&text) || contains_risky_operation(&text) {
                RustMaterialAssessment::Unknown
            } else {
                RustMaterialAssessment::Clean
            };
        };
        if let Some(text) = resolve_rust_text(&expression, locals, functions, 0) {
            if contains_strong_instruction_injection(&text) {
                RustMaterialAssessment::Finding
            } else {
                RustMaterialAssessment::Clean
            }
        } else {
            let text = tokens.to_string();
            if contains_instruction_marker(&text) || contains_risky_operation(&text) {
                RustMaterialAssessment::Unknown
            } else {
                RustMaterialAssessment::Clean
            }
        }
    }
}

fn assess_json_macro(
    tokens: proc_macro2::TokenStream,
    locals: &BTreeMap<String, String>,
    functions: &BTreeMap<String, String>,
) -> RustMaterialAssessment {
    let mut remaining = MAX_RUST_CHANGED_CONTEXT_TOKENS;
    if !count_token_tree(tokens.clone(), 0, &mut remaining) {
        return RustMaterialAssessment::Unknown;
    }
    assess_json_value_tokens(tokens.clone(), locals, functions, 0)
}

fn line_column_le(left: proc_macro2::LineColumn, right: proc_macro2::LineColumn) -> bool {
    (left.line, left.column) <= (right.line, right.column)
}

fn line_column_in_range(
    position: proc_macro2::LineColumn,
    start: proc_macro2::LineColumn,
    end: proc_macro2::LineColumn,
) -> bool {
    line_column_le(start, position) && line_column_le(position, end)
}

fn rust_type_is_path(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(path) => {
            let names = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>();
            path.path.leading_colon.is_some()
                && matches!(names.as_slice(), [root, module, name] if root == "std" && module == "path" && matches!(name.as_str(), "Path" | "PathBuf"))
        }
        syn::Type::Reference(reference) => rust_type_is_path(&reference.elem),
        syn::Type::Paren(paren) => rust_type_is_path(&paren.elem),
        syn::Type::Group(group) => rust_type_is_path(&group.elem),
        _ => false,
    }
}

fn path_constructor(expression: &syn::Expr) -> bool {
    let syn::Expr::Call(call) = expression else {
        return false;
    };
    let syn::Expr::Path(function) = call.func.as_ref() else {
        return false;
    };
    let names = function
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    function.path.leading_colon.is_some()
        && (matches!(names.as_slice(), [root, module, path, method] if root == "std" && module == "path" && path == "PathBuf" && matches!(method.as_str(), "from" | "new"))
            || matches!(names.as_slice(), [root, module, path, method] if root == "std" && module == "path" && path == "Path" && method == "new"))
}

fn is_supported_json_macro(path: &syn::Path) -> bool {
    let names = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    path.leading_colon.is_some()
        && matches!(names.as_slice(), [crate_name, macro_name] if crate_name == "serde_json" && macro_name == "json")
}

fn path_expression_is_proven(expression: &syn::Expr, path_locals: &BTreeSet<String>) -> bool {
    match expression {
        syn::Expr::Path(path) if path.path.segments.len() == 1 => {
            path_locals.contains(&path.path.segments[0].ident.to_string())
        }
        syn::Expr::Call(_) => path_constructor(expression),
        syn::Expr::MethodCall(call) if call.method == "join" => {
            path_expression_is_proven(&call.receiver, path_locals)
        }
        syn::Expr::Paren(paren) => path_expression_is_proven(&paren.expr, path_locals),
        syn::Expr::Group(group) => path_expression_is_proven(&group.expr, path_locals),
        syn::Expr::Reference(reference) => path_expression_is_proven(&reference.expr, path_locals),
        _ => false,
    }
}

fn pattern_path_type(pattern: &syn::Pat) -> bool {
    match pattern {
        syn::Pat::Type(typed) => rust_type_is_path(&typed.ty),
        syn::Pat::Ident(_) => false,
        _ => false,
    }
}

fn pattern_identifier(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(ident) => Some(ident.ident.to_string()),
        syn::Pat::Type(typed) => pattern_identifier(&typed.pat),
        _ => None,
    }
}

fn assess_rust_comments(source: &str, changed_lines: &BTreeSet<usize>) -> RustMaterialAssessment {
    use std::str::FromStr;
    if source.len() > MAX_RUST_SOURCE_BYTES {
        return RustMaterialAssessment::Unknown;
    }
    let Ok(tokens) = proc_macro2::TokenStream::from_str(source) else {
        return RustMaterialAssessment::Unknown;
    };
    let mut starts = vec![0usize];
    for (offset, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(offset + 1);
        }
    }
    let mut literal_mask = vec![false; source.len()];
    fn mask_literals(
        tokens: proc_macro2::TokenStream,
        starts: &[usize],
        mask: &mut [bool],
        depth: usize,
        remaining: &mut usize,
    ) -> bool {
        if depth > 64 {
            return false;
        }
        for token in tokens {
            if *remaining == 0 {
                return false;
            }
            *remaining -= 1;
            match token {
                proc_macro2::TokenTree::Literal(literal) => {
                    let span = literal.span();
                    let (start, end) = (span.start(), span.end());
                    let Some(begin) = source_offset(starts, start) else {
                        return false;
                    };
                    let Some(finish) = source_offset(starts, end) else {
                        return false;
                    };
                    if begin > finish || finish > mask.len() {
                        return false;
                    }
                    mask[begin..finish].fill(true);
                }
                proc_macro2::TokenTree::Group(group)
                    if !mask_literals(group.stream(), starts, mask, depth + 1, remaining) =>
                {
                    return false;
                }
                _ => {}
            }
        }
        true
    }
    let mut remaining = MAX_RUST_LEXICAL_TOKENS;
    if !mask_literals(tokens, &starts, &mut literal_mask, 0, &mut remaining) {
        return RustMaterialAssessment::Unknown;
    }
    let bytes = source.as_bytes();
    let mut cursor = 0usize;
    let mut line = 1usize;
    let mut adjacent_line_comment = String::new();
    let mut previous_line_comment_end = 0usize;
    let mut adjacent_line_comment_start = 1usize;
    let mut adjacent_line_comment_overflow = false;
    while cursor + 1 < bytes.len() {
        if literal_mask[cursor] || bytes[cursor] != b'/' || literal_mask[cursor + 1] {
            line += usize::from(bytes[cursor] == b'\n');
            cursor += 1;
            continue;
        }
        let comment_start = cursor;
        let start_line = line;
        if bytes[cursor + 1] == b'/' {
            cursor += 2;
            while cursor < bytes.len() && bytes[cursor] != b'\n' {
                cursor += 1;
            }
            if previous_line_comment_end + 1 != start_line {
                adjacent_line_comment.clear();
                adjacent_line_comment_overflow = false;
                adjacent_line_comment_start = start_line;
            }
            let current_comment = &source[comment_start..cursor];
            let changed_in_chain = changed_lines
                .range(adjacent_line_comment_start..=start_line)
                .next()
                .is_some();
            if changed_in_chain && contains_strong_instruction_injection(current_comment) {
                return RustMaterialAssessment::Finding;
            }
            let next_len = adjacent_line_comment
                .len()
                .saturating_add(current_comment.len())
                .saturating_add(1);
            if next_len > MAX_RUST_COMMENT_CONTEXT_BYTES {
                adjacent_line_comment_overflow = true;
            } else if !adjacent_line_comment_overflow {
                adjacent_line_comment.push_str(current_comment);
                adjacent_line_comment.push('\n');
            }
            previous_line_comment_end = start_line;
            if adjacent_line_comment_overflow && changed_in_chain {
                return RustMaterialAssessment::Unknown;
            }
        } else if bytes[cursor + 1] == b'*' {
            adjacent_line_comment.clear();
            adjacent_line_comment_overflow = false;
            cursor += 2;
            let mut depth = 1usize;
            while cursor + 1 < bytes.len() && depth > 0 {
                if bytes[cursor] == b'/' && bytes[cursor + 1] == b'*' {
                    depth += 1;
                    cursor += 2;
                } else if bytes[cursor] == b'*' && bytes[cursor + 1] == b'/' {
                    depth -= 1;
                    cursor += 2;
                } else {
                    cursor += 1;
                }
            }
            if depth != 0 {
                return RustMaterialAssessment::Unknown;
            }
        } else {
            cursor += 1;
            continue;
        }
        line += bytes[comment_start..cursor]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count();
        if changed_lines.range(start_line..=line).next().is_some() {
            if contains_strong_instruction_injection(&source[comment_start..cursor])
                || contains_strong_instruction_injection(&adjacent_line_comment)
            {
                return RustMaterialAssessment::Finding;
            }
            if adjacent_line_comment_overflow {
                return RustMaterialAssessment::Unknown;
            }
        }
    }
    RustMaterialAssessment::Clean
}

/// Classify only source units with added-line provenance. The raw added-line
/// stream is useful for non-Rust material, but joining every Rust line loses
/// literal/item boundaries and caused PR #1009's false finding.
pub(super) fn diagnose_rust_material(
    change: &cockpit_git::ChangeEvidence,
) -> RustMaterialDiagnosis {
    use syn::spanned::Spanned;
    use syn::visit::Visit;

    let added_text = change.added_lines.join("\n");
    let Some(source) = change.after_text.as_deref() else {
        struct DirectLiteral(bool);
        impl<'ast> Visit<'ast> for DirectLiteral {
            fn visit_lit(&mut self, literal: &'ast syn::Lit) {
                if rust_literal_text(literal)
                    .as_deref()
                    .is_some_and(contains_strong_instruction_injection)
                {
                    self.0 = true;
                }
            }
        }
        if let Ok(statement) = syn::parse_str::<syn::Stmt>(&added_text) {
            let mut finding = DirectLiteral(false);
            finding.visit_stmt(&statement);
            if finding.0 {
                return RustMaterialDiagnosis::FINDING;
            }
        }
        return if contains_instruction_marker(&added_text) || contains_risky_operation(&added_text)
        {
            RustMaterialDiagnosis::unknown(MaterialUnknownCause::MissingCompleteSource)
        } else {
            RustMaterialDiagnosis::CLEAN
        };
    };
    if source.len() > MAX_RUST_SOURCE_BYTES {
        return RustMaterialDiagnosis::unknown(MaterialUnknownCause::SourceOverBudget);
    }
    let source_lines = source.lines().collect::<Vec<_>>();
    let changed_lines = if change.kind == ChangeKind::Added && change.added_line_origins.is_empty()
    {
        (1..=source_lines.len()).collect::<BTreeSet<_>>()
    } else if change.added_line_origins.len() == change.added_lines.len() {
        let mut lines = BTreeSet::new();
        for (added, origin) in change.added_lines.iter().zip(&change.added_line_origins) {
            if origin.after_line == 0
                || source_lines.get(origin.after_line - 1).copied() != Some(added.as_str())
            {
                return RustMaterialDiagnosis::unknown(MaterialUnknownCause::InvalidProvenance);
            }
            lines.insert(origin.after_line);
        }
        lines
    } else {
        return RustMaterialDiagnosis::unknown(MaterialUnknownCause::InvalidProvenance);
    };
    let Ok(parsed) = syn::parse_file(source) else {
        return RustMaterialDiagnosis::unknown(MaterialUnknownCause::ParseUnavailable);
    };
    match assess_rust_comments(source, &changed_lines) {
        RustMaterialAssessment::Finding => return RustMaterialDiagnosis::FINDING,
        RustMaterialAssessment::Unknown => {
            return RustMaterialDiagnosis::unknown(MaterialUnknownCause::AnalysisIncomplete);
        }
        RustMaterialAssessment::Clean => {}
    }

    #[derive(Default)]
    struct LiteralCollector {
        literals: Vec<(String, proc_macro2::LineColumn, proc_macro2::LineColumn)>,
    }
    impl<'ast> Visit<'ast> for LiteralCollector {
        fn visit_lit_str(&mut self, literal: &'ast syn::LitStr) {
            let span = literal.span();
            self.literals
                .push((literal.value(), span.start(), span.end()));
        }

        fn visit_lit_byte_str(&mut self, literal: &'ast syn::LitByteStr) {
            let span = literal.span();
            self.literals.push((
                String::from_utf8_lossy(&literal.value()).into_owned(),
                span.start(),
                span.end(),
            ));
        }
    }

    fn semantic_items<'a>(
        items: &'a [syn::Item],
        scope: &str,
        out: &mut Vec<(String, &'a syn::Item)>,
        depth: usize,
    ) -> bool {
        if depth > 64 || out.len().saturating_add(items.len()) > 4_096 {
            return false;
        }
        for item in items {
            if let syn::Item::Mod(module) = item
                && let Some((_, children)) = &module.content
            {
                let child_scope = format!("{scope}::{}", module.ident);
                if !semantic_items(children, &child_scope, out, depth + 1) {
                    return false;
                }
                continue;
            }
            out.push((scope.to_owned(), item));
        }
        true
    }
    let mut items = Vec::new();
    if !semantic_items(&parsed.items, "", &mut items, 0) {
        return RustMaterialDiagnosis::unknown(MaterialUnknownCause::AnalysisIncomplete);
    }

    let mut starts = vec![0usize];
    for (offset, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(offset + 1);
        }
    }
    for (_, item) in &items {
        let span = item.span();
        let start = span.start();
        let end = span.end();
        if changed_lines
            .range(start.line..=end.line.max(start.line))
            .next()
            .is_none()
        {
            continue;
        }
        let (Some(begin), Some(finish)) =
            (source_offset(&starts, start), source_offset(&starts, end))
        else {
            return RustMaterialDiagnosis::unknown(MaterialUnknownCause::AnalysisIncomplete);
        };
        if begin > finish || finish > source.len() {
            return RustMaterialDiagnosis::unknown(MaterialUnknownCause::AnalysisIncomplete);
        }
        let Ok(tokens) = source[begin..finish].parse::<proc_macro2::TokenStream>() else {
            return RustMaterialDiagnosis::unknown(MaterialUnknownCause::AnalysisIncomplete);
        };
        let mut remaining = MAX_RUST_CHANGED_CONTEXT_TOKENS;
        if !count_token_tree(tokens, 0, &mut remaining) {
            return RustMaterialDiagnosis::unknown(MaterialUnknownCause::AnalysisIncomplete);
        }
    }

    let mut scoped_functions = BTreeMap::<String, BTreeMap<String, String>>::new();
    for (scope, item) in &items {
        let functions = scoped_functions.entry(scope.clone()).or_default();
        if let syn::Item::Fn(function) = item
            && function.sig.inputs.is_empty()
        {
            let mut local_values = BTreeMap::new();
            let pure_bindings = function.block.stmts
                [..function.block.stmts.len().saturating_sub(1)]
                .iter()
                .all(|statement| {
                    let syn::Stmt::Local(local) = statement else {
                        return false;
                    };
                    let (syn::Pat::Ident(name), Some(initializer)) = (&local.pat, &local.init)
                    else {
                        return false;
                    };
                    if name.mutability.is_some() {
                        return false;
                    }
                    let Some(text) =
                        resolve_rust_text(&initializer.expr, &local_values, functions, 0)
                    else {
                        return false;
                    };
                    local_values.insert(name.ident.to_string(), text);
                    true
                });
            if !pure_bindings {
                continue;
            }
            if let Some(syn::Stmt::Expr(expr, None)) = function.block.stmts.last()
                && let Some(text) = resolve_rust_text(expr, &local_values, functions, 0)
            {
                functions.insert(function.sig.ident.to_string(), text);
            }
        }
    }

    struct CompositionCollector<'a> {
        changed_lines: &'a BTreeSet<usize>,
        functions: &'a BTreeMap<String, String>,
        locals: BTreeMap<String, String>,
        path_locals: BTreeSet<String>,
        safe_path_ranges: Vec<(proc_macro2::LineColumn, proc_macro2::LineColumn)>,
        finding: bool,
        unknown: bool,
        analysis_incomplete: bool,
        ambiguous_composition: bool,
    }
    impl<'ast> Visit<'ast> for CompositionCollector<'_> {
        fn visit_local(&mut self, local: &'ast syn::Local) {
            if let Some(key) = pattern_identifier(&local.pat) {
                let mutable = match &local.pat {
                    syn::Pat::Ident(name) => name.mutability.is_some(),
                    syn::Pat::Type(typed) => {
                        matches!(typed.pat.as_ref(), syn::Pat::Ident(name) if name.mutability.is_some())
                    }
                    _ => true,
                };
                let value = if !mutable {
                    local.init.as_ref().and_then(|initializer| {
                        resolve_rust_text(&initializer.expr, &self.locals, self.functions, 0)
                    })
                } else {
                    None
                };
                if let Some(text) = value {
                    self.locals.insert(key.clone(), text);
                } else {
                    self.locals.remove(&key);
                }

                let typed_path = pattern_path_type(&local.pat);
                let initialized_path = local.init.as_ref().is_some_and(|initializer| {
                    path_expression_is_proven(&initializer.expr, &self.path_locals)
                });
                if !mutable && (typed_path || initialized_path) {
                    self.path_locals.insert(key);
                    if initialized_path && let Some(initializer) = &local.init {
                        let span = initializer.expr.span();
                        self.safe_path_ranges.push((span.start(), span.end()));
                    }
                } else {
                    self.path_locals.remove(&key);
                }
            }
            syn::visit::visit_local(self, local);
        }

        fn visit_block(&mut self, block: &'ast syn::Block) {
            let outer_locals = self.locals.clone();
            let outer_path_locals = self.path_locals.clone();
            syn::visit::visit_block(self, block);
            self.locals = outer_locals;
            self.path_locals = outer_path_locals;
        }

        fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
            let outer_path_locals = self.path_locals.clone();
            for input in &function.sig.inputs {
                if let syn::FnArg::Typed(typed) = input
                    && rust_type_is_path(&typed.ty)
                    && let Some(name) = pattern_identifier(&typed.pat)
                {
                    self.path_locals.insert(name);
                }
            }
            syn::visit::visit_item_fn(self, function);
            self.path_locals = outer_path_locals;
        }

        fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
            let outer_path_locals = self.path_locals.clone();
            for input in &closure.inputs {
                if let syn::Pat::Type(typed) = input
                    && rust_type_is_path(&typed.ty)
                    && let Some(name) = pattern_identifier(&typed.pat)
                {
                    self.path_locals.insert(name);
                }
            }
            syn::visit::visit_expr_closure(self, closure);
            self.path_locals = outer_path_locals;
        }

        fn visit_expr_macro(&mut self, expression: &'ast syn::ExprMacro) {
            let span = expression.span();
            if self
                .changed_lines
                .range(span.start().line..=span.end().line.max(span.start().line))
                .next()
                .is_some()
            {
                if is_supported_json_macro(&expression.mac.path) {
                    let assessment = assess_json_macro(
                        expression.mac.tokens.clone(),
                        &self.locals,
                        self.functions,
                    );
                    match assessment {
                        RustMaterialAssessment::Finding => self.finding = true,
                        RustMaterialAssessment::Unknown => self.analysis_incomplete = true,
                        RustMaterialAssessment::Clean => {}
                    }
                } else {
                    match resolve_rust_text(
                        &syn::Expr::Macro(expression.clone()),
                        &self.locals,
                        self.functions,
                        0,
                    ) {
                        Some(text) if contains_strong_instruction_injection(&text) => {
                            self.finding = true;
                        }
                        Some(_) => {}
                        None => {
                            use syn::parse::Parser;
                            let partial_candidate = syn::punctuated::Punctuated::<
                                syn::Expr,
                                syn::Token![,],
                            >::parse_terminated
                                .parse2(expression.mac.tokens.clone())
                                .ok()
                                .is_some_and(|args| {
                                    args.iter().any(|arg| {
                                        resolve_rust_text(arg, &self.locals, self.functions, 0)
                                            .as_deref()
                                            .is_some_and(|text| {
                                                contains_instruction_marker(text)
                                                    || contains_risky_operation(text)
                                            })
                                    })
                                });
                            let tokens = expression.mac.tokens.to_string();
                            if partial_candidate
                                || contains_instruction_marker(&tokens)
                                || contains_risky_operation(&tokens)
                            {
                                self.unknown = true;
                            }
                        }
                    }
                }
            }
            syn::visit::visit_expr_macro(self, expression);
        }

        fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
            let span = expression.span();
            if self
                .changed_lines
                .range(span.start().line..=span.end().line.max(span.start().line))
                .next()
                .is_some()
            {
                match expression.method.to_string().as_str() {
                    "join"
                        if path_expression_is_proven(&expression.receiver, &self.path_locals) =>
                    {
                        self.safe_path_ranges.push((span.start(), span.end()));
                    }
                    "join" | "push_str" => {
                        self.ambiguous_composition = true;
                    }
                    _ => {}
                }
            }
            syn::visit::visit_expr_method_call(self, expression);
        }

        fn visit_expr_binary(&mut self, expression: &'ast syn::ExprBinary) {
            if matches!(expression.op, syn::BinOp::Add(_)) {
                let span = expression.span();
                if self
                    .changed_lines
                    .range(span.start().line..=span.end().line.max(span.start().line))
                    .next()
                    .is_some()
                {
                    self.ambiguous_composition = true;
                    if resolve_rust_text(
                        &syn::Expr::Binary(expression.clone()),
                        &self.locals,
                        self.functions,
                        0,
                    )
                    .as_deref()
                    .is_some_and(contains_strong_instruction_injection)
                    {
                        self.finding = true;
                    }
                }
            }
            syn::visit::visit_expr_binary(self, expression);
        }
    }

    let mut unresolved_pair = false;
    let mut analysis_incomplete = false;
    let empty_functions = BTreeMap::new();
    for (scope, item) in &items {
        let mut compositions = CompositionCollector {
            changed_lines: &changed_lines,
            functions: scoped_functions.get(scope).unwrap_or(&empty_functions),
            locals: BTreeMap::new(),
            path_locals: BTreeSet::new(),
            safe_path_ranges: Vec::new(),
            finding: false,
            unknown: false,
            analysis_incomplete: false,
            ambiguous_composition: false,
        };
        compositions.visit_item(item);
        if compositions.finding {
            return RustMaterialDiagnosis::FINDING;
        }
        unresolved_pair |= compositions.unknown;
        analysis_incomplete |= compositions.analysis_incomplete;
        let safe_path_ranges = compositions.safe_path_ranges;
        let mut collector = LiteralCollector::default();
        collector.visit_item(item);
        let mut marker_seen = false;
        let mut risk_seen = false;
        let mut any_candidate = false;
        for (text, start, end) in &collector.literals {
            any_candidate |= contains_instruction_marker(text) || contains_risky_operation(text);
            if !changed_lines
                .range(start.line..=end.line.max(start.line))
                .next()
                .is_some()
            {
                continue;
            }
            if contains_strong_instruction_injection(text) {
                return RustMaterialDiagnosis::FINDING;
            }
            if safe_path_ranges.iter().any(|(range_start, range_end)| {
                line_column_in_range(*start, *range_start, *range_end)
                    && line_column_in_range(*end, *range_start, *range_end)
            }) {
                continue;
            }
            marker_seen |= contains_instruction_marker(text);
            risk_seen |= contains_risky_operation(text);
        }
        unresolved_pair |= marker_seen && risk_seen;
        unresolved_pair |= compositions.ambiguous_composition && any_candidate;
    }
    if analysis_incomplete {
        RustMaterialDiagnosis::unknown(MaterialUnknownCause::AnalysisIncomplete)
    } else if unresolved_pair {
        RustMaterialDiagnosis::unknown(MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown)
    } else {
        RustMaterialDiagnosis::CLEAN
    }
}

#[cfg(test)]
mod diagnosis_tests {
    use super::*;
    use cockpit_git::{AddedLineOrigin, ChangeContentState, ChangeEvidence};

    fn changed(source: String) -> ChangeEvidence {
        ChangeEvidence {
            path: "src/material.rs".into(),
            kind: ChangeKind::Modified,
            added_lines: source.lines().map(str::to_owned).collect(),
            added_line_origins: source
                .lines()
                .enumerate()
                .map(|(index, _)| AddedLineOrigin {
                    after_line: index + 1,
                    hunk_index: 0,
                })
                .collect(),
            removed_lines: Vec::new(),
            after_text: Some(source),
            content_state: ChangeContentState::Text,
        }
    }

    #[test]
    fn typed_causes_do_not_turn_missing_or_invalid_evidence_into_reviewable_syntax() {
        let source = r#"
fn material() -> String {
    let mut label = String::from("token");
    label.push_str("ization");
    label
}
"#
        .to_owned();
        let complete = changed(source);
        assert_eq!(
            diagnose_rust_material(&complete).unknown_cause,
            Some(MaterialUnknownCause::ReadableCommittedRustSyntaxUnknown)
        );
        let mut missing = complete.clone();
        missing.after_text = None;
        assert_eq!(
            diagnose_rust_material(&missing).unknown_cause,
            Some(MaterialUnknownCause::MissingCompleteSource)
        );
        let mut invalid_origin = complete;
        invalid_origin.added_line_origins[0].after_line = 999;
        assert_eq!(
            diagnose_rust_material(&invalid_origin).unknown_cause,
            Some(MaterialUnknownCause::InvalidProvenance)
        );
        let malformed = changed("fn material( {".into());
        assert_eq!(
            diagnose_rust_material(&malformed).unknown_cause,
            Some(MaterialUnknownCause::ParseUnavailable)
        );
        let oversized = changed(format!(
            "fn material() {{ /*{}*/ }}",
            "x".repeat(MAX_RUST_SOURCE_BYTES)
        ));
        assert_eq!(
            diagnose_rust_material(&oversized).unknown_cause,
            Some(MaterialUnknownCause::SourceOverBudget)
        );
    }

    #[test]
    fn direct_malicious_literal_remains_finding() {
        let payload = include_str!("../../../tests/conformance/fixtures/repository-prompt-injection/repository/material.txt").trim();
        let change = changed(format!(
            "fn material() {{ let instruction = {payload:?}; }}"
        ));
        let diagnosis = diagnose_rust_material(&change);
        assert_eq!(diagnosis.assessment, RustMaterialAssessment::Finding);
        assert_eq!(diagnosis.unknown_cause, None);
    }
}
