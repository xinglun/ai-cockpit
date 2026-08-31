---
author: AI Cockpit maintainers
title: "Task Outcome Events"
description: "Append-only event rules for the Rust Task Outcome projection."
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-457
---

# Task Outcome Events

The Rust Runtime stores generated Task Outcome events at
`.ai/work-items/active/<id>.events.jsonl`. Each line is a strict
`TaskOutcomeEvent` with repository and Work Item identity. `finish` creates the
initial completion event and records generated warnings, stops, and resolutions.

The stream is append-only. A correction is a new event with a relationship to a
previous event; deleting or rewriting a historical line is not a valid repair.
The validator rejects malformed JSON, unknown fields, duplicate IDs, foreign
identity, unsafe evidence paths, secret-like details, and references to events
that have not appeared earlier in the stream.

The event-family vocabulary is explicit: `finding`, `risk`, `warning`,
`confirmation`, `stop`, `resume`, `resolution`, `risk-accepted`,
`check-pass-after-fix`, `prevention`, `completed`, and `cancelled`. The Runtime
also retains the historical `blocked` and `recovered` events. Corrections and
supersessions use `event_corrected` or `event_superseded` with a prior
`correctionOf` event ID; an unbound correction is rejected.

`finding` and `risk` events carry a deterministic `findingFingerprint`. Rust
derives it from the event family, whitespace-normalized detail, and sorted
repository-relative evidence references. A repeated fingerprint is rejected
unless it is an explicitly linked correction/supersession, so a post-fix
recurrence is a new auditable event rather than a mutation of the original.

`archive` moves the stream byte-for-byte and binds `eventsDigest` in the archive
manifest. `close` validates the archived stream before writing the final report.
The stream is an evidence source, not a lifecycle authority: it cannot approve
scope, merge, release, provider identity, or enterprise compliance.

The Rust Runtime performs the equivalent generation and validation in-process;
the reference Python scripts are semantic source material, not Runtime
dependencies. Event counts are not performance scores.

Blocked lifecycle gates are projected as red active Outcomes with a deterministic
failed gate and recovery condition. A later `work-item recover` receipt may
authorize a retry or explicitly link a successor, but it never rewrites the
blocked predecessor or makes verification green. The receipt binds the
predecessor Contract/Summary/Outcome/event digests and current Runtime; later
decisions are appended under digest-suffixed paths.

This is semantic parity, not source wire compatibility: Rust keeps its strict
`TaskOutcomeEvent` shape and repository binding instead of copying the template's
Python schemas or Make targets. Publication/provider evidence, locale projection,
and Status/PR summaries remain separate evidence and presentation boundaries.

[Task Outcome Report](../features/task-outcome-report.md) | [Outcome reference](outcome-report.md) |
[中文](../features/task-outcome-report.zh-CN.md) | [日本語](../features/task-outcome-report.ja.md)
