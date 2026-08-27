---
author: AI Cockpit maintainers
title: "WI-325 — reference file comparison batch 05"
workItemId: WI-325-reference-file-comparison-batch-05
description: "Compare the next nine pinned reference documentation paths and register their Rust-native semantic boundaries."
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-325-reference-file-comparison-batch-05
---

# WI-325 — reference file comparison batch 05

## Intent and boundary

Compare the next nine paths from the pinned reference commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` one file at a time. Preserve
evidence-backed semantics for adopters without copying the reference
repository's Python, Make, fixture, or internal-progress implementation.

The shared Rust Runtime remains external and every repository remains bound by
an explicit `--repo`. The Cursor adopter feedback was used as an external
observation: stable Outcome and entry-gate behavior are already covered by
existing Runtime work; optional host UX conveniences are not silently claimed
by this parity batch.

## File-level comparison

| Pinned reference path | Classification | Rust/adopter counterpart and boundary |
| --- | --- | --- |
| `docs/features/task-outcome-report-self-check.md` | `reference-only` | Current Outcome/report/event pages and `.ai/README.md`; the source WI22 progress and release claims are historical and are not copied. |
| `docs/fixtures/real-fixture-evidence.ja.md` | `implemented-different-by-design` | Japanese fixture layout, Release adopter/upgrade acceptance, distribution, and adversarial-validation pages; local, provider, and enterprise evidence remain separate. |
| `docs/fixtures/real-fixture-evidence.md` | `implemented-different-by-design` | Rust fixtures plus the immutable Release adopter/upgrade harness; the source seven-stack `make`/Python matrix is not a Runtime capability. |
| `docs/guides/lightweight-verification.ja.md` | `implemented-different-by-design` | Japanese verification route, semantics, CI quality, and cost pages; warnings never authorize and critical failures stop. |
| `docs/guides/lightweight-verification.md` | `implemented-different-by-design` | Rust stage-aware verification and dynamic light/standard/strict routing; source checker scripts are not copied. |
| `docs/guides/lightweight-verification.zh-CN.md` | `implemented-different-by-design` | Chinese verification route, semantics, CI quality, and cost pages with the same fail-closed boundary. |
| `docs/installation.md` | `implemented-different-by-design` | Reader-first installation, Release distribution/security, and `.ai/README.md`; installation does not attach a repository or imply calibration. |
| `docs/maintainers/adding-or-classifying-a-check.md` | `implemented-different-by-design` | Versioned gate manifest, dynamic route, runner, and regression checks; required profiles, dependencies, skips, and hard failures remain explicit. |
| `docs/maintainers/task-outcome-events.md` | `implemented-different-by-design` | Typed Rust Task Outcome events, append-only corrections, privacy validation, archive binding, and human handoff. |

## Non-goals

This Work Item does not add Runtime commands, copy source Python/Make/YAML or
fixture files, require `Makefile.ai`, change Cursor or global Agent/MCP
configuration, or implement optional `close-gap`, automatic controls templates,
or host-panel expansion. Those are separate product decisions, not hidden
parity.

## Acceptance and evidence

1. All nine pinned paths are read and have exactly one inventory record with a
   non-empty, evidence-backed reason.
2. The generated inventory records eight `implemented-different-by-design`
   entries and one `reference-only` entry for `WI-325-reference-file-comparison-batch-05`;
   none remains deferred or is marked as a migrate gap.
3. English, Simplified Chinese, and Japanese parity pages and this Work Item
   record agree on the source pin, classifications, and semantic boundaries.
4. Internal progress claims, source-specific fixture results, and unrun
   provider/enterprise assurance are not presented as current Runtime facts.
5. Installed Runtime verification, documentation/conformance checks, hosted
   CI, lifecycle closure, and exact branch/worktree cleanup provide the
   terminal evidence. Historical evidence is not rewritten.

[中文](WI-325-reference-file-comparison-batch-05.zh-CN.md) ·
[日本語](WI-325-reference-file-comparison-batch-05.ja.md)
