---
author: AI Cockpit maintainers
title: "WI-957 — start authority and typed verification validation"
description: "Reject unsupported authority values before active state and execute typed required verification under its declared identity."
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:sei-rinn
workItemId: WI-957-start-authority-recovery
lastVerifiedBy: WI-957-start-authority-recovery
---

[简体中文](WI-957-start-authority-recovery.zh-CN.md) · [日本語](WI-957-start-authority-recovery.ja.md)

# WI-957 — start authority and typed verification validation

Runtime start rejects unsupported authority values before it writes an active Contract or Summary.
The CLI also plans a typed required verification declaration as its declared
`check` identity, so the verification receipt can satisfy the same finish gate
without falling back to unrelated default workspace tests.
After an additive post-checkpoint amendment invalidates that evidence, one
current replacement verification remains permitted; unrelated control failures
still stop before a subprocess starts.
