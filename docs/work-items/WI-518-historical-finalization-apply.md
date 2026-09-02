---
author: AI Cockpit maintainers
title: "WI-518 — historical finalization apply"
description: "Make the published Runtime able to record a real legacy direct merge without a PR when no canonical predecessor exists, with precise fail-closed identity diagnostics."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-518-historical-finalization-apply
lastVerifiedBy: WI-518-historical-finalization-apply
---

[简体中文](WI-518-historical-finalization-apply.zh-CN.md) · [日本語](WI-518-historical-finalization-apply.ja.md)

## Goal

Provide an auditable, repository-bound apply path for a legacy direct merge
(`historicalKind=direct_merge_no_pr`) that has no pull request and no existing
canonical finalization receipt. Preserve immutable history, require the real
Git merge commit and parents, and make resource-context failures actionable.

## Scope

- Rust protocol and repository validation/recording paths.
- CLI help for `finalize-recovery`.
- Focused protocol and repository regressions.
- English, Simplified Chinese, and Japanese command documentation.

The object repository remains read-only. This Work Item does not rewrite
historical receipts, weaken current-runtime checks, invent PRs or human
decisions, or publish a release.

## Acceptance

- A complete direct-merge receipt is accepted by `finalize-recovery` as the
  first canonical record when the predecessor is absent, through the same
  archive, Contract, Git-parent, repository, and current-Runtime checks as
  `finalize`.
- Only an explicit historical low-assurance direct merge may resolve a
  provisional legacy context; foreign worktree/base/provider bindings remain
  fail-closed and identify the binding category.
- `finalize-recovery-plan` exposes deterministic identity facts and human-owned
  fields without inventing branch, authority, PR, or decision facts.
- Immutable predecessor and repository state are unchanged on rejected input.
- Documentation describes the semantic/non-wire and historical-low assurance
  boundary in all three languages.

## Verification

```text
cargo test --locked -p cockpit-protocol --test resource_finalization -- --test-threads=1
cargo test --locked -p cockpit-repository --test resource_finalization_transition -- --test-threads=1
cargo test --locked -p cockpit-cli --test resource_finalization -- --test-threads=1
cargo test --locked --workspace
```

The published-artifact adopter acceptance remains a post-release responsibility
and is not replaced by these source tests.
