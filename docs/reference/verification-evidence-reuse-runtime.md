---
author: AI Cockpit maintainers
title: Verification evidence reuse runtime
description: How the Rust Runtime plans bounded evidence reuse without weakening protected verification.
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Verification evidence reuse runtime

[简体中文](verification-evidence-reuse-runtime.zh-CN.md) · [日本語](verification-evidence-reuse-runtime.ja.md)

AI Cockpit separates planning from execution. A request-scoped plan may mark a
node `execute` or `reuse`, but only the declared execution route can run the
command. A reused result is evidence, not a permission to skip a required gate.

## What may be reused

The Runtime can reuse a passed, non-expired receipt only when the repository,
Work Item, base/head snapshot, normalized change set, command, scope, stage,
runner/toolchain, policy, and output identity all match. Content, diff, and
environment bindings are dimensions of the existing verification node; they do
not create a second checker API. A missing, malformed, stale, foreign, or
contradictory receipt is `unknown` and the required node executes again.

Scope, security/trust, governance, coverage, identity, source-bound, and supply
chain gates remain protected. They execute whenever policy or stage requires
them, even if an advisory cost estimate suggests a shortcut. `stage_not_applicable`
is not execution evidence.

## Observable evidence

Verification results record planned, executed, reused, stale-rerun,
unknown-rerun, protected-node, timing, worker, and receipt-identity facts.
The lower execution count must come from an actual adapter call-count
observation; elapsed time or a cache label is not proof. Each Work Item still
gets its own identity-bound receipt, even when physical execution is shared.

This is a Rust-native semantic boundary. The reference source's Python module,
Make targets, and JSON wire shape are not copied into the Runtime or an adopter.
