#!/usr/bin/env python3
from __future__ import annotations

import argparse
import fnmatch
import hashlib
import json
import subprocess
import sys
from pathlib import Path, PurePosixPath
from typing import Any


PROFILE_ORDER = ("light", "standard", "strict")
STAGES = ("task", "pre_ci", "pull_request", "merge", "release")
HIGH_RISKS = {"high", "critical", "destructive"}


class RouteValidationError(ValueError):
    """A stable, human-actionable route failure without raw command output."""

    def __init__(self, code: str, message: str, remediation: str) -> None:
        self.code = code
        self.remediation = remediation
        super().__init__(f"{code}: {message}; remediation: {remediation}")


def failure_metadata(detail: str) -> tuple[str, str]:
    """Map a bounded diagnostic to one stable root and remediation."""
    normalized = detail.lower()
    if "lifecycle_transition_stale" in normalized:
        return (
            "lifecycle_transition_stale",
            "repair the active Work Item lifecycle, rerun preflight, and retry the transition",
        )
    if "lifecycle_transition_invalid" in normalized:
        return (
            "lifecycle_transition_invalid",
            "restore the declared lifecycle order and checkpoint/preflight bindings before pushing",
        )
    if "required_evidence_missing" in normalized:
        return (
            "required_evidence_missing",
            "collect the Contract-required evidence and rerun the declared verification",
        )
    if "reference" in normalized and "inventory" in normalized:
        return (
            "reference_inventory_mismatch",
            "refresh the pinned reference inventory and rerun the conformance check",
        )
    return (
        "quality_route_failed",
        "inspect the bound route receipt and rerun the declared repository gate locally",
    )


