use cockpit_core::DecisionState;
use cockpit_protocol::{HumanDecision, OutcomeClaim, OutcomeState, OutcomeV2};
use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::repository_id;

/// Render the repository Outcome as an explicit, human-facing handoff.
///
/// This is intentionally shared by the CLI and MCP adapters. The OutcomeV2
/// value is produced and validated by outcome_v2; this function only projects
/// it for a conversation. Contract text remains in its original language and
/// no governance decision is inferred or translated.
pub fn render_human_outcome(root: &Path, outcome: &OutcomeV2, language: &str) -> String {
    let language = match language {
        "zh" | "ja" => language,
        _ => "en",
    };
    let historical_kind = outcome.historical_status.as_deref();
    let historical = historical_kind.is_some();
    let superseded = historical_kind == Some("superseded");
    let (marker, status) = if superseded {
        match language {
            "zh" => ("🟡", "历史已替代"),
            "ja" => ("🟡", "履歴として置換済み"),
            _ => ("🟡", "Superseded historical item"),
        }
    } else {
        outcome_status(&outcome.state, outcome.decision_state.as_ref(), language)
    };
    let report = &outcome.human_benefit_report;
    let task_report = outcome.task_outcome_report.as_ref();
    let none = match language {
        "zh" => "无",
        "ja" => "なし",
        _ => "None",
    };
    let (
        title,
        completed,
        problems,
        stops,
        resolved,
        avoided,
        remaining,
        unknowns,
        decisions,
        verification,
        impact,
        next_action,
        evidence,
    ) = match language {
        "zh" => (
            "结果",
            "已完成",
            "发现的问题",
            "触发的停止",
            "已解决的问题",
            "避免的风险",
            "剩余风险",
            "未知项",
            "人工决定",
            "验证",
            "影响",
            "下一步",
            "证据",
        ),
        "ja" => (
            "結果",
            "完了したこと",
            "発見された問題",
            "発動した停止",
            "解決した問題",
            "回避したリスク",
            "残存リスク",
            "不明点",
            "人間の判断",
            "検証",
            "影響",
            "次のアクション",
            "証拠",
        ),
        _ => (
            "Task Result",
            "What was completed",
            "Problems found",
            "Stops triggered",
            "Problems resolved",
            "Risks avoided",
            "Remaining risks",
            "Unknowns",
            "Human decisions",
            "Verification",
            "Impact",
            "Next action",
            "Evidence",
        ),
    };
    let not_ready = match language {
        "zh" => "必需的验证证据尚未生成，不能宣称完成。",
        "ja" => "必須の検証証拠がまだなく、完了とは言えません。",
        _ => "Required verification evidence is not present; completion cannot be claimed.",
    };
    let no_benefit = match language {
        "zh" => "用户可见收益尚未声明。",
        "ja" => "ユーザー向けの効果はまだ宣言されていません。",
        _ => "User-visible benefit has not been declared.",
    };
    let contract_language = match language {
        "zh" => "验收标准（Contract 原文） / Acceptance criteria (Contract language)",
        "ja" => "受入れ基準（Contract 原文） / Acceptance criteria (Contract language)",
        _ => "Acceptance criteria (Contract language)",
    };
    let invalid_evidence = match language {
        "zh" => "验证证据无效或与当前 Work Item / repository 不匹配，已停止。",
        "ja" => {
            "検証 evidence が無効、または Work Item / repository と一致しないため停止しました。"
        }
        _ => {
            "Verification evidence is invalid or does not match this Work Item/repository; stopped."
        }
    };
    let next = if historical {
        if superseded {
            match language {
                "zh" => {
                    "保留原始历史证据；后续工作由 successor Work Item 负责，不要重新解释为当前失败。"
                }
                "ja" => {
                    "元の履歴 evidence を保持し、後続作業は successor Work Item で行います。現在の失敗とは解釈しません。"
                }
                _ => {
                    "Preserve the historical evidence; the successor owns follow-up work, and this is not a current failure."
                }
            }
        } else {
            match language {
                "zh" => {
                    "保留历史证据；如需当前结果，再用当前 Runtime 重新验证，不要将其解释为当前失败。"
                }
                "ja" => {
                    "履歴 evidence を保持し、current result が必要な場合だけ現在の Runtime で再検証してください。現在の失敗とは解釈しません。"
                }
                _ => {
                    "Preserve the historical evidence; reverify with the current Runtime only when a current result is needed, and do not treat it as a current failure."
                }
            }
        }
    } else {
        match (language, &outcome.state) {
            ("zh", OutcomeState::Verified) => {
                "审阅证据后再决定是否继续；🟢 不代表已授权合并或发布。"
            }
            ("zh", _) if outcome.decision_state == Some(DecisionState::Red) => {
                "修复无效证据并重新验证；在此之前保持停止。"
            }
            ("zh", _) => "补齐缺失证据并重新验证；在此之前保持停止。",
            ("ja", OutcomeState::Verified)
                if outcome.decision_state != Some(DecisionState::Red) =>
            {
                "証拠を確認してから続行を判断してください。🟢 はマージやリリースの承認ではありません。"
            }
            ("ja", _) if outcome.decision_state == Some(DecisionState::Red) => {
                "無効な evidence を修復して再検証してください。それまでは停止します。"
            }
            ("ja", _) => {
                "不足している証拠を補い、再検証してください。それまでは停止状態を維持します。"
            }
            (_, OutcomeState::Verified) if outcome.decision_state != Some(DecisionState::Red) => {
                "Review the evidence before deciding whether to proceed; 🟢 does not authorize merge or release."
            }
            (_, _) if outcome.decision_state == Some(DecisionState::Red) => {
                "Repair the invalid evidence and verify again; remain stopped until then."
            }
            (_, _) => "Repair the missing evidence and verify again; remain stopped until then.",
        }
    };
    let failed_gate = if historical {
        None
    } else {
        outcome
            .failed_gate
            .as_deref()
            .or_else(|| task_report.and_then(|report| report.failed_gate.as_deref()))
    };
    let recovery_label = match language {
        "zh" => "失败 gate",
        "ja" => "失敗した gate",
        _ => "Failed gate",
    };
    let recovery_action_label = match language {
        "zh" => "恢复条件",
        "ja" => "復旧条件",
        _ => "Recovery condition",
    };
    let failed_gate_item = failed_gate.map(|gate| format!("{recovery_label}: {gate}"));
    let recovery_action = failed_gate.map(|gate| localized_recovery_action(gate, language));
    let mut unknowns_all = outcome.unknowns.clone();
    unknowns_all.extend(report.unknowns.iter().cloned());
    unknowns_all.sort();
    unknowns_all.dedup();
    let mut remaining_risks = unknowns_all.clone();
    if let Some(task_report) = task_report {
        remaining_risks.extend(claim_texts(&task_report.sections.risks));
        remaining_risks.extend(claim_texts(&task_report.sections.warnings));
        remaining_risks.extend(claim_texts(&task_report.sections.residual_risks));
    }
    remaining_risks.sort();
    remaining_risks.dedup();
    let mut problems_found = task_report
        .map(|report| claim_texts(&report.sections.findings))
        .unwrap_or_default();
    if !historical && outcome.decision_state == Some(DecisionState::Red) {
        problems_found.push(invalid_evidence.to_string());
    } else if !historical && matches!(outcome.state, OutcomeState::NotReady) {
        problems_found.push(not_ready.to_string());
    }
    if let Some(item) = failed_gate_item.as_ref() {
        problems_found.push(item.clone());
    }
    let acceptance_results = human_acceptance_results(&outcome.acceptance_results);
    let localized_summary = if historical {
        if superseded {
            match language {
                "zh" => {
                    "该 Work Item 已作为历史 predecessor 被显式替代；原始证据未被重写，也未按当前 Runtime 重验证。"
                }
                "ja" => {
                    "この Work Item は履歴 predecessor として明示的に置換されました。元の evidence は書き換えず、現在の Runtime では再検証していません。"
                }
                _ => {
                    "This Work Item was explicitly superseded as a historical predecessor; original evidence was not rewritten or revalidated under the current Runtime."
                }
            }
        } else {
            match language {
                "zh" => "该 Work Item 的历史验证证据未按当前 Runtime 重新验证；这不是当前失败。",
                "ja" => {
                    "この Work Item の履歴 verification evidence は現在の Runtime で再検証されていません。現在の失敗ではありません。"
                }
                _ => {
                    "This Work Item has historical verification evidence that was not revalidated under the current Runtime; it is not a current failure."
                }
            }
        }
    } else {
        localized_outcome_summary(&outcome.state, outcome.decision_state.as_ref(), language)
    };
    let mut completed_items = vec![localized_summary.to_string()];
    if let Some(report) = task_report {
        completed_items.extend(claim_texts(&report.sections.delivered_changes));
    }
    if completed_items.len() == 1 && !acceptance_results.is_empty() {
        completed_items.push(contract_language.to_string());
        completed_items.extend(acceptance_results.clone());
    }
    if !acceptance_results.is_empty()
        && !completed_items.iter().any(|item| item == contract_language)
    {
        completed_items.push(contract_language.to_string());
    }
    let impact_items = if report.user_visible_changes.is_empty() && report.affected_users.is_empty()
    {
        vec![no_benefit.to_string()]
    } else {
        report
            .user_visible_changes
            .iter()
            .chain(report.affected_users.iter())
            .cloned()
            .collect()
    };
    let verification_items = if outcome.evidence_refs.is_empty() {
        vec![status.to_string()]
    } else {
        outcome
            .evidence_refs
            .iter()
            .map(|reference| format!("{status}: {reference}"))
            .collect()
    };
    let decision_items = match load_human_decision(root, &outcome.work_item_id) {
        HumanDecisionProjection::Missing => Vec::new(),
        HumanDecisionProjection::Valid(decision) => {
            vec![render_human_decision(&decision, language, none)]
        }
        HumanDecisionProjection::Invalid(reason) => {
            let label = match language {
                "zh" => "未知：结构化人工决定记录无效",
                "ja" => "不明：構造化された人間の判断記録が無効です",
                _ => "Unknown: structured human decision record is invalid",
            };
            vec![format!("{label} ({reason})")]
        }
    };
    let mut stop_items = task_report
        .map(|report| claim_texts(&report.sections.forced_stops))
        .unwrap_or_default();
    if historical {
        stop_items.clear();
    } else if stop_items.is_empty() && outcome.decision_state == Some(DecisionState::Red) {
        stop_items = vec![invalid_evidence.to_string()];
    } else if matches!(
        outcome.state,
        OutcomeState::NotReady | OutcomeState::Unknown
    ) {
        stop_items = vec![not_ready.to_string()];
    }
    if let Some(action) = recovery_action.as_ref() {
        stop_items.push(format!("{recovery_action_label}: {action}"));
    }
    let resolved_items = task_report
        .map(|report| claim_texts(&report.sections.resolutions))
        .unwrap_or_default();
    let avoided_items = task_report
        .map(|report| claim_texts(&report.sections.avoided_impact))
        .unwrap_or_default();
    let header = format!(
        "Outcome: {marker} {status} — {}\n{title}",
        outcome.work_item_id
    );
    format!(
        "{header}\n\n{completed}\n{}\n\n{problems}\n{}\n\n{stops}\n{}\n\n{resolved}\n{}\n\n{avoided}\n{}\n\n{remaining}\n{}\n\n{unknowns}\n{}\n\n{decisions}\n{}\n\n{verification}\n{}\n\n{impact}\n{}\n\n{next_action}\n- {next}\n\n{evidence}\n{}",
        bullet_lines(&completed_items, none),
        bullet_lines(&problems_found, none),
        bullet_lines(&stop_items, none),
        bullet_lines(&resolved_items, none),
        bullet_lines(&avoided_items, none),
        bullet_lines(&remaining_risks, none),
        bullet_lines(&unknowns_all, none),
        bullet_lines(&decision_items, none),
        bullet_lines(&verification_items, none),
        bullet_lines(&impact_items, none),
        bullet_lines(&outcome.evidence_refs, none),
    )
}

