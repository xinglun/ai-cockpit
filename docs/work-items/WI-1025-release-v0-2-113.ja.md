---
author: AI Cockpit maintainers
workItemId: WI-1025-release-v0-2-113
title: v0.2.113 release
description: reviewed main を四対象の v0.2.113 として公開し、公開 acceptance と正確な lifecycle cleanup を完了する。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1025-release-v0-2-113
---

[English](WI-1025-release-v0-2-113.md) · [简体中文](WI-1025-release-v0-2-113.zh-CN.md)

# WI-1025 — v0.2.113 release

この Work Item は reviewed `main` を v0.2.113 として公開する。immutable tag、公開 Release、四つの対象 artifact、downloaded adopter acceptance、Runtime lifecycle の close、documentation projection、正確な cleanup evidence を結び付ける。

## 境界

- 正式 release の対象は `aarch64-apple-darwin`、`aarch64-unknown-linux-gnu`、`x86_64-unknown-linux-gnu`、`x86_64-pc-windows-msvc` の四つである。
- Intel macOS の release asset、task-progress ledger、歴史的 Release や tag の書き換え、global Agent/MCP configuration の変更は行わない。
- green build や公開 Release だけでは user-visible benefit を証明しない。その benefit は独立した evidence 境界である。
