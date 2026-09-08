use cockpit_core::DecisionState;
use cockpit_protocol::{HumanDecision, OutcomeClaim, OutcomeState, OutcomeV2, TaskOutcomeReport};
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
    let not_recorded = match language {
        "zh" => "未记录",
        "ja" => "未記録",
        _ => "Not recorded",
    };
    let status_labels = match language {
        "zh" => ("验证状态", "生命周期状态", "人工决定状态", "治理信号"),
        "ja" => (
            "検証状態",
            "ライフサイクル状態",
            "人間の判断状態",
            "ガバナンスシグナル",
        ),
        _ => (
            "Verification",
            "Lifecycle",
            "Human decision",
            "Governance signal",
        ),
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
    let decision_projection = load_human_decision(root, &outcome.work_item_id);
    let lifecycle = localized_lifecycle_status(
        lifecycle_status(root, outcome, historical, superseded),
        language,
    );
    let human_decision_status = localized_human_decision_status(&decision_projection, language);
    let governance_signal = localized_governance_signal(outcome.decision_state.as_ref(), language);
    let mut next = if historical {
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
    let archived_contract = root
        .join(".ai/work-items/archive")
        .join(format!("{}.contract.json", outcome.work_item_id));
    let archived_unclosed = !historical
        && archived_contract.is_file()
        && !crate::close_decision_is_valid_for_status(
            root,
            &outcome.work_item_id,
            &outcome.repository_id,
        );
    if archived_unclosed {
        let finalization_pending = outcome
            .unknowns
            .iter()
            .any(|unknown| unknown == "resource_finalization_pending");
        next = match (language, finalization_pending) {
            ("zh", true) => {
                "先完成 provider finalization：清理并删除该 Work Item 的精确分支和工作树，记录 finalization receipt，运行 finalize-verify，随后 close。"
            }
            ("ja", true) => {
                "まず provider finalization を完了します。対象 Work Item の正確な branch と worktree を cleanup/delete し、finalization receipt を記録して finalize-verify を実行し、その後 close してください。"
            }
            (_, true) => {
                "Complete provider finalization first: clean up and delete the exact Work Item branch and worktree, record the finalization receipt, run finalize-verify, then close."
            }
            ("zh", false) => {
                "审阅归档证据后记录明确的人工 close 决定；完成 close 前不得开始下一个 Work Item。"
            }
            ("ja", false) => {
                "アーカイブ evidence を確認して明示的な人間の close 判断を記録してください。close 完了前に次の Work Item を開始しないでください。"
            }
            (_, false) => {
                "Review the archive evidence and record the explicit human close decision; do not start another Work Item until close is complete."
            }
        };
    }
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
    let mut remaining_risks = Vec::new();
    if let Some(task_report) = task_report {
        remaining_risks.extend(calibrated_risk_claims(
            &task_report.sections.risks,
            language,
        ));
        remaining_risks.extend(calibrated_risk_claims(
            &task_report.sections.warnings,
            language,
        ));
        remaining_risks.extend(calibrated_risk_claims(
            &task_report.sections.residual_risks,
            language,
        ));
        remaining_risks.extend(
            task_report
                .sections
                .limitations
                .iter()
                .map(|claim| format_claim(claim, language, "limitation")),
        );
    }
    if remaining_risks.is_empty() {
        remaining_risks.push(localized_risk_absence(task_report, language));
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
                "zh" => "历史验证证据未按当前 Runtime 重新验证；这不是当前失败。",
                "ja" => {
                    "履歴の検証 evidence は現在の Runtime で再検証されていません。現在の失敗ではありません。"
                }
                _ => {
                    "Historical verification evidence was not revalidated under the current Runtime; this is not a current failure."
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
    let decision_items = match &decision_projection {
        HumanDecisionProjection::Missing => vec![human_decision_status.clone()],
        HumanDecisionProjection::Valid {
            decision,
            assurance,
        } => {
            vec![render_human_decision(
                decision,
                assurance.as_deref(),
                language,
                not_recorded,
            )]
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
        "Outcome: {marker} {status} — {}\n{title}\n- {}: {}\n- {}: {}\n- {}: {}\n- {}: {}",
        outcome.work_item_id,
        status_labels.0,
        status,
        status_labels.1,
        lifecycle,
        status_labels.2,
        human_decision_status,
        status_labels.3,
        governance_signal,
    );
    format!(
        "{header}\n\n{completed}\n{}\n\n{problems}\n{}\n\n{stops}\n{}\n\n{resolved}\n{}\n\n{avoided}\n{}\n\n{remaining}\n{}\n\n{unknowns}\n{}\n\n{decisions}\n{}\n\n{verification}\n{}\n\n{impact}\n{}\n\n{next_action}\n- {next}\n\n{evidence}\n{}",
        bullet_lines(&completed_items, not_recorded),
        bullet_lines(&problems_found, not_recorded),
        bullet_lines(&stop_items, not_recorded),
        bullet_lines(&resolved_items, not_recorded),
        bullet_lines(&avoided_items, not_recorded),
        bullet_lines(&remaining_risks, not_recorded),
        bullet_lines(&unknowns_all, not_recorded),
        bullet_lines(&decision_items, not_recorded),
        bullet_lines(&verification_items, not_recorded),
        bullet_lines(&impact_items, not_recorded),
        bullet_lines(&outcome.evidence_refs, not_recorded),
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

fn format_claim(claim: &OutcomeClaim, language: &str, kind: &str) -> String {
    let text = if claim.inference {
        format!("Inference: {}", claim.text)
    } else {
        claim.text.clone()
    };
    if kind == "limitation" {
        match language {
            "zh" => format!("限制：{text}"),
            "ja" => format!("制限：{text}"),
            _ => format!("Limitation: {text}"),
        }
    } else {
        text
    }
}

fn calibrated_risk_claims(claims: &[OutcomeClaim], language: &str) -> Vec<String> {
    claims
        .iter()
        .filter(|claim| !claim.text.trim().is_empty())
        .map(|claim| {
            let lower = claim.text.to_ascii_lowercase();
            if lower.contains("test")
                && lower.contains("weak")
                && (lower.contains("no ") || lower.contains("not "))
            {
                let scope = if claim.evidence_refs.is_empty() {
                    match language {
                        "zh" => "引用的检查范围未记录".into(),
                        "ja" => "参照された検査範囲は未記録".into(),
                        _ => "the referenced check scope is not recorded".into(),
                    }
                } else {
                    claim.evidence_refs.join(", ")
                };
                match language {
                    "zh" => format!(
                        "测试弱化扫描：在所引用的检查范围（{scope}）内未触发规则；这不等于证明测试未被弱化。"
                    ),
                    "ja" => format!(
                        "テスト弱化スキャン：参照された検査範囲（{scope}）ではルールは発動していません。テストが弱化されていないことの証明ではありません。"
                    ),
                    _ => format!(
                        "Test-weakening scan: no trigger was recorded for the referenced check scope ({scope}); this does not prove that tests were not weakened."
                    ),
                }
            } else {
                format_claim(claim, language, "risk")
            }
        })
        .collect()
}

fn localized_risk_absence(task_report: Option<&TaskOutcomeReport>, language: &str) -> String {
    if task_report.is_none() {
        return match language {
            "zh" => "风险评估：未记录。".into(),
            "ja" => "リスク評価：未記録。".into(),
            _ => "Risk findings: Not recorded.".into(),
        };
    }
    let has_evidence = task_report.is_some_and(|report| {
        report
            .bindings
            .evidence_refs
            .iter()
            .any(|reference| !reference.trim().is_empty())
    });
    if !has_evidence {
        return match language {
            "zh" => "风险评估：未评估；没有可用的证据范围。".into(),
            "ja" => "リスク評価：未評価。利用可能な evidence 範囲がありません。".into(),
            _ => "Risk findings: Not assessed; no usable evidence scope is recorded.".into(),
        };
    }
    match language {
        "zh" => "风险评估：未记录；现有证据未声明已完成风险检查。".into(),
        "ja" => "リスク評価：未記録。現存する evidence はリスク検査の完了を宣言していません。".into(),
        _ => "Risk findings: Not recorded; the available evidence does not declare that a risk check was completed.".into(),
    }
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
    let marker = match decision_state {
        DecisionState::Green => "🟢",
        DecisionState::Yellow => "🟡",
        DecisionState::Red => "🔴",
    };
    let label = match (language, state) {
        ("zh", OutcomeState::Verified) => "已声明的验证通过",
        ("zh", OutcomeState::Partial) => "部分验证",
        ("zh", OutcomeState::NotReady) => "验证尚未就绪",
        ("zh", OutcomeState::Unknown) => "验证状态未知",
        ("ja", OutcomeState::Verified) => "宣言された検証済み",
        ("ja", OutcomeState::Partial) => "検証は一部のみ",
        ("ja", OutcomeState::NotReady) => "検証未準備",
        ("ja", OutcomeState::Unknown) => "検証状態不明",
        (_, OutcomeState::Verified) => "Declared verification passed",
        (_, OutcomeState::Partial) => "Partial verification",
        (_, OutcomeState::NotReady) => "Verification not ready",
        (_, OutcomeState::Unknown) => "Verification status unknown",
    };
    (marker, label)
}

fn lifecycle_status(
    root: &Path,
    outcome: &OutcomeV2,
    historical: bool,
    superseded: bool,
) -> String {
    if historical {
        return if superseded {
            "historical_superseded".into()
        } else {
            "historical".into()
        };
    }
    let archive_contract = root
        .join(".ai/work-items/archive")
        .join(format!("{}.contract.json", outcome.work_item_id));
    if archive_contract.is_file() {
        if crate::close_decision_is_valid_for_status(
            root,
            &outcome.work_item_id,
            &outcome.repository_id,
        ) {
            return "closed".into();
        }
        return "archived".into();
    }
    let summary_path = root
        .join(".ai/work-items/active")
        .join(format!("{}.summary.json", outcome.work_item_id));
    fs::read(&summary_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .and_then(|summary| {
            summary
                .get("state")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "unknown".into())
}

fn localized_lifecycle_status(status: String, language: &str) -> String {
    match (language, status.as_str()) {
        ("zh", "implementation_active") => "实施中".into(),
        ("zh", "checkpointed") => "已建立检查点".into(),
        ("zh", "finish_ready") => "已具备 finish 条件".into(),
        ("zh", "blocked") => "已阻断".into(),
        ("zh", "archived") => "已归档".into(),
        ("zh", "closed") => "已关闭".into(),
        ("zh", "historical") => "历史记录".into(),
        ("zh", "historical_superseded") => "历史记录（已替代）".into(),
        ("zh", _) => "状态未知".into(),
        ("ja", "implementation_active") => "実装中".into(),
        ("ja", "checkpointed") => "チェックポイント済み".into(),
        ("ja", "finish_ready") => "finish 準備完了".into(),
        ("ja", "blocked") => "ブロック中".into(),
        ("ja", "archived") => "アーカイブ済み".into(),
        ("ja", "closed") => "クローズ済み".into(),
        ("ja", "historical") => "履歴".into(),
        ("ja", "historical_superseded") => "履歴（置換済み）".into(),
        ("ja", _) => "状態不明".into(),
        (_, "implementation_active") => "Implementation active".into(),
        (_, "checkpointed") => "Checkpointed".into(),
        (_, "finish_ready") => "Ready to finish".into(),
        (_, "blocked") => "Blocked".into(),
        (_, "archived") => "Archived".into(),
        (_, "closed") => "Closed".into(),
        (_, "historical") => "Historical".into(),
        (_, "historical_superseded") => "Superseded historical record".into(),
        (_, _) => "Unknown".into(),
    }
}

fn localized_human_decision_status(decision: &HumanDecisionProjection, language: &str) -> String {
    match decision {
        HumanDecisionProjection::Missing => match language {
            "zh" => "未记录".into(),
            "ja" => "未記録".into(),
            _ => "Not recorded".into(),
        },
        HumanDecisionProjection::Valid { decision, .. } => match language {
            "zh" => format!("已记录：{}", decision.decision),
            "ja" => format!("記録済み：{}", decision.decision),
            _ => format!("Recorded: {}", decision.decision),
        },
        HumanDecisionProjection::Invalid(_) => match language {
            "zh" => "未知：结构化人工决定记录无效".into(),
            "ja" => "不明：構造化された人間の判断記録が無効".into(),
            _ => "Unknown: structured human decision record is invalid".into(),
        },
    }
}

fn localized_governance_signal(decision_state: Option<&DecisionState>, language: &str) -> String {
    match (language, decision_state) {
        ("zh", Some(DecisionState::Green)) => "绿色；不是人工批准".into(),
        ("zh", Some(DecisionState::Yellow)) => "黄色；需要关注，不是人工批准".into(),
        ("zh", Some(DecisionState::Red)) => "红色；必须停止，不是人工批准".into(),
        ("zh", None) => "未知".into(),
        ("ja", Some(DecisionState::Green)) => "緑。人間の承認ではありません".into(),
        ("ja", Some(DecisionState::Yellow)) => "黄。人間の承認ではありません".into(),
        ("ja", Some(DecisionState::Red)) => "赤。停止が必要で、人間の承認ではありません".into(),
        ("ja", None) => "不明".into(),
        (_, Some(DecisionState::Green)) => "Green; not a human approval".into(),
        (_, Some(DecisionState::Yellow)) => "Yellow; not a human approval".into(),
        (_, Some(DecisionState::Red)) => "Red; stop required, not a human approval".into(),
        (_, None) => "Unknown".into(),
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
    Valid {
        decision: HumanDecision,
        assurance: Option<String>,
    },
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
    let assurance = record
        .get("assurance")
        .or_else(|| structured.get("assurance"))
        .and_then(Value::as_str)
        .filter(|value| {
            matches!(
                *value,
                "self_declared"
                    | "repository_verified"
                    | "provider_verified"
                    | "enterprise_verified"
            )
        })
        .map(str::to_owned);
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
    HumanDecisionProjection::Valid {
        decision,
        assurance,
    }
}

fn render_human_decision(
    decision: &HumanDecision,
    assurance: Option<&str>,
    language: &str,
    not_recorded: &str,
) -> String {
    let (
        decision_label,
        actor_label,
        authority_label,
        assurance_label,
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
            "保证级别",
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
            "保証レベル",
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
            "Assurance level",
            "Reason",
            "Evidence refs",
            "Policy refs",
            "Decided at",
            "Resume condition",
        ),
    };
    let evidence_refs = if decision.evidence_refs.is_empty() {
        not_recorded.to_string()
    } else {
        decision.evidence_refs.join(", ")
    };
    let policy_refs = if decision.policy_refs.is_empty() {
        not_recorded.to_string()
    } else {
        decision.policy_refs.join(", ")
    };
    let resume_condition = decision.resume_condition.as_deref().unwrap_or(not_recorded);
    let unknown_assurance = match language {
        "zh" => "未知",
        "ja" => "不明",
        _ => "Unknown",
    };
    let assurance = assurance.unwrap_or(unknown_assurance);
    format!(
        "{decision_label}: {}\n  {actor_label}: {}\n  {authority_label}: {}\n  {assurance_label}: {assurance}\n  {reason_label}: {}\n  {evidence_label}: {evidence_refs}\n  {policy_label}: {policy_refs}\n  {decided_label}: {}\n  {resume_label}: {resume_condition}",
        decision.decision,
        decision.actor,
        decision.authority_source,
        decision.reason,
        decision.decided_at,
    )
}
