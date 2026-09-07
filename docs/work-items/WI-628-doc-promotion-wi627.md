---
author: AI Cockpit maintainers
title: "WI-628 — WI-627 documentation promotion"
description: "Promote the verified WI-627 terminal status into the tri-language reader documentation."
audience: [maintainer, reviewer, adopter]
workItemId: WI-628-doc-promotion-wi627
status: in_progress
authority: canonical
lastVerifiedBy: WI-628-doc-promotion-wi627
---

# WI-628 — WI-627 documentation promotion

## Intent

Promote the closed WI-627 reference rebaseline into the three-language reader
documentation without changing Contract, evidence, archive, finalization, or
decision bytes.

## Boundary

This Work Item is documentation-only. It covers the WI-627 English, Chinese,
and Japanese pages, this Work Item's three pages, and the three reference parity
ledgers. Runtime code, tests, CI, the reference ledger, generated governance
records, and global Agent/MCP configuration are out of scope.

## Acceptance

- WI-627 reader-facing status and terminal evidence links are current in all
  three languages.
- This Work Item has matching English, Chinese, and Japanese pages and is
  registered in all three parity ledgers before verification.
- The documentation checks pass after promotion.

## Verification

```text
python3 tests/docs/promote_closed_work_item.py --check-all
python3 tests/docs/reference_comparison_metadata_test.py
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
```

See also: [中文](WI-628-doc-promotion-wi627.zh-CN.md) ·
[日本語](WI-628-doc-promotion-wi627.ja.md).
