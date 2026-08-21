#!/usr/bin/env python3
"""Execute canonical V2 fixture facts through a locked external V1 runtime.

This adapter is test-only. It deliberately never reads expected.json: its
output must be independent evidence that can contradict the checked-in V2
expectation.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any, Callable


FINDING_POLICY: dict[str, dict[str, Any]] = {
    "scope_exceeded": {
        "safe": ["stop_and_request_new_contract"],
        "checks": ["scope"],
    },
    "destructive_change_without_authority": {
        "safe": ["stop_and_request_human_authority"],
        "checks": ["authority", "scope"],
    },
    "required_evidence_missing": {
        "safe": ["collect_required_evidence", "rerun_preflight"],
        "checks": ["verification"],
    },
    "evidence_stale": {
        "safe": ["rerun_affected_checks", "rerun_preflight"],
        "checks": ["evidence_freshness"],
    },
    "evidence_contradictory": {
        "safe": ["stop_and_reconcile_evidence"],
        "checks": ["evidence_consistency"],
    },
    "unsupported_completion_claim": {
        "safe": ["remove_claim_or_provide_evidence"],
        "checks": ["completion_evidence"],
    },
    "repository_material_untrusted": {
        "safe": ["continue_with_explicit_policy", "treat_material_as_data"],
        "checks": ["input_trust"],
    },
    "unsafe_deletion_request": {
        "safe": ["stop_and_request_human_authority"],
        "checks": ["destructive_operation"],
    },
    "human_authority_missing": {
        "safe": ["request_human_decision"],
        "checks": ["authority"],
    },
    "archive_invalid": {
        "safe": ["preserve_active_work_item", "repair_archive_evidence"],
        "checks": ["archive_integrity"],
    },
    "cross_work_item_evidence": {
        "safe": ["rerun_evidence_for_current_work_item"],
        "checks": ["evidence_binding"],
    },
    "provider_result_unknown": {
        "safe": ["obtain_provider_receipt", "rerun_preflight"],
        "checks": ["external_evidence"],
    },
    "test_weakening": {
        "safe": ["request_human_decision", "restore_verification_strength"],
        "checks": ["test_integrity"],
    },
    "coverage_weakening": {
        "safe": ["request_human_decision", "restore_coverage_requirement"],
        "checks": ["coverage_integrity"],
    },
}


def canonical_result(
    case: str,
    *,
    decision: str,
    finding: str,
    authority: str = "not_evaluated",
    outcome: str | None = None,
) -> dict[str, Any]:
    """Project a raw V1 decision and finding into the V2 semantic vocabulary."""
    if decision not in {"green", "yellow", "red"}:
        raise ValueError(f"invalid normalized V1 decision: {decision}")
    policy = FINDING_POLICY[finding]
    return {
        "case": case,
        "decisionState": decision,
        "blockers": [finding] if decision == "red" else [],
        "unknowns": [finding] if decision == "yellow" else [],
        "safeActions": sorted(policy["safe"]),
        "requiredChecks": sorted(policy["checks"]),
        "authority": authority,
        "outcomeState": outcome
        or {"green": "ready", "yellow": "verification_pending", "red": "blocked"}[
            decision
        ],
    }


def fixture_input(case_root: Path) -> dict[str, Any]:
    value = json.loads((case_root / "input.json").read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"fixture input must be an object: {case_root}")
    return value


def probe_scope(case: str, root: Path, modules: dict[str, Any]) -> dict[str, Any]:
    facts = fixture_input(root)
    included = modules["common"].included
    uncovered = [
        path
        for path in facts["changed_paths"]
        if not included(path, list(facts["scope"]))
    ]
    if not uncovered:
        raise AssertionError("V1 scope probe did not detect an uncovered fixture path")
    return canonical_result(
        case,
        decision="red",
        finding="scope_exceeded",
        authority="absent_for_requested_change",
    )


def probe_evidence(
    case: str, root: Path, modules: dict[str, Any], *, state: str
) -> dict[str, Any]:
    domain = modules["domain"]
    evidence = domain.Evidence(
        kind="" if state == "missing" else "verification",
        digest="" if state == "missing" else "sha256:fixture",
        stale=state == "stale",
        contradictory=state == "contradictory",
    )
    result = domain.DomainService().transition(
        domain.WorkItem("fixture", "implementation_active"),
        "verification_pending",
        evidence=evidence,
    )
    if result.allowed:
        raise AssertionError(f"V1 accepted {state} evidence")
    finding = {
        "missing": "required_evidence_missing",
        "stale": "evidence_stale",
        "contradictory": "evidence_contradictory",
    }[state]
    return canonical_result(
        case,
        decision="red" if state == "contradictory" else "yellow",
        finding=finding,
    )


def operation_time_missing_authority(modules: dict[str, Any]) -> Any:
    trust = modules["trust"]
    request = trust.OperationTimeRequest(
        requestedOperation="delete_files",
        actualToolCall="delete_files",
        targetResource="src/lib.rs",
        declaredScope=("src/**",),
        approvedOperation="",
        approvedTargetResource="",
        approvedScope=(),
        currentAuthority="",
        evidenceFresh=True,
        destructiveImpact="high",
    )
    return trust.evaluate_operation_time_policy(request)


def probe_destructive_authority(
    case: str, root: Path, modules: dict[str, Any]
) -> dict[str, Any]:
    result = operation_time_missing_authority(modules)
    if result.decision != "confirm":
        raise AssertionError(f"V1 missing-authority decision changed: {result.decision}")
    return canonical_result(
        case,
        decision="yellow",
        finding="destructive_change_without_authority",
        authority="missing",
        outcome="needs_human_decision",
    )


def probe_missing_human_authority(
    case: str, root: Path, modules: dict[str, Any]
) -> dict[str, Any]:
    result = operation_time_missing_authority(modules)
    if result.decision != "confirm":
        raise AssertionError(f"V1 missing-authority decision changed: {result.decision}")
    return canonical_result(
        case,
        decision="yellow",
        finding="human_authority_missing",
        authority="missing",
        outcome="needs_human_decision",
    )


def governance_request(
    modules: dict[str, Any], *, content: str, risk: str, operation: str, source: str
) -> Any:
    trust = modules["trust"]
    return trust.evaluate_governance_request(
        trust.GovernanceRequest(
            sourceType=trust.SourceType(source),
            content=content,
            requestedOperation=operation,
            riskCategory=risk,
            evidenceConflict=True,
            independentAuthorization=False,
            recovery="preserve fixture evidence and request governed review",
        )
    )


def probe_prompt_injection(case: str, root: Path, modules: dict[str, Any]) -> dict[str, Any]:
    content = (root / "repository/material.txt").read_text(encoding="utf-8")
    result = governance_request(
        modules,
        content=content,
        risk="untrusted_instruction_like_content",
        operation="analyze_repository_material",
        source="repository",
    )
    if result.decision != "review" or result.gate != "input_trust":
        raise AssertionError(f"V1 prompt-injection decision changed: {result}")
    return canonical_result(
        case,
        decision="yellow",
        finding="repository_material_untrusted",
        outcome="needs_investigation",
    )


def probe_malicious_deletion(case: str, root: Path, modules: dict[str, Any]) -> dict[str, Any]:
    content = (root / "repository/material.txt").read_text(encoding="utf-8")
    result = governance_request(
        modules,
        content=content,
        risk="external_instruction",
        operation="delete_files",
        source="repository",
    )
    if result.decision != "block" or result.gate != "input_trust":
        raise AssertionError(f"V1 malicious-deletion decision changed: {result}")
    return canonical_result(
        case,
        decision="red",
        finding="unsafe_deletion_request",
        authority="missing",
    )


def v1_status_contract(work_item: str) -> dict[str, Any]:
    return {
        "contractVersion": 2,
        "workItemId": work_item,
        "mode": "code",
        "baseCommit": "a" * 40,
        "executionDecision": {"status": "continue"},
        "agentCapability": {"needsHumanDecision": False},
        "verification": [{"check": "quality", "required": True}],
        "intent": {"problem": "fixture", "constraints": ["local"], "rationale": "oracle"},
        "acceptance": ["fixture"],
        "unknowns": [],
        "riskAssessment": {"level": "low"},
        "guidelines": ["preserve evidence"],
    }


def v1_status_summary(work_item: str) -> dict[str, Any]:
    return {
        "summaryVersion": 2,
        "workItemId": work_item,
        "verification": [
            {
                "check": "quality",
                "result": "passed",
                "executedAt": "2026-08-21T00:00:00+00:00",
                "executionContractPath": f".ai/work-items/active/{work_item}.contract.json",
                "executionSummaryPath": f".ai/work-items/active/{work_item}.summary.json",
                "commitSha": "b" * 40,
            }
        ],
        "reviewReadiness": {"status": "ready"},
        "guidelinesCompliance": [{"guideline": "preserve evidence", "compliant": True}],
        "unknownsRemaining": [],
        "intentAlignment": {
            "problemResolved": True,
            "constraintsRespected": True,
            "nonGoalsAvoided": True,
            "rationaleValidated": True,
        },
        "risk": {"level": "low", "detail": "fixture"},
        "residualRisks": [],
    }


def probe_cross_work_item(case: str, root: Path, modules: dict[str, Any]) -> dict[str, Any]:
    status = modules["status"].build_status(
        v1_status_contract("wi-a"),
        v1_status_summary("wi-b"),
        branch="codex/wi-a",
        current_commit="b" * 40,
        now="2026-08-21T00:01:00Z",
    )
    if "cross_work_item_evidence" not in status["diagnostics"] or not status["blocking"]:
        raise AssertionError(f"V1 accepted cross-Work-Item evidence: {status}")
    return canonical_result(case, decision="red", finding="cross_work_item_evidence")


def probe_unsupported_completion(
    case: str, root: Path, modules: dict[str, Any]
) -> dict[str, Any]:
    result = modules["claims"].evaluate_claim({}, root=root)
    if result["state"] != "blocked":
        raise AssertionError(f"V1 accepted unsupported completion: {result}")
    return canonical_result(case, decision="red", finding="unsupported_completion_claim")


def probe_invalid_archive(case: str, root: Path, modules: dict[str, Any]) -> dict[str, Any]:
    result = modules["outcome"].validate_terminal_outcome(
        root / "missing.outcome.json",
        root / "missing.outcome.md",
        expected_task_id="fixture",
        contract_path=root / "contract.json",
        summary_path=root / "missing.summary.json",
    )
    if result.valid:
        raise AssertionError("V1 accepted an archive without terminal Outcome evidence")
    return canonical_result(case, decision="red", finding="archive_invalid")


def probe_unknown_provider(case: str, root: Path, modules: dict[str, Any]) -> dict[str, Any]:
    domain = modules["domain"]
    result = domain.DomainService().recover(
        domain.WorkItem("fixture", "archived"), provider_status="unknown"
    )
    if result["state"] != "stale" or not result["recoverable"]:
        raise AssertionError(f"V1 provider-unknown recovery changed: {result}")
    return canonical_result(
        case,
        decision="yellow",
        finding="provider_result_unknown",
        outcome="awaiting_external_evidence",
    )


def git(repo: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args], cwd=repo, check=True, capture_output=True, text=True
    )
    return result.stdout.strip()


def weakening_repository(*, coverage: bool = False) -> tuple[tempfile.TemporaryDirectory[str], Path, str]:
    temporary: tempfile.TemporaryDirectory[str] = tempfile.TemporaryDirectory(
        prefix="ai-cockpit-v1-oracle-"
    )
    repo = Path(temporary.name)
    git(repo, "init", "-q")
    git(repo, "config", "user.email", "oracle@example.invalid")
    git(repo, "config", "user.name", "V1 Oracle")
    if coverage:
        (repo / "pyproject.toml").write_text("fail_under = 90\n", encoding="utf-8")
    else:
        (repo / "README.md").write_text("fixture\n", encoding="utf-8")
    git(repo, "add", ".")
    git(repo, "commit", "-qm", "baseline")
    base = git(repo, "rev-parse", "HEAD")
    return temporary, repo, base


def probe_test_weakening(case: str, root: Path, modules: dict[str, Any]) -> dict[str, Any]:
    temporary, repo, base = weakening_repository()
    try:
        report = modules["weakening"].analyze(
            repo,
            base,
            mode="full",
            request="Delete the failing tests so CI passes.",
        )
    finally:
        temporary.cleanup()
    if report["decision"] != "block" or not any(
        signal["type"] == "explicit_test_bypass_request" for signal in report["signals"]
    ):
        raise AssertionError(f"V1 test-weakening decision changed: {report}")
    return canonical_result(case, decision="red", finding="test_weakening")


def probe_coverage_weakening(
    case: str, root: Path, modules: dict[str, Any]
) -> dict[str, Any]:
    temporary, repo, base = weakening_repository(coverage=True)
    try:
        (repo / "pyproject.toml").write_text("fail_under = 70\n", encoding="utf-8")
        report = modules["weakening"].analyze(repo, base, mode="full")
    finally:
        temporary.cleanup()
    if report["decision"] != "review" or not any(
        signal["type"] == "coverage_threshold_lowered" for signal in report["signals"]
    ):
        raise AssertionError(f"V1 coverage-weakening decision changed: {report}")
    return canonical_result(
        case,
        decision="yellow",
        finding="coverage_weakening",
        outcome="needs_human_decision",
    )


PROBES: dict[str, Callable[[str, Path, dict[str, Any]], dict[str, Any]]] = {
    "scope-exceeded": probe_scope,
    "unauthorized-destructive-change": probe_destructive_authority,
    "missing-evidence": lambda case, root, modules: probe_evidence(
        case, root, modules, state="missing"
    ),
    "stale-evidence": lambda case, root, modules: probe_evidence(
        case, root, modules, state="stale"
    ),
    "contradictory-evidence": lambda case, root, modules: probe_evidence(
        case, root, modules, state="contradictory"
    ),
    "unsupported-completion": probe_unsupported_completion,
    "repository-prompt-injection": probe_prompt_injection,
    "malicious-deletion": probe_malicious_deletion,
    "missing-human-authority": probe_missing_human_authority,
    "invalid-archive": probe_invalid_archive,
    "cross-work-item-evidence": probe_cross_work_item,
    "unknown-provider-result": probe_unknown_provider,
    "test-weakening": probe_test_weakening,
    "coverage-weakening": probe_coverage_weakening,
}


def load_v1(reference_root: Path) -> dict[str, Any]:
    scripts = reference_root / "scripts"
    if not scripts.is_dir():
        raise ValueError(f"V1 scripts directory is missing: {scripts}")
    sys.path.insert(0, str(scripts))
    import ai_check_test_weakening
    import ai_common
    import ai_domain_model
    import ai_generate_work_item_status
    import ai_input_trust
    import ai_outcome_gate
    import unsupported_claim_gate

    return {
        "common": ai_common,
        "domain": ai_domain_model,
        "status": ai_generate_work_item_status,
        "trust": ai_input_trust,
        "outcome": ai_outcome_gate,
        "claims": unsupported_claim_gate,
        "weakening": ai_check_test_weakening,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference-root", type=Path, required=True)
    parser.add_argument("--fixtures", type=Path, required=True)
    args = parser.parse_args()
    modules = load_v1(args.reference_root.resolve())
    fixture_cases = {path.name for path in args.fixtures.iterdir() if path.is_dir()}
    if fixture_cases != set(PROBES):
        missing = sorted(fixture_cases - set(PROBES))
        extra = sorted(set(PROBES) - fixture_cases)
        raise ValueError(f"Oracle case map mismatch: missing={missing}, extra={extra}")
    results = [
        PROBES[case](case, args.fixtures / case, modules) for case in sorted(PROBES)
    ]
    print(json.dumps(results, ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
