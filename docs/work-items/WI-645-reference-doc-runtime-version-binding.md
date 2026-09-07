---
author: AI Cockpit maintainers
title: WI-645 — reference comparison Runtime version binding
description: Keep the current reference-comparison Runtime projection synchronized with its machine-readable metadata.
audience: [maintainer, reviewer, adopter]
workItemId: WI-645-reference-doc-runtime-version-binding
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-645-reference-doc-runtime-version-binding
---

# WI-645 — reference comparison Runtime version binding

[简体中文](WI-645-reference-doc-runtime-version-binding.zh-CN.md) · [日本語](WI-645-reference-doc-runtime-version-binding.ja.md)

## Intent

Correct the current-snapshot Runtime version shown by the six tri-language
reference-comparison pages and add a regression assertion that binds the human
projection to `reference-comparison-metadata.json`.

## Boundary

This Work Item changes only current documentation projections and their static
metadata check. It does not change Runtime behavior, Protocol bytes, immutable
history, the pinned reference checkout, or any adopter repository.

## Acceptance and verification

- Every current-snapshot Runtime version and binary digest matches the metadata
  sidecar exactly once in all six pages.
- The metadata test fails for a stale current-snapshot version and passes for
  the current metadata.
- Tri-language documentation, inventory, governance integrity, and workspace
  checks pass without modifying historical records.

Terminal Contract, verification, finalization, and decision records are linked
from the archived Work Item after closure.
