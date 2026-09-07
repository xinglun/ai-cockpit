---
author: AI Cockpit maintainers
title: "WI-627 — Rebaseline the reference file inventory"
description: "Bind the file-by-file comparison ledger to the latest local reference checkout without losing audit history."
audience: [maintainer, reviewer, adopter]
workItemId: WI-627-reference-rebaseline
status: implemented
authority: canonical
lastVerifiedBy: WI-627-reference-rebaseline
terminalArchive: .ai/work-items/archive/WI-627-reference-rebaseline.contract.json
terminalVerification: .ai/evidence/WI-627-reference-rebaseline.verification.json
terminalFinalization: .ai/decisions/WI-627-reference-rebaseline.finalize.json
terminalDecision: .ai/decisions/WI-627-reference-rebaseline.close.json
---

# WI-627 — Rebaseline the reference file inventory

## Intent

Rebind the comparison ledger to the latest local reference source commit
`a9224aed77b5c317b53c4551a9eec306d91ee330` and make the validator preserve
historical decisions while assigning every changed or newly visible path to a
future semantic batch. This is a ledger and documentation correction, not a
source implementation port.

## Result

The current inventory contains 5,175 tracked paths: 4,262 generated-history,
426 implemented-different-by-design, 1 implemented-equivalent, 8
not-applicable, 124 reference-only, 354 deferred-next-batch, and 0 migrate-gap.
No paths were retired in this rebaseline. The Rust baseline is
`98f12b18b978db509fc884a8a6225afeb7f10df5`, reviewed with Runtime v0.2.85
binary digest `sha256:ece00d0b596c4674eaf37225e95a83aacec66a4c33f0bb89857f5dcaecde3a50`.

The validator checks the complete current path set first, then excludes source-
changed paths only from historical batch ownership assertions. This prevents a
legitimate rebaseline from being reported as a missing old-batch record while
still requiring a current classification for every path.

## Boundaries and adopter inheritance

The reference checkout is local and pinned; no source Python, shell, Make,
provider configuration, or source JSON wire format is copied. Attached object
repositories continue to inherit one shared Runtime with explicit `--repo`,
isolated Contract/evidence/knowledge, and visible human Outcome handoff. Their
repository state remains independent from this comparison ledger.

## Verification

```text
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/docs/reference_comparison_metadata_test.py
```

See also: [中文](WI-627-reference-rebaseline.zh-CN.md) ·
[日本語](WI-627-reference-rebaseline.ja.md).
