---
author: AI Cockpit maintainers
title: "WI-681 — P1 collaboration invariant coverage mapping"
description: "Maps the ten collaboration-language semantic invariants to existing automated test coverage, citing exact tests, and names remaining gaps as bounded follow-on Work Items."
audience: [maintainer, reviewer, adopter]
workItemId: WI-681-p1-invariant-coverage-mapping
status: in_progress
authority: authorized
lastVerifiedBy: WI-681-p1-invariant-coverage-mapping
---

[简体中文](WI-681-p1-invariant-coverage-mapping.zh-CN.md) · [日本語](WI-681-p1-invariant-coverage-mapping.ja.md)

# WI-681 — P1 collaboration invariant coverage mapping

## Intent

Continue the AI Cockpit collaboration-language initiative's P1 scope by
answering, for each of the ten semantic invariants stated in WI-679's
collaboration language contract (`docs/reference/collaboration-language-contract.md`,
not yet on the default branch as of this Work Item; see PR #675), whether
an automated test in this repository's own suite already enforces
it today, citing the exact test. This is a reuse-first survey: it exists to
avoid duplicating test infrastructure and to state honestly which
invariants have no automated cross-check yet, per explicit repository-owner
delegation to continue the initiative through the Work Item process.

## Boundary

This is a documentation-only Work Item. It adds
`docs/reference/collaboration-invariant-coverage.md` (+ zh-CN/ja), one index
entry in `docs/reference/README.md` (+ zh-CN/ja), this record page, and its
own reference-parity registration. It changes no test file, no CLI/MCP
behavior, and no Contract/Outcome schema. Writing the four identified
missing tests is explicitly out of scope and named as bounded follow-on
Work Items in the delivered document, each citing an existing test pattern
to extend rather than a new one to invent.

## Acceptance and lifecycle

- Every "Yes"/"Partial" row cites a test file and function verified by
  direct reading of the current test source at delivery time.
- Every gap ("No automated test found") is stated precisely, with a
  concrete, reuse-based follow-on approach.
- `start → preflight → checkpoint → verify → finish → archive → close` is the
  governed route; `user_visible_benefit_not_declared` remains explicit.
- `bash tests/docs/documentation_acceptance.sh` passes on the exact reviewed
  head.

## Evidence

- archive: `.ai/work-items/archive/WI-681-p1-invariant-coverage-mapping.contract.json`
- verification: `.ai/evidence/WI-681-p1-invariant-coverage-mapping.verification.json`
- finalization: `.ai/decisions/WI-681-p1-invariant-coverage-mapping.finalize.json` (pending merge)
- close: `.ai/decisions/WI-681-p1-invariant-coverage-mapping.close.json` (pending merge)
