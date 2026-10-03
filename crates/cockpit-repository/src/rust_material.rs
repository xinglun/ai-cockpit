use std::collections::{BTreeMap, BTreeSet};

use cockpit_git::ChangeKind;

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

fn assess_rust_comments(source: &str, changed_lines: &BTreeSet<usize>) -> RustMaterialAssessment {
    use std::str::FromStr;
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
                    let Some(begin) = starts.get(start.line - 1).map(|line| line + start.column)
                    else {
                        return false;
                    };
                    let Some(finish) = starts.get(end.line - 1).map(|line| line + end.column)
                    else {
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
    if !mask_literals(tokens, &starts, &mut literal_mask, 0, &mut 16_384) {
        return RustMaterialAssessment::Unknown;
    }
    let bytes = source.as_bytes();
    let mut cursor = 0usize;
    let mut line = 1usize;
    let mut adjacent_line_comment = String::new();
    let mut previous_line_comment_end = 0usize;
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
            }
            adjacent_line_comment.push_str(&source[comment_start..cursor]);
            adjacent_line_comment.push('\n');
            previous_line_comment_end = start_line;
        } else if bytes[cursor + 1] == b'*' {
            adjacent_line_comment.clear();
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
        if changed_lines.range(start_line..=line).next().is_some()
            && (contains_strong_instruction_injection(&source[comment_start..cursor])
                || contains_strong_instruction_injection(&adjacent_line_comment))
        {
            return RustMaterialAssessment::Finding;
        }
    }
    RustMaterialAssessment::Clean
}

/// Classify only source units with added-line provenance. The raw added-line
/// stream is useful for non-Rust material, but joining every Rust line loses
/// literal/item boundaries and caused PR #1009's false finding.
pub(super) fn assess_rust_material(change: &cockpit_git::ChangeEvidence) -> RustMaterialAssessment {
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
                return RustMaterialAssessment::Finding;
            }
        }
        return if contains_instruction_marker(&added_text) || contains_risky_operation(&added_text)
        {
            RustMaterialAssessment::Unknown
        } else {
            RustMaterialAssessment::Clean
        };
    };
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
                return RustMaterialAssessment::Unknown;
            }
            lines.insert(origin.after_line);
        }
        lines
    } else {
        return RustMaterialAssessment::Unknown;
    };
    let Ok(parsed) = syn::parse_file(source) else {
        return RustMaterialAssessment::Unknown;
    };
    match assess_rust_comments(source, &changed_lines) {
        RustMaterialAssessment::Finding => return RustMaterialAssessment::Finding,
        RustMaterialAssessment::Unknown => return RustMaterialAssessment::Unknown,
        RustMaterialAssessment::Clean => {}
    }

    #[derive(Default)]
    struct LiteralCollector {
        literals: Vec<(String, usize, usize)>,
    }
    impl<'ast> Visit<'ast> for LiteralCollector {
        fn visit_lit_str(&mut self, literal: &'ast syn::LitStr) {
            let span = literal.span();
            self.literals
                .push((literal.value(), span.start().line, span.end().line));
        }

        fn visit_lit_byte_str(&mut self, literal: &'ast syn::LitByteStr) {
            let span = literal.span();
            self.literals.push((
                String::from_utf8_lossy(&literal.value()).into_owned(),
                span.start().line,
                span.end().line,
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
        return RustMaterialAssessment::Unknown;
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
        finding: bool,
        unknown: bool,
        ambiguous_composition: bool,
    }
    impl<'ast> Visit<'ast> for CompositionCollector<'_> {
        fn visit_local(&mut self, local: &'ast syn::Local) {
            if let syn::Pat::Ident(name) = &local.pat {
                let key = name.ident.to_string();
                let value = if name.mutability.is_none() {
                    local.init.as_ref().and_then(|initializer| {
                        resolve_rust_text(&initializer.expr, &self.locals, self.functions, 0)
                    })
                } else {
                    None
                };
                if let Some(text) = value {
                    self.locals.insert(key, text);
                } else {
                    self.locals.remove(&key);
                }
            }
            syn::visit::visit_local(self, local);
        }

        fn visit_block(&mut self, block: &'ast syn::Block) {
            let outer_locals = self.locals.clone();
            syn::visit::visit_block(self, block);
            self.locals = outer_locals;
        }

        fn visit_expr_macro(&mut self, expression: &'ast syn::ExprMacro) {
            let span = expression.span();
            if self
                .changed_lines
                .range(span.start().line..=span.end().line.max(span.start().line))
                .next()
                .is_some()
            {
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
            syn::visit::visit_expr_macro(self, expression);
        }

        fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
            if matches!(expression.method.to_string().as_str(), "push_str" | "join") {
                let span = expression.span();
                if self
                    .changed_lines
                    .range(span.start().line..=span.end().line.max(span.start().line))
                    .next()
                    .is_some()
                {
                    self.ambiguous_composition = true;
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
    let empty_functions = BTreeMap::new();
    for (scope, item) in &items {
        let mut compositions = CompositionCollector {
            changed_lines: &changed_lines,
            functions: scoped_functions.get(scope).unwrap_or(&empty_functions),
            locals: BTreeMap::new(),
            finding: false,
            unknown: false,
            ambiguous_composition: false,
        };
        compositions.visit_item(item);
        if compositions.finding {
            return RustMaterialAssessment::Finding;
        }
        unresolved_pair |= compositions.unknown;
        let mut collector = LiteralCollector::default();
        collector.visit_item(item);
        let mut marker_seen = false;
        let mut risk_seen = false;
        let mut any_candidate = false;
        for (text, start, end) in &collector.literals {
            any_candidate |= contains_instruction_marker(text) || contains_risky_operation(text);
            if !changed_lines
                .range(*start..=(*end).max(*start))
                .next()
                .is_some()
            {
                continue;
            }
            if contains_strong_instruction_injection(text) {
                return RustMaterialAssessment::Finding;
            }
            marker_seen |= contains_instruction_marker(text);
            risk_seen |= contains_risky_operation(text);
        }
        unresolved_pair |= marker_seen && risk_seen;
        unresolved_pair |= compositions.ambiguous_composition && any_candidate;
    }
    if unresolved_pair {
        RustMaterialAssessment::Unknown
    } else {
        RustMaterialAssessment::Clean
    }
}
