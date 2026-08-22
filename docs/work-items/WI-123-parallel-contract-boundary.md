---
author: AI Cockpit maintainers
title: "WI-123 — Parallel Contract Boundary and Slots"
description: "Contract-owned parallel path boundaries and repository-local slot leases."
audience:
  - adopter
  - maintainer
status: implementation
authority: canonical
lastVerifiedBy: WI-123
capabilityClaims:
  - parallel_contract_boundary
---

# WI-123 — Parallel Contract Boundary and Slots

## Objective

Make parallel Work Item authorization explicit, repository-local, and
fail-closed. A Contract may add a `concurrencyBoundary` with four path classes,
schema, reason, and `maxWorkers`; legacy intelligence remains the compatible
sidecar for dependencies, conflicts, and the `parallelizable` declaration.

## Scope

- additive `ConcurrencyBoundary` and strict `ParallelSlotLease` protocol types;
- conservative boundary overlap with exact, prefix, nested glob, and Windows
  separator handling;
- repository-local exclusive slot reservation and lease acquire/release/list;
- CLI `work-item boundary` and `work-item slot` commands and MCP
  `work_item_parallel` actions;
- English, Chinese, and Japanese capability/command documentation and race
  regression tests.

## Safety boundary

Unknown or malformed boundary/path/lease state serializes and fails closed. A
lease is bound to repository identity and Work Item identity, has no implicit
expiry, and cannot be released by another Work Item. `maxWorkers` controls
parallel slots and is distinct from `verify --workers`. No global Agent/MCP
configuration or global current repository is created.

## Compatibility

`concurrencyBoundary` is optional so existing Contract JSON and intelligence
sidecars remain readable. Without a boundary, the existing scope comparison is
used. When either side declares a boundary, missing or incompatible boundary
information is unknown and cannot authorize parallel execution.

## Verification

Protocol round trips, strict unknown-field rejection, boundary overlap, Windows
separators, missing boundary fail-closed behavior, slot capacity, duplicate-ID
races, and same-ID isolation across repositories are covered by the targeted
Rust tests. Full workspace checks and documentation acceptance remain required
before merge.
