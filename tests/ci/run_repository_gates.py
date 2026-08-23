#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

from quality_route import (
    PROFILE_ORDER,
    file_digest,
    load_manifest,
    profile_includes,
    validate_route_receipt,
)


def load_receipt(path: Path) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise ValueError("route receipt must be a regular file")
    try:
        receipt = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"unable to load route receipt: {error}") from error
    if not isinstance(receipt, dict):
        raise ValueError("route receipt must be an object")
    return receipt


def write_report(path: Path, report: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run only canonical repository gates selected by a typed route receipt"
    )
    parser.add_argument("--repo", required=True)
    parser.add_argument("--manifest", required=True)
    parser.add_argument("--report", required=True)
    parser.add_argument("--route-receipt")
    parser.add_argument("--profile", choices=PROFILE_ORDER)
    parser.add_argument("--list-only", action="store_true")
    args = parser.parse_args()

    repository = Path(args.repo).resolve()
    manifest_path = Path(args.manifest).resolve()
    report_path = Path(args.report)
    if not report_path.is_absolute():
        report_path = repository / report_path

    try:
        manifest = load_manifest(manifest_path)
        if args.list_only:
            if args.route_receipt:
                raise ValueError("--route-receipt cannot be combined with --list-only")
            selected_profile = args.profile or "strict"
            required_gate_ids = [
                gate["id"]
                for gate in manifest["gates"]
                if profile_includes(manifest, selected_profile, gate["minimumProfile"])
            ]
            route_binding: dict[str, Any] = {
                "manifestDigest": file_digest(manifest_path),
                "requiredGateIds": required_gate_ids,
                "selectedProfile": selected_profile,
            }
        else:
            if args.profile:
                raise ValueError("--profile is diagnostic-only and requires --list-only")
            if not args.route_receipt:
                raise ValueError("execution requires --route-receipt")
            receipt = load_receipt(Path(args.route_receipt))
            validate_route_receipt(
                receipt,
                repository=repository,
                manifest_path=manifest_path,
            )
            selected_profile = receipt["selectedProfile"]
            required_gate_ids = receipt["requiredGateIds"]
            route_binding = {
                "manifestDigest": receipt["manifestDigest"],
                "receiptDigest": receipt["receiptDigest"],
                "requiredGateIds": required_gate_ids,
                "selectedProfile": selected_profile,
            }
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.error(str(error))

    selected_ids = set(required_gate_ids)
    gates_by_id = {gate["id"]: gate for gate in manifest["gates"]}
    if len(selected_ids) != len(required_gate_ids) or selected_ids - gates_by_id.keys():
        parser.error("route receipt contains duplicate or unknown required gate IDs")
    gates = [gate for gate in manifest["gates"] if gate["id"] in selected_ids]
    if [gate["id"] for gate in gates] != required_gate_ids:
        parser.error("route receipt gate order does not match the canonical manifest")

    results: list[dict[str, Any]] = []
    failed = False
    for gate in gates:
        result: dict[str, Any] = {
            "category": gate["category"],
            "command": gate["command"],
            "id": gate["id"],
        }
        if gate.get("covers"):
            result["covers"] = gate["covers"]
        if args.list_only:
            result["state"] = "listed"
        else:
            command = list(gate["command"])
            if command[0].endswith(".sh"):
                command.insert(0, "bash")
            try:
                completed = subprocess.run(command, cwd=repository, check=False)
            except OSError as error:
                result["launchError"] = str(error)
                result["state"] = "failed"
                failed = True
            else:
                result["exitCode"] = completed.returncode
                result["state"] = "passed" if completed.returncode == 0 else "failed"
                failed = failed or completed.returncode != 0
            print(f"repository gate {result['id']}: {result['state']}", flush=True)
        results.append(result)

    report = {
        "gates": results,
        "route": route_binding,
        "schemaVersion": 2,
        "state": "listed" if args.list_only else ("failed" if failed else "passed"),
    }
    write_report(report_path, report)
    return 1 if failed else 0


if __name__ == "__main__":
    getattr(sys, "exit")(main())
