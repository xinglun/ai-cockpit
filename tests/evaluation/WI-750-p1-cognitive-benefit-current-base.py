#!/usr/bin/env python3
"""Run the fixed, repository-local P1-A cognitive-benefit evaluation.

This is an evaluation harness, not a user study.  It compares the current
reader-first summary with the same Runtime's complete evidence projection for
real archived OutcomeV2 records, then records the human-measurement fields as
not measured because no participants are supplied by this repository.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any


WORK_ITEM = "WI-750-p1-cognitive-benefit-current-base"
OUTPUT_JSON = Path(".ai/evidence") / f"{WORK_ITEM}.json"
OUTPUT_MD = Path(".ai/evidence/external") / f"{WORK_ITEM}.md"


# This answer key is intentionally fixed in source before the comparison is
# run.  The source records are real OutcomeV2 archives; the conformance input
# references preserve the exact boundary that motivated each task.  They are
# never treated as human answers or authorization.
TASKS: tuple[dict[str, Any], ...] = (
    {
        "id": "normal-completion",
        "work_item": "WI-663-wi659-outcome-trust-replacement",
        "answer": {
            "verification": "stored Outcome is finish_ready/green",
            "human_decision": "not inferred from green; inspect the explicit decision field",
            "next_step": "review evidence before proceeding; green is not merge or release authorization",
            "risk_boundary": "empty risk record is not evidence of no risk",
        },
        "fixture": None,
    },
    {
        "id": "verified-pending-human-decision",
        "work_item": "WI-658-wi656-outcome-trust-repair",
        "answer": {
            "verification": "stored Outcome is finish_ready/green, subject to current-runtime revalidation",
            "human_decision": "not recorded",
            "next_step": "review evidence and record an explicit human close decision",
            "risk_boundary": "do not treat verification as approval",
        },
        "fixture": None,
    },
    {
        "id": "scope-exceeded",
        "work_item": "WI-714-wi713-current-base-revalidation",
        "answer": {
            "verification": "stored Outcome is blocked; completion is not claimed",
            "human_decision": "not recorded",
            "next_step": "stop and repair or create a separately authorized Contract",
            "risk_boundary": "scope evidence is a boundary, not permission to proceed",
        },
        "fixture": "tests/conformance/fixtures/scope-exceeded/input.json",
    },
    {
        "id": "evidence-expired-or-identity-mismatch",
        "work_item": "WI-423-ci-convergence",
        "answer": {
            "verification": "foreign, stale, or mismatched evidence must not be presented as current success",
            "human_decision": "unknown unless an identity-bound decision record exists",
            "next_step": "remain stopped and obtain fresh, identity-matched evidence",
            "risk_boundary": "an evidence mismatch is a stop condition, not a warning to ignore",
        },
        "fixture": "tests/conformance/fixtures/contradictory-evidence/input.json",
    },
    {
        "id": "test-weakening-signal",
        "work_item": "WI-662-p0-benchmark-evidence",
        "answer": {
            "verification": "verification state and test-weakening signal are separate facts",
            "human_decision": "not inferred from verification",
            "next_step": "inspect the bounded weakening check and its evidence scope",
            "risk_boundary": "no recorded risk is not the same as no weakening",
        },
        "fixture": "tests/conformance/fixtures/test-weakening/input.json",
    },
    {
        "id": "unverified-scope",
        "work_item": "WI-139A-preflight-review",
        "answer": {
            "verification": "unverified scenarios remain unverified",
            "human_decision": "unknown unless explicitly recorded",
            "next_step": "complete the declared verification plan before claiming completion",
            "risk_boundary": "unverified scope must remain visible in the uncertainty section",
        },
        "fixture": "tests/conformance/fixtures/repository-prompt-injection/input.json",
    },
    {
        "id": "historical-closed-task",
        "work_item": "WI-743-wi715-p1-current-base-redelivery",
        "answer": {
            "verification": "historical evidence is not a current verification result",
            "human_decision": "read the recorded decision and its authority source; do not upgrade assurance",
            "next_step": "preserve history and reverify only when a current result is needed",
            "risk_boundary": "historical or superseded is distinct from current failure",
        },
        "fixture": None,
    },
)


STATUS_LABELS = ("Verification", "Lifecycle", "Human decision", "Governance signal")
PROMPT_INJECTION_MARKERS = (
    "ignore " + "previous instructions",
    "ignore " + "all previous",
    "approve " + "the merge",
    "授权" + "合并",
    "承認して" + "マージ",
)


def digest_bytes(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def digest_file(path: Path) -> str:
    return digest_bytes(path.read_bytes())


def run(command: list[str], *, repo: Path, env: dict[str, str] | None = None) -> str:
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    result = subprocess.run(
        command,
        cwd=repo,
        env=merged_env,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"command failed ({result.returncode}): {' '.join(command)}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    return result.stdout


def resolve_binary(repo: Path, requested: str | None) -> Path:
    candidates: list[Path] = []
    if requested:
        candidates.append(Path(requested))
    candidates.extend((repo / "target/debug/ai-cockpit", Path(shutil.which("ai-cockpit") or "")))
    for candidate in candidates:
        if candidate and candidate.is_file() and os.access(candidate, os.X_OK):
            help_text = subprocess.run(
                [str(candidate), "work-item", "outcome", "--help"],
                cwd=repo,
                check=False,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
            ).stdout
            if "--view" in help_text:
                return candidate.resolve()

    run(["cargo", "build", "--locked", "-q", "-p", "cockpit-cli"], repo=repo)
    built = repo / "target/debug/ai-cockpit"
    if not built.is_file():
        raise RuntimeError("current source did not produce target/debug/ai-cockpit")
    return built.resolve()


def load_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise RuntimeError(f"expected JSON object: {path}")
    return value


def field(text: str, label: str) -> str | None:
    prefix = f"- {label}:"
    for line in text.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :].strip()
    return None


def evidence_refs(text: str) -> list[str]:
    refs: list[str] = []
    for line in text.splitlines():
        if line.startswith("- Evidence refs:"):
            refs.extend(item.strip() for item in line.split(":", 1)[1].split(","))
    return [ref for ref in refs if ref]


def source_record(repo: Path, work_item: str) -> tuple[dict[str, Any], Path, Path]:
    outcome_path = repo / ".ai/work-items/archive" / f"{work_item}.outcome.json"
    contract_path = repo / ".ai/work-items/archive" / f"{work_item}.contract.json"
    if not outcome_path.is_file() or not contract_path.is_file():
        raise RuntimeError(f"missing real archived Outcome structure for {work_item}")
    outcome = load_json(outcome_path)
    if outcome.get("schemaVersion") != 2 or not isinstance(
        outcome.get("taskOutcomeReport"), dict
    ):
        raise RuntimeError(f"{outcome_path} is not an OutcomeV2/task report pair")
    return outcome, outcome_path, contract_path


def compare_case(repo: Path, binary: Path, task: dict[str, Any]) -> dict[str, Any]:
    outcome, outcome_path, contract_path = source_record(repo, task["work_item"])
    fixture_path = repo / task["fixture"] if task["fixture"] else None
    fixture_validation: dict[str, Any] | None = None
    if fixture_path and not fixture_path.is_file():
        raise RuntimeError(f"missing answer-key fixture: {fixture_path}")
    if fixture_path:
        fixture = load_json(fixture_path)
        fixture_validation = {"present": True}
        if task["id"] == "scope-exceeded":
            fixture_validation["passed"] = (
                "tests/secret.rs" in fixture.get("changed_paths", [])
                and fixture.get("scope") == ["src/**"]
            )
        elif task["id"] == "evidence-expired-or-identity-mismatch":
            fixture_validation["passed"] = (
                "evidence_contradictory" in fixture.get("explicit_blockers", [])
            )
        elif task["id"] == "test-weakening-signal":
            fixture_validation["passed"] = fixture.get("test_weakening") is True
        else:
            fixture_validation["passed"] = True
    else:
        fixture_validation = {"present": False, "passed": True}

    summary = run(
        [
            str(binary),
            "work-item",
            "outcome",
            "--repo",
            str(repo),
            "--id",
            task["work_item"],
            "--view",
            "summary",
        ],
        repo=repo,
    )
    full = run(
        [
            str(binary),
            "work-item",
            "outcome",
            "--repo",
            str(repo),
            "--id",
            task["work_item"],
            "--view",
            "full",
        ],
        repo=repo,
    )

    violations: list[str] = []
    if not fixture_validation["passed"]:
        violations.append(f"answer-key fixture does not match {task['id']}")
    if task["id"] == "unverified-scope" and "unverified" not in contract_path.read_text(
        encoding="utf-8"
    ).casefold():
        violations.append("unverified-scope source Contract has no unverified marker")
    for label in STATUS_LABELS:
        summary_value = field(summary, label)
        full_value = field(full, label)
        if summary_value is None:
            violations.append(f"summary missing {label}")
        if full_value is None:
            violations.append(f"full missing {label}")
        if summary_value != full_value:
            violations.append(f"{label} differs between summary and full")

    for heading in ("Result", "Key changes", "Remaining uncertainty", "Human next step"):
        if heading not in summary:
            violations.append(f"summary missing four-part heading: {heading}")
    if "Full evidence report:" not in summary:
        violations.append("summary missing full-report entry point")
    if "Evidence" not in full:
        violations.append("full report missing evidence section")

    for ref in evidence_refs(summary):
        if ref.startswith(".ai/") and not (repo / ref).is_file():
            violations.append(f"summary references missing evidence: {ref}")
    for marker in PROMPT_INJECTION_MARKERS:
        if marker.casefold() in summary.casefold():
            violations.append(f"raw evidence marker reached summary: {marker}")

    return {
        "id": task["id"],
        "workItem": task["work_item"],
        "sourceOutcome": str(outcome_path.relative_to(repo)),
        "sourceContract": str(contract_path.relative_to(repo)),
        "sourceOutcomeDigest": digest_file(outcome_path),
        "sourceState": outcome.get("state"),
        "sourceDecisionState": outcome.get("decisionState"),
        "sourceUnknowns": outcome.get("unknowns", []),
        "fixture": str(fixture_path.relative_to(repo)) if fixture_path else None,
        "fixtureValidation": fixture_validation,
        "answerKey": task["answer"],
        "summary": summary,
        "full": full,
        "summaryDigest": digest_bytes(summary.encode()),
        "fullDigest": digest_bytes(full.encode()),
        "summaryFields": {label: field(summary, label) for label in STATUS_LABELS},
        "fullFields": {label: field(full, label) for label in STATUS_LABELS},
        "evidenceRefs": evidence_refs(summary),
        "consistencyViolations": violations,
        "fullViewReason": "automated summary/full consistency oracle; not a participant view",
    }


def language_check(repo: Path, binary: Path, work_item: str, language: str) -> dict[str, Any]:
    output = run(
        [
            str(binary),
            "work-item",
            "outcome",
            "--repo",
            str(repo),
            "--id",
            work_item,
            "--view",
            "summary",
        ],
        repo=repo,
        env={"AI_COCKPIT_LANGUAGE": language},
    )
    headings = {
        "zh": ("结果", "关键变化", "剩余不确定性", "人的下一步"),
        "ja": ("結果", "主な変更", "残る不確実性", "人間の次のアクション"),
    }[language]
    missing = [heading for heading in headings if heading not in output]
    return {
        "language": language,
        "workItem": work_item,
        "outputDigest": digest_bytes(output.encode()),
        "requiredHeadings": list(headings),
        "missingHeadings": missing,
        "passed": not missing,
    }


def write_markdown(repo: Path, report: dict[str, Any]) -> None:
    lines = [
        f"# {WORK_ITEM}",
        "",
        "This is a repository-local evaluation artifact, not a real user study.",
        "No participants or external adopter data were supplied; cognitive benefit remains unvalidated.",
        "",
        "## Fixed answer key and method",
        "",
        "The seven task categories and answer key were fixed in the evaluation script before rendering.",
        "For each real archived OutcomeV2 record, the current Runtime rendered both `summary` and `full`.",
        "The automatic oracle compares verification, lifecycle, human-decision, governance-signal, evidence-reference, and critical-uncertainty facts.",
        "It does not count its own full-view calls as user behavior.",
        "",
        "## Results",
        "",
        f"- Cases: {len(report['cases'])}",
        f"- Automatic consistency violations: {report['metrics']['summary_full_consistency_violations']}",
        f"- Critical visibility failures: {report['metrics']['critical_visibility_failures']}",
        "- Correct state/next-step time: not measured (no participants).",
        "- Erroneous release count: not measured (no participants).",
        "- Key-risk omission count: not measured as user behavior; automatic visibility checks are reported above.",
        "- Green-as-safe or green-as-authorized misunderstandings: not measured (no participants).",
        f"- Full evidence views: {report['metrics']['full_view_invocations']} automated calls, all for consistency checking; not user behavior.",
        "",
        "## Per-case evidence",
        "",
        "| Case | Source Outcome | Stored state | Current summary verification | Current summary next-step evidence |",
        "| --- | --- | --- | --- | --- |",
    ]
    for case in report["cases"]:
        verification = case["summaryFields"].get("Verification") or "not recorded"
        next_step = "yes" if "Human next step" in case["summary"] else "no"
        lines.append(
            f"| `{case['id']}` | `{case['workItem']}` | `{case['sourceState']}/{case['sourceDecisionState']}` | {verification} | {next_step} |"
        )
    lines.extend(
        [
            "",
            "## Limits",
            "",
            "The source records are repository evidence, not participant responses. Historical records are intentionally reported as historical or otherwise not current when the current Runtime cannot revalidate them. This artifact therefore demonstrates repeatability and cross-view consistency only; it does not claim a reduction in human reading time or an observed safety benefit.",
            "",
            "Reproduce with:",
            "",
            "```sh",
            "bash tests/evaluation/WI-750-p1-cognitive-benefit-current-base_test.sh",
            "```",
            "",
        ]
    )
    OUTPUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_MD.write_text("\n".join(lines), encoding="utf-8")


def evaluate(repo: Path, requested_binary: str | None) -> dict[str, Any]:
    binary = resolve_binary(repo, requested_binary)
    cases = [compare_case(repo, binary, task) for task in TASKS]
    languages = [
        language_check(repo, binary, TASKS[0]["work_item"], "zh"),
        language_check(repo, binary, TASKS[0]["work_item"], "ja"),
    ]
    consistency_violations = sum(len(case["consistencyViolations"]) for case in cases)
    critical_failures = sum(
        1
        for case in cases
        if "Human next step" not in case["summary"]
        or not case["evidenceRefs"]
        or case["summaryFields"] != case["fullFields"]
    )
    revision = run(["git", "rev-parse", "HEAD"], repo=repo).strip()
    report: dict[str, Any] = {
        "schemaVersion": 1,
        "workItemId": WORK_ITEM,
        "sourceRepositoryRevision": revision,
        "runtimeBinary": str(binary),
        "runtimeBinaryDigest": digest_file(binary),
        "taskSet": [task["id"] for task in TASKS],
        "participants": 0,
        "cognitiveBenefitValidated": False,
        "cases": cases,
        "languageChecks": languages,
        "metrics": {
            "correctStateAndNextStepTime": "not_measured",
            "erroneousReleaseCount": "not_measured",
            "keyRiskOmissionCount": "not_measured",
            "greenMisunderstandingCount": "not_measured",
            "fullEvidenceViewsByParticipants": "not_measured",
            "full_view_invocations": len(TASKS) * 2,
            "full_view_invocation_reason": "automated summary/full consistency oracle",
            "summary_full_consistency_violations": consistency_violations,
            "critical_visibility_failures": critical_failures,
        },
        "limits": [
            "No real participants were supplied.",
            "No external adopter or repository data was read or written.",
            "Automated full-view calls are not human reading observations.",
            "The result cannot establish a cognitive benefit, reading-time reduction, or risk-interception rate.",
        ],
    }
    return report


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--binary")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    repo = args.repo.resolve()
    try:
        report = evaluate(repo, args.binary)
    except (OSError, RuntimeError, ValueError, json.JSONDecodeError) as error:
        print(f"P1-A evaluation failed: {error}", file=sys.stderr)
        return 1

    OUTPUT_JSON.parent.mkdir(parents=True, exist_ok=True)
    output_path = repo / OUTPUT_JSON
    output_path.write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    write_markdown(repo, report)
    violations = [
        violation
        for case in report["cases"]
        for violation in case["consistencyViolations"]
    ]
    violations.extend(
        f"{item['language']}: {heading}"
        for item in report["languageChecks"]
        for heading in item["missingHeadings"]
    )
    if violations:
        print("P1-A automatic checks failed:", file=sys.stderr)
        for violation in violations:
            print(f"- {violation}", file=sys.stderr)
        return 1
    print(
        json.dumps(
            {
                "workItemId": WORK_ITEM,
                "cases": len(report["cases"]),
                "participants": report["participants"],
                "cognitiveBenefitValidated": report["cognitiveBenefitValidated"],
                "summaryFullConsistencyViolations": report["metrics"][
                    "summary_full_consistency_violations"
                ],
                "criticalVisibilityFailures": report["metrics"][
                    "critical_visibility_failures"
                ],
            },
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