fn localized_recovery_action(gate: &str, language: &str) -> String {
    match (language, gate) {
        ("zh", "finish.verification") => {
            "记录当前有效的验证证据，重新 preflight，然后重试 finish。".into()
        }
        ("ja", "finish.verification") => {
            "現在の有効な検証 evidence を記録し、preflight を再実行して finish を再試行します。"
                .into()
        }
        (_, "finish.verification") => {
            "Record valid current verification evidence, rerun preflight, and retry finish.".into()
        }
        ("zh", "finish.preflight") => "记录新的非红色 preflight 结果，然后重试 finish。".into(),
        ("ja", "finish.preflight") => {
            "新しい非 red の preflight 結果を記録してから finish を再試行します。".into()
        }
        (_, "finish.preflight") => {
            "Record a fresh non-red preflight result, then retry finish.".into()
        }
        ("zh", _) => "修复失败的治理条件，完成新的检查后重试 finish。".into(),
        ("ja", _) => {
            "失敗した governance 条件を修正し、新しい check 後に finish を再試行します。".into()
        }
        (_, _) => {
            "Repair the failed governance condition, complete fresh checks, and retry finish."
                .into()
        }
    }
}

fn claim_texts(claims: &[OutcomeClaim]) -> Vec<String> {
    claims
        .iter()
        .filter(|claim| !claim.text.trim().is_empty())
        .map(|claim| {
            if claim.inference {
                format!("Inference: {}", claim.text)
            } else {
                claim.text.clone()
            }
        })
        .collect()
}

