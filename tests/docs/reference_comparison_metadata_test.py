#!/usr/bin/env python3
"""Fail-closed guard for the live reference-comparison metadata projection."""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
METADATA_PATH = ROOT / "docs/reference/reference-comparison-metadata.json"
LOCK_PATH = ROOT / "tests/conformance/reference-source.lock"
INVENTORY_PATH = ROOT / "tests/conformance/reference_file_inventory.json"
COMPARISON_DOCS = (
    ROOT / "docs/reference/reference-file-comparison.md",
    ROOT / "docs/reference/reference-file-comparison.zh-CN.md",
    ROOT / "docs/reference/reference-file-comparison.ja.md",
)
PARITY_DOCS = (
    ROOT / "docs/reference/reference-parity.md",
    ROOT / "docs/reference/reference-parity.zh-CN.md",
    ROOT / "docs/reference/reference-parity.ja.md",
)
ALL_DOCS = COMPARISON_DOCS + PARITY_DOCS


def fail(message: str) -> "NoReturn":
    raise SystemExit(f"reference comparison metadata check failed: {message}")


def parse_lock_commit() -> str:
    for line in LOCK_PATH.read_text(encoding="utf-8").splitlines():
        key, separator, value = line.partition("=")
        if separator and key.strip() == "commit":
            return value.strip().strip('"')
    fail("reference-source.lock has no commit")


def git_head() -> str:
    result = subprocess.run(
        ["git", "rev-parse", "HEAD^{commit}"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def frontmatter(text: str) -> dict[str, str]:
    if not text.startswith("---\n"):
        fail("reference page has no frontmatter")
    _, body, _ = text.split("---\n", 2)
    fields: dict[str, str] = {}
    for line in body.splitlines():
        key, separator, value = line.partition(":")
        if separator:
            fields[key.strip()] = value.strip().strip('"')
    return fields


def main() -> None:
    metadata = json.loads(METADATA_PATH.read_text(encoding="utf-8"))
    required = {
        "schemaVersion",
        "lastVerifiedBy",
        "referenceCommit",
        "rustBaselineCommit",
        "runtimeVersion",
        "runtimeBinaryDigest",
        "currentPathCount",
        "classifiedPathCount",
        "semanticDecisionCount",
        "deferredPathCount",
        "retiredPathCount",
        "migrateGapCount",
    }
    if set(metadata) != required:
        fail(f"metadata keys differ: expected {sorted(required)}, got {sorted(metadata)}")
    if metadata["schemaVersion"] != 1:
        fail("unsupported metadata schema")
    reference_commit = parse_lock_commit()
    if metadata["referenceCommit"] != reference_commit:
        fail("metadata referenceCommit differs from reference-source.lock")
    inventory = json.loads(INVENTORY_PATH.read_text(encoding="utf-8"))
    if metadata["referenceCommit"] != inventory["referenceCommit"]:
        fail("metadata referenceCommit differs from inventory")
    records = inventory["records"]
    retired_paths = set(inventory.get("retiredReferencePaths", []))
    current_records = [record for record in records if record["referencePath"] not in retired_paths]
    if metadata["currentPathCount"] != len(current_records):
        fail("currentPathCount differs from non-retired inventory records")
    if metadata["retiredPathCount"] != len(records) - len(current_records):
        fail("retiredPathCount differs from inventory records")
    classifications = {}
    for record in current_records:
        classification = record["classification"]
        classifications[classification] = classifications.get(classification, 0) + 1
    classified = len(current_records) - classifications.get("deferred-next-batch", 0)
    semantic_decisions = sum(
        count
        for classification, count in classifications.items()
        if classification not in {"generated-history", "deferred-next-batch"}
    )
    if metadata["classifiedPathCount"] != classified:
        fail("classifiedPathCount differs from inventory")
    if metadata["semanticDecisionCount"] != semantic_decisions:
        fail("semanticDecisionCount differs from inventory")
    if metadata["deferredPathCount"] != classifications.get("deferred-next-batch", 0):
        fail("deferredPathCount differs from inventory")
    if metadata["migrateGapCount"] != classifications.get("migrate-gap", 0):
        fail("migrateGapCount differs from inventory")
    if not re.fullmatch(r"sha256:[0-9a-f]{64}", metadata["runtimeBinaryDigest"]):
        fail("runtimeBinaryDigest is not a SHA-256 identity")
    # The baseline is the reviewed default-branch checkout used for this
    # projection.  The documentation commit itself may be a descendant (and
    # a later release may be newer), so require ancestry rather than equality.
    baseline = metadata["rustBaselineCommit"]
    try:
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", baseline, git_head()],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        )
    except subprocess.CalledProcessError as error:
        fail(f"rustBaselineCommit is not an ancestor of the reviewed checkout: {error}")
    for path in ALL_DOCS:
        text = path.read_text(encoding="utf-8")
        fields = frontmatter(text)
        if fields.get("lastVerifiedBy") != metadata["lastVerifiedBy"]:
            fail(f"{path}: stale lastVerifiedBy")
        if "reference-comparison-metadata.json" not in text:
            fail(f"{path}: missing metadata sidecar link")
        if f"`{metadata['referenceCommit']}`" not in text:
            fail(f"{path}: current reference commit is missing")
    comparison_text = COMPARISON_DOCS[0].read_text(encoding="utf-8")
    inventory_marker = (
        "reference-inventory-counts: "
        f"total={metadata['currentPathCount']} "
        f"generated-history={classifications.get('generated-history', 0)} "
        "implemented-different-by-design="
        f"{classifications.get('implemented-different-by-design', 0)} "
        "implemented-equivalent="
        f"{classifications.get('implemented-equivalent', 0)} "
        "not-applicable="
        f"{classifications.get('not-applicable', 0)} "
        "reference-only="
        f"{classifications.get('reference-only', 0)} "
        "deferred-next-batch="
        f"{classifications.get('deferred-next-batch', 0)} "
        "migrate-gap="
        f"{classifications.get('migrate-gap', 0)}"
    )
    for path in COMPARISON_DOCS:
        text = path.read_text(encoding="utf-8")
        for marker in (
            metadata["rustBaselineCommit"],
            metadata["runtimeVersion"],
            metadata["runtimeBinaryDigest"],
            inventory_marker,
        ):
            if marker not in text:
                fail(f"{path}: current comparison marker missing: {marker}")
    if comparison_text.count(metadata["rustBaselineCommit"]) != 1:
        fail("comparison baseline commit must appear once in the live section")
    print("reference comparison metadata check passed")


if __name__ == "__main__":
    main()
