---
author: AI Cockpit maintainers
workItemId: WI-1025-release-v0-2-113
title: v0.2.113 release
description: Publish the reviewed main revision as the four-target v0.2.113 release with public acceptance and exact lifecycle cleanup.
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1025-release-v0-2-113
---

[简体中文](WI-1025-release-v0-2-113.zh-CN.md) · [日本語](WI-1025-release-v0-2-113.ja.md)

# WI-1025 — v0.2.113 release

This Work Item publishes the reviewed `main` revision as v0.2.113. It binds the immutable tag, public Release, four supported target artifacts, downloaded adopter acceptance, Runtime lifecycle closure, documentation projection, and exact cleanup evidence.

## Boundaries

- The official release targets are `aarch64-apple-darwin`, `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`.
- It does not generate an Intel macOS release asset, add a task-progress ledger, rewrite historical Releases or tags, or modify global Agent/MCP configuration.
- A green build or public Release does not by itself prove user-visible benefit; that remains an explicit evidence boundary.