def canonical_digest(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def file_digest(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def _string_list(value: Any, field: str, *, allow_empty: bool = False) -> list[str]:
    if not isinstance(value, list) or (not value and not allow_empty):
        raise ValueError(f"{field} must be a {'list' if allow_empty else 'non-empty list'}")
    if any(not isinstance(item, str) or not item.strip() for item in value):
        raise ValueError(f"{field} must contain non-empty strings")
    return list(value)


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        manifest = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"unable to load gate manifest: {error}") from error
    if not isinstance(manifest, dict):
        raise ValueError("gate manifest must be an object")
    expected = {"gates", "pathProfiles", "profileOrder", "releaseOwnedPatterns", "schemaVersion", "stageFloors", "unknownProfile"}
    if set(manifest) != expected or manifest.get("schemaVersion") != 2:
        raise ValueError("gate manifest fields do not match schemaVersion 2")
    if tuple(_string_list(manifest.get("profileOrder"), "profileOrder")) != PROFILE_ORDER:
        raise ValueError(f"profileOrder must be {list(PROFILE_ORDER)}")
    if manifest.get("unknownProfile") not in PROFILE_ORDER:
        raise ValueError("unknownProfile is invalid")
    profiles = manifest.get("pathProfiles")
    if not isinstance(profiles, dict) or set(profiles) != set(PROFILE_ORDER):
        raise ValueError("pathProfiles must define light, standard, and strict")
    for profile in PROFILE_ORDER:
        _string_list(profiles[profile], f"pathProfiles.{profile}")
    _string_list(manifest.get("releaseOwnedPatterns"), "releaseOwnedPatterns")
    floors = manifest.get("stageFloors")
    if not isinstance(floors, dict) or set(floors) != set(STAGES):
        raise ValueError("stageFloors must define every verification stage")
    if any(value not in PROFILE_ORDER for value in floors.values()):
        raise ValueError("stageFloors contains an invalid profile")
    gates = manifest.get("gates")
    if not isinstance(gates, list) or not gates:
        raise ValueError("gates must be a non-empty list")
    ids: list[str] = []
    commands: list[tuple[str, ...]] = []
    for index, gate in enumerate(gates):
        if not isinstance(gate, dict):
            raise ValueError(f"gates[{index}] must be an object")
        if not {"category", "command", "id", "minimumProfile"}.issubset(gate):
            raise ValueError(f"gates[{index}] is missing required fields")
        if set(gate) - {"category", "command", "covers", "id", "minimumProfile"}:
            raise ValueError(f"gates[{index}] contains unknown fields")
        gate_id = gate["id"]
        if not isinstance(gate_id, str) or not gate_id:
            raise ValueError(f"gates[{index}].id is invalid")
        command = tuple(_string_list(gate["command"], f"gates[{index}].command"))
        if gate["minimumProfile"] not in PROFILE_ORDER:
            raise ValueError(f"gates[{index}].minimumProfile is invalid")
        if "covers" in gate:
            _string_list(gate["covers"], f"gates[{index}].covers")
        ids.append(gate_id)
        commands.append(command)
    if ids != sorted(ids) or len(ids) != len(set(ids)):
        raise ValueError("gate IDs must be sorted and unique")
    if len(commands) != len(set(commands)):
        raise ValueError("gate commands must be unique")
    return manifest


def _rank(profile: str) -> int:
    try:
        return PROFILE_ORDER.index(profile)
    except ValueError as error:
        raise ValueError(f"unsupported profile: {profile}") from error


def profile_includes(manifest: dict[str, Any], selected: str, minimum: str) -> bool:
    del manifest
    return _rank(selected) >= _rank(minimum)


def normalize_paths(paths: list[str], repository: Path | None = None) -> list[str]:
    root = repository.resolve() if repository is not None else None
    normalized: set[str] = set()
    for raw in paths:
        value = raw.replace("\\", "/")
        pure = PurePosixPath(value)
        if not value or pure.is_absolute() or ".." in pure.parts or value.startswith("./"):
            raise ValueError(f"unsafe changed path: {raw}")
        value = pure.as_posix()
        if root is not None and (root / value).exists():
            resolved = (root / value).resolve()
            try:
                resolved.relative_to(root)
            except ValueError as error:
                raise ValueError(f"changed path escapes repository: {raw}") from error
        normalized.add(value)
    return sorted(normalized)


def _git(repository: Path, arguments: list[str]) -> str:
    result = subprocess.run(["git", "-C", str(repository), *arguments], check=False, capture_output=True, text=True)
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "git inspection failed"
        raise ValueError(detail)
    return result.stdout.strip()


def resolve_commit(repository: Path, revision: str) -> str:
    commit = _git(repository, ["rev-parse", "--verify", f"{revision}^{{commit}}"])
    if len(commit) != 40 or any(character not in "0123456789abcdef" for character in commit):
        raise ValueError(f"invalid Git commit: {revision}")
    return commit


def changed_paths(repository: Path, base: str, head: str) -> list[str]:
    commands = (["diff", "--name-only", f"{base}...{head}", "--"], ["diff", "--name-only", head, "--"], ["ls-files", "--others", "--exclude-standard"])
    paths: list[str] = []
    for command in commands:
        output = _git(repository, command)
        paths.extend(output.splitlines())
    return normalize_paths(paths, repository)


def select_route(manifest: dict[str, Any], *, paths: list[str], risk: str, stage: str, requested_profile: str | None) -> dict[str, Any]:
    if stage not in STAGES:
        raise ValueError(f"unsupported stage: {stage}")
    normalized = normalize_paths(paths)
    decisions: list[dict[str, Any]] = []
    for path in normalized:
        if any(fnmatch.fnmatchcase(path, pattern) for pattern in manifest["releaseOwnedPatterns"]):
            profile, reason = "strict", f"release-owned path requires strict: {path}"
        else:
            matches = [profile for profile in PROFILE_ORDER if any(fnmatch.fnmatchcase(path, pattern) for pattern in manifest["pathProfiles"][profile])]
            if matches:
                profile = max(matches, key=_rank)
                reason = f"{profile} path policy: {path}"
            else:
                profile = manifest["unknownProfile"]
                reason = f"unknown path defaults to {profile}: {path}"
        decisions.append({"path": path, "profile": profile, "reason": reason})
    automatic = max((item["profile"] for item in decisions), key=_rank) if decisions else manifest["unknownProfile"]
    reasons = [item["reason"] for item in decisions if item["profile"] == automatic]
    stage_floor = manifest["stageFloors"][stage]
    if _rank(stage_floor) > _rank(automatic):
        automatic = stage_floor
        reasons.append(f"stage {stage} requires at least {stage_floor}")
    if risk.strip().lower() in HIGH_RISKS and automatic != "strict":
        automatic = "strict"
        reasons.append(f"risk {risk} requires strict")
    selected = automatic
    if requested_profile is not None:
        if requested_profile not in PROFILE_ORDER:
            raise ValueError(f"unsupported requested profile: {requested_profile}")
        if _rank(requested_profile) < _rank(automatic):
            raise ValueError(f"explicit profile {requested_profile} cannot lower automatic profile {automatic}")
        if _rank(requested_profile) > _rank(automatic):
            selected = requested_profile
            reasons.append(f"explicit escalation to {requested_profile}")
    required_gate_ids = [gate["id"] for gate in manifest["gates"] if profile_includes(manifest, selected, gate["minimumProfile"])]
    return {"automaticProfile": automatic, "pathDecisions": decisions, "reasons": sorted(set(reasons)) or [f"empty diff defaults to {automatic}"], "requiredGateIds": required_gate_ids, "selectedProfile": selected}


def _contract_binding(repository: Path, contract_path: Path | None) -> tuple[str | None, str | None, str | None]:
    if contract_path is None:
        return None, None, None
    root = repository.resolve()
    candidate = contract_path if contract_path.is_absolute() else root / contract_path
    if candidate.is_symlink() or not candidate.is_file():
        raise ValueError("Contract path must be a regular file")
    candidate = candidate.resolve()
    try:
        relative = candidate.relative_to(root).as_posix()
    except ValueError as error:
        raise ValueError("Contract path escapes repository") from error
    try:
        contract = json.loads(candidate.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"unable to read Contract: {error}") from error
    contract_risk = contract.get("risk")
    if not isinstance(contract_risk, str) or not contract_risk.strip():
        raise ValueError("Contract risk is missing")
    return relative, file_digest(candidate), contract_risk


def validate_lifecycle_boundary(repository: Path, contract_relative: str | None) -> None:
    """Reject known stale/unnormalized active lifecycle states at CI entry.

    This is deliberately a narrow boundary check. The Rust Runtime remains the
    lifecycle authority; CI only refuses to spend a hosted run on an active
    projection that already records an impossible transition. A missing
    Summary is left to the Runtime Contract gate for compatibility with
    standalone route fixtures.
    """
    if not contract_relative:
        return
    contract_name = Path(contract_relative).name
    if not contract_name.endswith(".contract.json"):
        return
    work_item_id = contract_name.removesuffix(".contract.json")
    summary_path = repository / ".ai/work-items/active" / f"{work_item_id}.summary.json"
    if not summary_path.exists():
        return
    if summary_path.is_symlink() or not summary_path.is_file():
        raise RouteValidationError(
            "lifecycle_transition_invalid",
            "active lifecycle Summary is not a regular file",
            "restore the repository-local Summary and rerun preflight before pushing",
        )
    try:
        summary = json.loads(summary_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise RouteValidationError(
            "lifecycle_transition_invalid",
            "active lifecycle Summary is malformed",
            "restore a Runtime-generated Summary and rerun preflight before pushing",
        ) from error
    if not isinstance(summary, dict):
        raise RouteValidationError(
            "lifecycle_transition_invalid",
            "active lifecycle Summary must be an object",
            "restore a Runtime-generated Summary and rerun preflight before pushing",
        )

    state = summary.get("state")
    if state not in {"implementation_active", "checkpointed", "finish_ready"}:
        raise RouteValidationError(
            "lifecycle_transition_invalid",
            f"active lifecycle state {state!r} cannot enter the CI route",
            "restore the declared lifecycle order and retry from the current Work Item",
        )
    if state in {"checkpointed", "finish_ready"} and summary.get("checkpointCount") != 1:
        raise RouteValidationError(
            "lifecycle_transition_invalid",
            "checkpointed lifecycle does not contain exactly one checkpoint",
            "record one valid checkpoint after fresh preflight before pushing",
        )
    if state == "finish_ready" and summary.get("preflightState") != "green":
        raise RouteValidationError(
            "lifecycle_transition_invalid",
            "finish_ready lifecycle is not backed by a green preflight",
            "rerun the required checks and preflight before retrying finish",
        )
    failed_gate = summary.get("failedGate")
    finalization_state = summary.get("finalizationState")
    if failed_gate or finalization_state in {"stale", "invalid", "blocked"}:
        raise RouteValidationError(
            "lifecycle_transition_stale",
            "active lifecycle contains a failed or stale transition marker",
            "use the Runtime recovery path, refresh evidence, and push only the repaired state",
        )


def plan_repository_route(*, repository: Path, manifest_path: Path, base: str, head: str, stage: str, risk: str, contract_path: Path | None, requested_profile: str | None) -> dict[str, Any]:
    repository = repository.resolve()
    manifest_path = manifest_path.resolve()
    manifest = load_manifest(manifest_path)
    base_commit = resolve_commit(repository, base)
    head_commit = resolve_commit(repository, head)
    paths = changed_paths(repository, base_commit, head_commit)
    contract_relative, contract_digest, contract_risk = _contract_binding(repository, contract_path)
    validate_lifecycle_boundary(repository, contract_relative)
    effective_risk = contract_risk or risk
    if risk.strip().lower() in HIGH_RISKS:
        effective_risk = risk
    selection = select_route(manifest, paths=paths, risk=effective_risk, stage=stage, requested_profile=requested_profile)
    receipt: dict[str, Any] = {"schemaVersion": 1, "kind": "repository_quality_route", "baseRevision": base_commit, "headRevision": head_commit, "stage": stage, "risk": effective_risk, "requestedRisk": risk, "requestedProfile": requested_profile, "manifestDigest": file_digest(manifest_path), "changedPaths": paths, "contractPath": contract_relative, "contractDigest": contract_digest, **selection}
    receipt["receiptDigest"] = canonical_digest(receipt)
    return receipt


def validate_route_receipt(receipt: dict[str, Any], *, repository: Path, manifest_path: Path) -> None:
    if not isinstance(receipt, dict) or receipt.get("schemaVersion") != 1 or receipt.get("kind") != "repository_quality_route":
        raise ValueError("route receipt schema or kind is invalid")
    if receipt.get("manifestDigest") != file_digest(manifest_path.resolve()):
        raise ValueError("route receipt manifest digest does not match")
    digest = receipt.get("receiptDigest")
    payload = dict(receipt)
    payload.pop("receiptDigest", None)
    if digest != canonical_digest(payload):
        raise ValueError("route receipt digest does not match")
    contract = receipt.get("contractPath")
    expected = plan_repository_route(repository=repository, manifest_path=manifest_path, base=str(receipt.get("baseRevision", "")), head=str(receipt.get("headRevision", "")), stage=str(receipt.get("stage", "")), risk=str(receipt.get("requestedRisk", "")), contract_path=Path(contract) if isinstance(contract, str) else None, requested_profile=str(receipt["requestedProfile"]) if receipt.get("requestedProfile") is not None else None)
    if receipt != expected:
        raise ValueError("route receipt does not match repository facts or required gates")


def main() -> int:
    parser = argparse.ArgumentParser(description="Plan a typed repository CI quality route")
    parser.add_argument("--repo", required=True)
    parser.add_argument("--manifest", required=True)
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", default="HEAD")
    parser.add_argument("--stage", choices=STAGES, required=True)
    parser.add_argument("--risk", default="normal")
    parser.add_argument("--contract")
    parser.add_argument("--profile", choices=PROFILE_ORDER)
    parser.add_argument("--receipt", required=True)
    args = parser.parse_args()
    repository = Path(args.repo).resolve()
    try:
        receipt = plan_repository_route(repository=repository, manifest_path=Path(args.manifest), base=args.base, head=args.head, stage=args.stage, risk=args.risk, contract_path=Path(args.contract) if args.contract else None, requested_profile=args.profile)
    except ValueError as error:
        code, remediation = failure_metadata(str(error))
        print(
            json.dumps(
                {"state": "failed", "failureCode": code, "remediation": remediation},
                sort_keys=True,
            ),
            file=sys.stderr,
        )
        parser.error(str(error))
    output = Path(args.receipt)
    if not output.is_absolute():
        output = repository / output
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(receipt, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
