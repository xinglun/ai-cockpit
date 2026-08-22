---
author: AI Cockpit maintainers
title: "Task Outcome Events"
description: "Append-only event rules for the Rust Task Outcome projection."
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-136
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

`archive` moves the stream byte-for-byte and binds `eventsDigest` in the archive
manifest. `close` validates the archived stream before writing the final report.
The stream is an evidence source, not a lifecycle authority: it cannot approve
scope, merge, release, provider identity, or enterprise compliance.

Event-sourced paused/blocked/stale/cancelled/rollback reconstruction is not yet
part of this boundary and remains a separate recovery capability.

[Task Outcome Report](../features/task-outcome-report.md) | [Outcome reference](outcome-report.md) |
[中文](../features/task-outcome-report.zh-CN.md) | [日本語](../features/task-outcome-report.ja.md)
