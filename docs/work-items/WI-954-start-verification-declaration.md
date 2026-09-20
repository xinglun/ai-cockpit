---
author: AI Cockpit maintainers
title: "WI-954 — documentation projection recovery"
description: "Restore the post-release documentation-projection recovery without repeating release verification."
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:xinglun
workItemId: WI-954-start-verification-declaration
lastVerifiedBy: WI-954-start-verification-declaration
---

[简体中文](WI-954-start-verification-declaration.zh-CN.md) · [日本語](WI-954-start-verification-declaration.ja.md)

# WI-954 — documentation projection recovery

## Intent

Restore the blocked post-release documentation-projection path after WI-953
was replaced, without repeating release verification or changing Runtime behavior.

## Boundary

This Work Item changes only three-language human documentation projections and
their parity rows. Release artifacts, completed verification evidence, and
object repositories remain out of scope.

## Acceptance

- The projection helper reaches a stable state without launching Cargo tests.
- Three-language pages and parity rows remain aligned.
