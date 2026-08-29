---
author: AI Cockpit maintainers
title: Verification evidence reuse decision
description: Source-backed decision boundary for safe, measurable verification reuse.
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Verification evidence reuse decision

[简体中文](verification-evidence-reuse.zh-CN.md) · [日本語](verification-evidence-reuse.ja.md)

Evidence classifiers decide whether a receipt is fresh, stale, or unknown; the
planner consumes that decision and the bounded adapter performs the required
execution. A fresh receipt can skip only an allowlisted, non-protected node.
Unknown or stale evidence causes a normal rerun. Protected security, scope,
governance, coverage, and source-bound nodes are never skipped by reuse.

## Required bindings

Reuse requires exact equality for the base/head revision, normalized changed
paths, command and command digest, environment/toolchain descriptors, policy
and stage, runner, repository/Work Item identity, and output receipt digest.
Changing any binding invalidates the candidate. The Runtime never infers safety
from age, timing, cache labels, or a provider result.

## Cost and limits

The adapter reports planned, executed, reused, stale, unknown, and protected
call counts. A useful optimization is demonstrated only when an unrelated
change reduces actual calls while protected calls remain unchanged. No provider
wait, human-wait, P95, or assurance improvement is inferred from a local run.

The source Python/Make orchestration and JSONL records remain reference
material. Rust keeps the same trust boundary through typed, repository-bound
receipts and explicit `--repo` context.