fn bullet_lines(items: &[String], none: &str) -> String {
    if items.is_empty() {
        format!("- {none}")
    } else {
        items
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn human_acceptance_results(results: &[String]) -> Vec<String> {
    let mut normalized: Vec<String> = Vec::new();
    for result in results {
        if result.starts_with(char::is_whitespace) && !normalized.is_empty() {
            if let Some(previous) = normalized.last_mut() {
                previous.push_str(result);
            }
        } else {
            normalized.push(result.clone());
        }
    }
    normalized
}

fn outcome_status(
    state: &OutcomeState,
    decision_state: Option<&DecisionState>,
    language: &str,
) -> (&'static str, &'static str) {
    let decision_state = decision_state.unwrap_or(match state {
        OutcomeState::Verified => &DecisionState::Green,
        OutcomeState::Partial | OutcomeState::NotReady | OutcomeState::Unknown => {
            &DecisionState::Yellow
        }
    });
    match (language, decision_state) {
        ("zh", DecisionState::Green) => ("🟢", "成功"),
        ("zh", DecisionState::Yellow) => ("🟡", "需要关注"),
        ("zh", DecisionState::Red) => ("🔴", "停止"),
        ("ja", DecisionState::Green) => ("🟢", "成功"),
        ("ja", DecisionState::Yellow) => ("🟡", "要確認"),
        ("ja", DecisionState::Red) => ("🔴", "停止"),
        (_, DecisionState::Green) => ("🟢", "Success"),
        (_, DecisionState::Yellow) => ("🟡", "Needs attention"),
        (_, DecisionState::Red) => ("🔴", "Stop"),
    }
}

fn localized_outcome_summary(
    state: &OutcomeState,
    decision_state: Option<&DecisionState>,
    language: &str,
) -> &'static str {
    let red = decision_state == Some(&DecisionState::Red);
    match (language, red, state) {
        ("zh", false, OutcomeState::Verified) => "验证证据有效；用户可见收益尚未声明。",
        ("zh", false, OutcomeState::NotReady) => "未找到或无法使用验证证据；结果尚未准备好。",
        ("zh", false, OutcomeState::Partial) => "验证证据部分有效；结果仍需关注。",
        ("zh", _, OutcomeState::Unknown) | ("zh", true, _) => {
            "验证证据无法确认或与当前上下文不一致；结果已停止。"
        }
        ("ja", false, OutcomeState::Verified) => {
            "検証 evidence は有効ですが、ユーザー向けの効果はまだ宣言されていません。"
        }
        ("ja", false, OutcomeState::NotReady) => {
            "検証 evidence がないか使用できず、結果はまだ準備できていません。"
        }
        ("ja", false, OutcomeState::Partial) => {
            "検証 evidence は一部有効ですが、結果にはまだ確認が必要です。"
        }
        ("ja", _, OutcomeState::Unknown) | ("ja", true, _) => {
            "検証 evidence を確認できないか現在の context と一致しないため、停止しました。"
        }
        (_, false, OutcomeState::Verified) => {
            "Verification evidence is valid; user-visible benefit remains explicitly unknown."
        }
        (_, false, OutcomeState::NotReady) => {
            "No usable verification evidence is present; the outcome is not ready."
        }
        (_, false, OutcomeState::Partial) => {
            "Verification evidence is partially valid; the outcome still needs attention."
        }
        (_, _, OutcomeState::Unknown) | (_, true, _) => {
            "Verification evidence could not be confirmed or does not match this context; the outcome is stopped."
        }
    }
}

#[derive(Debug)]
enum HumanDecisionProjection {
    Missing,
    Valid(HumanDecision),
    Invalid(&'static str),
}

fn load_human_decision(root: &Path, work_item_id: &str) -> HumanDecisionProjection {
    let path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.close.json"));
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return HumanDecisionProjection::Missing;
    };
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return HumanDecisionProjection::Invalid("decision record is not a regular file");
    }
    let Ok(bytes) = fs::read(&path) else {
        return HumanDecisionProjection::Invalid("decision record cannot be read");
    };
    let Ok(record): Result<Value, _> = serde_json::from_slice(&bytes) else {
        return HumanDecisionProjection::Invalid("decision record is not valid JSON");
    };
    if record.get("workItemId").and_then(Value::as_str) != Some(work_item_id) {
        return HumanDecisionProjection::Invalid(
            "decision record Work Item binding is missing or mismatched",
        );
    }
    let expected_repository_id = repository_id(root).to_string();
    if record.get("repositoryId").and_then(Value::as_str) != Some(expected_repository_id.as_str()) {
        return HumanDecisionProjection::Invalid(
            "decision record repository binding is missing or mismatched",
        );
    }
    if record.get("state").and_then(Value::as_str) != Some("closed") {
        return HumanDecisionProjection::Invalid("decision record state is not closed");
    }
    if record.get("decisionState").and_then(Value::as_str) != Some("confirmed") {
        return HumanDecisionProjection::Invalid("decision record is not confirmed");
    }
    let Some(structured) = record.get("structuredDecision").cloned() else {
        return HumanDecisionProjection::Invalid("structured decision is missing");
    };
    let Ok(decision): Result<HumanDecision, _> = serde_json::from_value(structured) else {
        return HumanDecisionProjection::Invalid(
            "structured decision fields are incomplete or unknown",
        );
    };
    for value in [
        decision.decision.as_str(),
        decision.actor.as_str(),
        decision.authority_source.as_str(),
        decision.reason.as_str(),
        decision.decided_at.as_str(),
    ] {
        if value.trim().is_empty() {
            return HumanDecisionProjection::Invalid(
                "structured decision contains an empty required field",
            );
        }
    }
    if record.get("humanDecision").and_then(Value::as_str) != Some(decision.decision.as_str()) {
        return HumanDecisionProjection::Invalid(
            "decision record summary does not match structured decision",
        );
    }
    HumanDecisionProjection::Valid(decision)
}

