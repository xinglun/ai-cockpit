#!/usr/bin/env python3
"""Check that the tri-language ledger snapshot matches the inventory."""

from __future__ import annotations

import json
import re
import sys
from collections import Counter
from pathlib import Path

MARKER = re.compile(r"<!--\s*reference-inventory-counts:\s*(.*?)\s*-->")


def expected_counts(manifest: Path) -> dict[str, int]:
    payload = json.loads(manifest.read_text(encoding="utf-8"))
    retired = {
        record if isinstance(record, str) else record["referencePath"]
        for record in payload.get("retiredReferencePaths", [])
        if isinstance(record, str) or (
            isinstance(record, dict) and isinstance(record.get("referencePath"), str)
        )
    }
    records = [
        record
        for record in payload["records"]
        if record.get("referencePath") not in retired
    ]
    counts = Counter(record["classification"] for record in records)
    return {
        "total": len(records),
        "generated-history": counts["generated-history"],
        "implemented-different-by-design": counts["implemented-different-by-design"],
        "implemented-equivalent": counts["implemented-equivalent"],
        "not-applicable": counts["not-applicable"],
        "reference-only": counts["reference-only"],
        "deferred-next-batch": counts["deferred-next-batch"],
        "migrate-gap": counts["migrate-gap"],
    }


def read_marker(document: Path) -> dict[str, int]:
    matches = MARKER.findall(document.read_text(encoding="utf-8"))
    if len(matches) != 1:
        raise ValueError(f"{document}: expected one reference-inventory-counts marker")
    values: dict[str, int] = {}
    for field in matches[0].split():
        key, separator, value = field.partition("=")
        if not separator or not value.isdigit():
            raise ValueError(f"{document}: malformed count field {field!r}")
        values[key] = int(value)
    return values


def main() -> int:
    root = Path(__file__).resolve().parents[2]
    expected = expected_counts(root / "tests/conformance/reference_file_inventory.json")
    documents = (
        root / "docs/reference/reference-file-comparison.md",
        root / "docs/reference/reference-file-comparison.zh-CN.md",
        root / "docs/reference/reference-file-comparison.ja.md",
    )
    for document in documents:
        actual = read_marker(document)
        if actual != expected:
            raise ValueError(
                f"{document}: marker {actual} does not match manifest {expected}"
            )
    print(f"reference inventory documentation counts match ({expected['total']} records)")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        raise SystemExit(1)
