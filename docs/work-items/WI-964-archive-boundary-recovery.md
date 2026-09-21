---
author: AI Cockpit maintainers
title: "WI-964 — immutable archive boundary recovery"
description: "Repair archive formatting at its source and keep mutable whitespace validation separate from immutable archive integrity."
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:sei-rinn
workItemId: WI-964-archive-boundary-recovery
lastVerifiedBy: WI-964-archive-boundary-recovery
---

[简体中文](WI-964-archive-boundary-recovery.zh-CN.md) · [日本語](WI-964-archive-boundary-recovery.ja.md)

# WI-964 — immutable archive boundary recovery

This successor preserves WI-963's failed verification record. It repairs the
task-report Markdown EOF format at generation time and validates whitespace
only in mutable candidate paths. Immutable archive bytes remain governed by
their digest and archive-integrity checks; this Work Item does not rewrite
them or claim a release or closure result.
