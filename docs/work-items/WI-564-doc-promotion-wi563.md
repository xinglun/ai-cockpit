---
author: AI Cockpit maintainers
title: "WI-564 — terminal documentation promotion for WI-563"
description: "Promote WI-563 and register this documentation promotion in the three-language governance projections."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-564-doc-promotion-wi563
lastVerifiedBy: WI-564-doc-promotion-wi563
---

[简体中文](WI-564-doc-promotion-wi563.zh-CN.md) · [日本語](WI-564-doc-promotion-wi563.ja.md)

# WI-564 — terminal documentation promotion for WI-563

## Objective

Promote the verified-close documentation projections for WI-563 and register
this promotion Work Item itself so the documentation governance gate can audit
every current-cycle Work Item.

## Scope and boundary

The scope is limited to the three WI-563 language pages, the three WI-564
language pages, and the three matching reference-parity pages. The Runtime
promotion helper supplies terminal links for WI-563; the WI-564 page records
the bounded self-projection and remains in progress until this Work Item is
verified and closed.

Runtime behavior, object repositories, the local reference checkout, release
artifacts, global Agent/MCP configuration, and immutable Contract/evidence/
decision/archive bytes are out of scope.

## Acceptance

- WI-563 pages are promoted to Implemented with archive, verification,
  finalization, and close references.
- WI-564 pages explain the scope and are linked from all three parity pages.
- The three parity pages use consistent status and evidence paths for both
  Work Items, and documentation/inventory/governance gates pass.
- No predecessor Contract, evidence, decision, or archive bytes are rewritten.

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `git diff --check`
