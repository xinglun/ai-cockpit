---
author: AI Cockpit maintainers
title: "WI-484 — WI-483 terminal documentation promotion"
description: "Promote the WI-483 terminal documentation projections without rewriting immutable evidence."
audience: [maintainer, reviewer, adopter]
workItemId: WI-484-wi483-doc-promotion
status: current
authority: canonical
lastVerifiedBy: WI-484-wi483-doc-promotion
---

# WI-484 — WI-483 terminal documentation promotion

This bounded Work Item promotes the verified and closed WI-483 lifecycle into
the tri-language Work Item and reference-parity projections. It does not alter
immutable Runtime evidence, archive records, or reference-source semantics.

[简体中文](WI-484-wi483-doc-promotion.zh-CN.md) · [日本語](WI-484-wi483-doc-promotion.ja.md)

## Scope

- Promote the three WI-483 documentation projections using the repository helper.
- Keep the promotion deterministic and bound to the exact terminal records.
- Register this Work Item's own pages and parity row before archive.

## Out of scope

Runtime/Core implementation, release or adopter artifacts, new reference
comparison paths, and immutable governance bytes.

## Acceptance

1. The three WI-483 projections contain terminal evidence-backed metadata.
2. All three reference-parity rows identify WI-483 as implemented and link the same terminal evidence.
3. This Work Item has tri-language documentation and a pre-archive parity row.
4. `promote_closed_work_item.py --check-all` passes after close.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `git diff --check`
