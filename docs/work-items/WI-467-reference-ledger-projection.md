---
author: AI Cockpit maintainers
title: "WI-467 — reference ledger projection consistency"
workItemId: WI-467-reference-ledger-projection
description: "Keep the current reference-ledger snapshot and its tri-language documentation derived from one checked source."
audience: [maintainer, reviewer]
status: recovered
authority: authorized
lastVerifiedBy: WI-467-reference-ledger-projection
---

# WI-467 — reference ledger projection consistency

## Intent

Repair the current reference-file comparison snapshot after its prose counts
diverged from the machine ledger. Preserve historical snapshots and retired
paths, while adding a regression gate that rejects a marker-only update.

## Scope

- Derive the current tri-language snapshot table from
  `tests/conformance/reference_file_inventory.json`, excluding retired paths
  from current counts and retaining the append-only total separately.
- Make the English, Simplified Chinese, and Japanese current snapshot sections
  show the same canonical counts.
- Extend `reference_inventory_docs_test.py` and its shell wrapper so the gate
  validates the human-readable table as well as the existing marker.

## Out of scope

Reference inventory bytes, source lock, historical narrative sections, Runtime
or object repositories, workflow architecture, release scripts, and global
Agent/MCP configuration.

## Acceptance

1. The current table matches the machine-derived counts: 4,450 current paths,
   3,681 generated-history, 252 implemented-different-by-design, one
   implemented-equivalent, four not-applicable, 62 reference-only, 450
   deferred-next-batch, zero migrate-gap, 669 retired paths, and 5,119
   append-only records.
2. A deliberately changed table fails even when the machine marker is intact.
3. Historical sections and retired-path records remain byte-for-byte outside
   the declared current snapshot edits.
4. The three language pages retain their reader routes and semantic/non-wire
   boundary.

## Verification

- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/conformance/reference_file_inventory_test.sh`
- the Contract-declared repository quality and documentation gates

## Boundary

The ledger remains the authority for current counts. Historical narrative is
an immutable audit record and is not silently rewritten to match a later
snapshot.
