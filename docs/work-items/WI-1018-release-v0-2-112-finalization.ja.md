---
author: AI Cockpit maintainers
workItemId: WI-1018-release-v0-2-112-finalization
title: v0.2.112 corrected release finalization
description: immutable な v0.2.111 取消候補を保持し、修正済みの四対象 release、公開 acceptance、正確な lifecycle cleanup を継続する。
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-1018-release-v0-2-112-finalization
terminalArchive: .ai/work-items/archive/WI-1018-release-v0-2-112-finalization.contract.json
terminalVerification: .ai/evidence/WI-1018-release-v0-2-112-finalization.verification.json
terminalDecision: .ai/decisions/WI-1018-release-v0-2-112-finalization.close.json
---

[English](WI-1018-release-v0-2-112-finalization.md) · [简体中文](WI-1018-release-v0-2-112-finalization.zh-CN.md)

# WI-1018 — v0.2.112 release finalization

この successor は synchronized main から修正済み v0.2.112 を公開する。immutable な v0.2.111 の五対象取消候補を保持し、公開 Release、downloaded adopter acceptance、Runtime lifecycle の close、documentation projection、正確な cleanup の evidence を記録する。

## 境界

- 正式 release の対象は `aarch64-apple-darwin`、`aarch64-unknown-linux-gnu`、`x86_64-unknown-linux-gnu`、`x86_64-pc-windows-msvc` の四つである。
- Intel macOS の release asset、task-progress ledger、global Agent/MCP configuration は追加しない。
- green build や公開 Release だけでは user-visible benefit を証明しない。その benefit は独立した evidence 境界である。
