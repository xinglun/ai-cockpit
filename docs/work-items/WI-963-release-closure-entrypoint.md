---
author: AI Cockpit maintainers
title: "WI-963 — release-closure entrypoint"
description: "Repair the narrow, fail-closed Work Item entrypoint bootstrap path for release closure."
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:sei-rinn
workItemId: WI-963-release-closure-entrypoint
lastVerifiedBy: WI-963-release-closure-entrypoint
---

[简体中文](WI-963-release-closure-entrypoint.zh-CN.md) · [日本語](WI-963-release-closure-entrypoint.ja.md)

# WI-963 — release-closure entrypoint

This successor narrows WI-962 to a fail-closed entrypoint correction: a current
Work Item's reader documentation projection must not prevent its first
checkpoint. Authorization, evidence, finalization, archive, and close rules
remain unchanged; the later documentation boundary remains enforceable.
