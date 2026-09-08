---
author: AI Cockpit maintainers
title: "WI-674 — P1 Repository responsibility split"
description: "Split cockpit-repository responsibilities into internal modules while preserving public behavior and persistence compatibility."
audience: [maintainer, reviewer, adopter]
workItemId: WI-674-p1-repository-split
status: in_progress
authority: authorized
lastVerifiedBy: WI-674-p1-repository-split
---

[简体中文](WI-674-p1-repository-split.zh-CN.md) · [日本語](WI-674-p1-repository-split.ja.md)

# WI-674 — P1 Repository responsibility split

## Intent

Improve internal separation in `cockpit-repository` without changing the
public API, wire format, persistence layout, error behavior, authorization
semantics, or lifecycle decisions. This is a structural refactor; it does not
claim a measured performance or cognitive benefit.

## Boundary and dependency map

| Module | Responsibility | Main dependencies and consumers |
| --- | --- | --- |
| `lifecycle.rs` | Work Item entry, checkpoint, preflight, finish, verification, recovery, and lifecycle-boundary helpers | Shared repository types and governance/evidence helpers in `lib.rs`; status/readiness projection; evidence store; Outcome projection helpers |
| `evidence_store.rs` | Reusable receipt storage, validity binding, bounded reads, atomic publication, and capability filesystem helpers | Protocol digests and repository identity; consumed by lifecycle and execution-context code; persistence paths and bytes remain unchanged |
| `execution_context.rs` | Repository/runtime execution context, executable identity, staging, shebang and environment identity, and verification-reuse assessment | Git snapshots and evidence-store bindings; consumed by verification and lifecycle code; reuse rules remain unchanged |
| `status_projection.rs` | Repository status, readiness, worktree topology, historical debt, and archive-close projection | Git snapshot and archive/history readers; consumed by lifecycle entry and status callers; projections remain read-only |
| `lib.rs` | Public API, shared protocol types, stable re-exports, and cross-cutting helpers | Re-exports the module APIs and retains the existing serialization and persistence contract |

The dependency direction is intentionally conservative: the extracted modules
use shared repository definitions from `lib.rs`; `lib.rs` re-exports only the
existing public surface and imports narrowly scoped internal helpers where
cross-module callers already depended on them. No new crate or framework was
introduced.

## Compatibility

The base is remote default `origin/main` at
`0b92a420ad76c5f9ce5ea80ae6c6870fcd45b130`. The change moves existing
implementations and adds no protocol fields. Public functions, serialized
artifacts, error paths, persistence paths, and status semantics are intended to
remain byte- and behavior-compatible; this statement is subject to the final
workspace and hosted checks below.

## Verification evidence

The following targeted behavior suites passed after the module moves (107
tests total): lifecycle entry/order; recovery decision/events/revalidation;
receipt store; evidence assurance; repository context; verification context;
verification service; and status projection.

Not yet verified at this checkpoint: full `cargo test --locked --workspace`,
format check, Clippy, documentation/parity checks, Runtime `verify`, hosted PR
checks, and terminal lifecycle records. No behavior defect was found during the
move-only refactor. If one is found, it must be recorded and handled as a
separate governed change rather than hidden in this structural WI.

## Scope exclusions

Outcome P0-A/P0-B behavior, P1-A cognitive-benefit evaluation, P2 first-use
documentation, the future governance-principle WI, other agents' worktrees,
and user-global Agent/MCP configuration are outside this Work Item.
