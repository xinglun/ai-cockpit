---
author: AI Cockpit maintainers
title: "Task Outcome Report"
description: "Evidence-bound report sections for what a Work Item delivered, found, stopped, and leaves for human review."
audience:
  - adopter
  - reviewer
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-136
capabilityClaims:
  - task_outcome_report
---

# Task Outcome Report

The Rust Runtime keeps `OutcomeV2` as the stable machine object and adds an
optional `taskOutcomeReport` projection for newly generated outcomes. The
projection is strict, repository-bound, and additive: historical OutcomeV2
bytes without the projection remain readable and are not rewritten.

The report contains explicit sections for outcome summary, task overview,
delivered changes, findings, risks, warnings, limitations, interventions,
forced stops, resolutions, recurrence prevention, avoided impact, residual
risks, human decisions, implementation approach, and evidence. Empty sections
are meaningful `None`; they do not imply that a check or benefit exists.

Each non-empty claim carries repository-local `evidenceRefs`, or is marked
`inference`. Contract intent, scope, acceptance criteria, and authority remain
human-authored source text. The Runtime never infers a user benefit, merge,
release, provider approval, enterprise assurance, or security claim.

## Lifecycle artifacts

After `finish`, the active Work Item contains `<id>.outcome.json` and an
append-only `<id>.events.jsonl`. Events record generated completion, warnings,
stops, and resolutions. `archive` moves the event stream byte-for-byte and
binds its digest in the archive manifest. `close` copies the validated report
into the repository-bound close receipt as `finalReport` with
`finalReportDigest`.

Failed or interrupted lifecycle attempts may also leave Runtime-owned
projections such as `<id>.outcome.finish-blocked.json` or
`<id>.events.finish-recovery.jsonl`. These are audit history, not active
Contracts, so `status` reports them separately as `activeArtifacts` and
`orphanedActiveArtifacts` while `activeWorkItems` remains Contract-based.
Normal `archive` moves recognized variants with the canonical artifacts and
records each digest in `historicalArtifacts`. For a Work Item archived by an
older Runtime, use `ai-cockpit work-item reconcile-artifacts --repo <repository>
--id <id>`; the command requires and validates the existing archive manifest,
moves only identity-bound regular files, and writes an append-only
reconciliation receipt. It never deletes or rewrites historical bytes.

`archive` and `close` are separate boundaries. An archived Work Item remains
pending until an explicit human decision closes it; an orphaned projection is
not evidence that the Work Item is active, but it is a readiness blocker until
it is archived or reconciled.

Event streams reject malformed JSON, unknown fields, foreign repository or
Work Item identity, unsafe evidence paths, secret-like content, duplicate IDs,
and relationships to events that have not already appeared. Corrections must
be new events; historical lines are never silently edited.

## Human handoff

`ai-cockpit work-item outcome --repo <repository> --id <id>` and the MCP
`work_item_outcome` tool use the same validated report and renderer. The
handoff shows the status marker, completed work, problems, stops, resolutions,
risks, unknowns, decisions, verification, impact, and next action. Runtime
labels are localized; Contract source text remains in its original language.

This report does not authorize a merge, publication, provider approval, or
organizational decision. Event-sourced paused/blocked/stale/cancelled/rollback
state reconstruction remains a separate recovery capability.

[Human Benefit Report](human-benefit-report.md) | [Outcome reference](../reference/outcome-report.md) |
[Features](README.md) | [中文](task-outcome-report.zh-CN.md) | [日本語](task-outcome-report.ja.md)
