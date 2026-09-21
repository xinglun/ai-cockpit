---
author: AI Cockpit maintainers
title: "WI-962 — release-closure bootstrap"
description: "Remove the lifecycle bootstrap cycle that makes resource-bound release closure require hand-maintained documentation projections."
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:sei-rinn
workItemId: WI-962-release-closure-bootstrap
lastVerifiedBy: WI-962-release-closure-bootstrap
---

[简体中文](WI-962-release-closure-bootstrap.zh-CN.md) · [日本語](WI-962-release-closure-bootstrap.ja.md)

# WI-962 — release-closure bootstrap

This successor repairs a Runtime lifecycle bootstrap defect exposed by the
v0.2.103 release-recovery chain. A Work Item must be able to establish its
pre-edit checkpoint before reader-facing documentation projection is required.
The fix will retain documentation integrity at the later, appropriate boundary
and will not alter release evidence or archived Work Item bytes.
