---
author: AI Cockpit maintainers
workItemId: WI-1018-release-v0-2-112-finalization
title: v0.2.112 corrected release finalization
description: Continue the corrected four-target release after the immutable v0.2.111 canceled candidate, with public acceptance and exact lifecycle cleanup.
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-1018-release-v0-2-112-finalization
terminalArchive: .ai/work-items/archive/WI-1018-release-v0-2-112-finalization.contract.json
terminalVerification: .ai/evidence/WI-1018-release-v0-2-112-finalization.verification.json
terminalDecision: .ai/decisions/WI-1018-release-v0-2-112-finalization.close.json
---

[简体中文](WI-1018-release-v0-2-112-finalization.zh-CN.md) · [日本語](WI-1018-release-v0-2-112-finalization.ja.md)

# WI-1018 — v0.2.112 release finalization

This successor publishes the corrected v0.2.112 source from synchronized main. It preserves the immutable v0.2.111 canceled five-target candidate and records the public Release, downloaded adopter acceptance, Runtime lifecycle closure, documentation projection, and exact cleanup evidence.

## Boundaries

- The official release has four targets: `aarch64-apple-darwin`, `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`.
- It does not generate an Intel macOS release asset, add a task-progress ledger, or modify global Agent/MCP configuration.
- A green build or public Release does not by itself prove user-visible benefit; that remains an explicit evidence boundary.