fn render_human_decision(decision: &HumanDecision, language: &str, none: &str) -> String {
    let (
        decision_label,
        actor_label,
        authority_label,
        reason_label,
        evidence_label,
        policy_label,
        decided_label,
        resume_label,
    ) = match language {
        "zh" => (
            "决定",
            "执行人",
            "授权来源",
            "理由",
            "证据引用",
            "策略引用",
            "决定时间",
            "恢复条件",
        ),
        "ja" => (
            "判断",
            "実行者",
            "権限の出所",
            "理由",
            "evidence 参照",
            "policy 参照",
            "判断日時",
            "再開条件",
        ),
        _ => (
            "Decision",
            "Actor",
            "Authority source",
            "Reason",
            "Evidence refs",
            "Policy refs",
            "Decided at",
            "Resume condition",
        ),
    };
    let evidence_refs = if decision.evidence_refs.is_empty() {
        none.to_string()
    } else {
        decision.evidence_refs.join(", ")
    };
    let policy_refs = if decision.policy_refs.is_empty() {
        none.to_string()
    } else {
        decision.policy_refs.join(", ")
    };
    let resume_condition = decision.resume_condition.as_deref().unwrap_or(none);
    format!(
        "{decision_label}: {}\n  {actor_label}: {}\n  {authority_label}: {}\n  {reason_label}: {}\n  {evidence_label}: {evidence_refs}\n  {policy_label}: {policy_refs}\n  {decided_label}: {}\n  {resume_label}: {resume_condition}",
        decision.decision,
        decision.actor,
        decision.authority_source,
        decision.reason,
        decision.decided_at,
    )
}
