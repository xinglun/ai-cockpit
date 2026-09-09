---
author: AI Cockpit maintainers
title: "WI-679 — P0 collaboration language contract"
description: "A docs-only cross-cutting map of the seven human-Agent communication moments to existing Runtime facts, plus ten checkable semantic invariants."
audience: [maintainer, reviewer, adopter]
workItemId: WI-679-p0-collaboration-language-contract
status: implemented
authority: authorized
lastVerifiedBy: WI-679-p0-collaboration-language-contract
terminalArchive: .ai/work-items/archive/WI-679-p0-collaboration-language-contract.contract.json
terminalVerification: .ai/evidence/WI-679-p0-collaboration-language-contract.verification.json
terminalFinalization: .ai/decisions/WI-679-p0-collaboration-language-contract.finalize.json
terminalDecision: .ai/decisions/WI-679-p0-collaboration-language-contract.close.json
---

[简体中文](WI-679-p0-collaboration-language-contract.zh-CN.md) · [日本語](WI-679-p0-collaboration-language-contract.ja.md)

# WI-679 — P0 collaboration language contract

## Intent

Give a new user or contributor who cannot ask a maintainer for a live
explanation a single cross-cutting reference mapping the repository's real
communication moments (task start/scope, authorization, verification,
block/recovery, merge confirmation, Outcome, Agent/session hand-off) to
existing Runtime facts, per explicit repository-owner delegation to complete
the AI Cockpit collaboration-language initiative through the Work Item
process.

## Boundary

This is a documentation-only Work Item. It adds
`docs/reference/collaboration-language-contract.md` (+ zh-CN/ja) and one
index entry in `docs/reference/README.md` (+ zh-CN/ja). It changes no CLI,
MCP, Contract schema, Outcome schema, or lifecycle behavior, and it does not
alter the meaning of any existing Outcome marker, decision-state color,
Receipt/authorization-reuse rule, or block/recovery vocabulary — it cites and
indexes the sources that already define them
(`.ai/glossary.md`, `docs/protocol/v1/specification.md`,
`docs/reference/outcome-report.md`,
`docs/reference/how-to-read-cockpit-status.md`,
`docs/reference/agent-workflow.md`, `docs/reference/troubleshooting.md`).
External participant recruitment, invitation, interview, and evaluation are
explicitly out of scope, per direction. The state/transition scenario matrix,
automated invariant checks, end-to-end consistency verification, and
handoff-completeness checks are named as follow-on Work Items in the
delivered document, not claimed as delivered here.

## Acceptance and lifecycle

- The delivered document covers the seven communication nodes and states ten
  semantic invariants, each citing where it is already true today and naming
  any automated-check gap explicitly.
- `bash tests/docs/documentation_acceptance.sh` passes for the new and
  modified files.
- `start → preflight → checkpoint → verify → finish → archive → close` is the
  governed route.

## Evidence

- archive: `.ai/work-items/archive/WI-679-p0-collaboration-language-contract.contract.json`
- verification: `.ai/evidence/WI-679-p0-collaboration-language-contract.verification.json`
- finalization: `.ai/decisions/WI-679-p0-collaboration-language-contract.finalize.json` (pending merge)
- close: `.ai/decisions/WI-679-p0-collaboration-language-contract.close.json` (pending merge)
