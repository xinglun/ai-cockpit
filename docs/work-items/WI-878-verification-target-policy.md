---
author: AI Cockpit maintainers
workItemId: WI-878-verification-target-policy
title: Verification target cache policy
description: Bound Cargo verification cache growth while preserving dependency reuse.
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-878-verification-target-policy
---

[简体中文](WI-878-verification-target-policy.zh-CN.md) · [日本語](WI-878-verification-target-policy.ja.md)

# WI-878 — Verification target cache policy

This Work Item makes the Runtime launch Cargo verification with
`CARGO_INCREMENTAL=0` and one stable user-cache target directory. It preserves
dependency reuse, leaves non-Cargo commands unchanged, and documents the safe
repository-local incremental cleanup boundary. It does not modify object
repositories, release artifacts, or verification authorization semantics.
