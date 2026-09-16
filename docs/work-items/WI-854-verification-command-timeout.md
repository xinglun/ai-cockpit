---
author: AI Cockpit maintainers
title: "WI-854 — governed verification command timeout"
description: "Bound verification command execution while preserving compatibility and durable evidence."
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
workItemId: WI-854-verification-command-timeout
lastVerifiedBy: WI-854-verification-command-timeout
terminalArchive: .ai/work-items/archive/WI-854-verification-command-timeout.contract.json
terminalVerification: .ai/evidence/WI-854-verification-command-timeout.verification.json
terminalDecision: .ai/decisions/WI-854-verification-command-timeout.close.json
---

[简体中文](WI-854-verification-command-timeout.zh-CN.md) · [日本語](WI-854-verification-command-timeout.ja.md)

# WI-854 — governed verification command timeout

WI-854 adds a finite, Contract-authorized timeout for verification commands.
The default remains compatible, invalid overrides are rejected before spawn,
and timeout attempts preserve their exit, deadline, elapsed-time, and log
identity for later diagnosis. The effective timeout is part of verification
reuse identity, so changing it cannot silently reuse stale evidence.
