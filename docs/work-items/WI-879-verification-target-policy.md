---
author: AI Cockpit maintainers
workItemId: WI-879-verification-target-policy
title: Verification target cache policy replacement
description: Carry the shared non-incremental Cargo verification target policy into a reviewed successor while repairing the archived WI-878 quality failure.
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-879-verification-target-policy
---

[简体中文](WI-879-verification-target-policy.zh-CN.md) · [日本語](WI-879-verification-target-policy.ja.md)

# WI-879 — Verification target cache policy replacement

This successor carries the WI-878 implementation from the latest remote main
line into a fresh reviewed branch and removes the hosted-quality regression
that was discovered after WI-878 was archived. Cargo verification keeps
`CARGO_INCREMENTAL=0` and one stable user-cache target directory so dependency
reuse does not create a new incremental tree for every run. Non-Cargo commands,
object repositories, Issue #851 recovery, and release publication remain out
of scope.

The archived WI-878 evidence remains immutable; this Work Item records only
the replacement implementation, documentation projection, and fresh quality
evidence.
