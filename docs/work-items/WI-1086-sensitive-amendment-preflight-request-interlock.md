---
author: AI Cockpit maintainers
title: "WI-1086 — sensitive amendment preflight request interlock"
description: "Allow a current review request while preserving receipt integrity and existing blockers."
audience:
  - maintainer
  - reviewer
workItemId: WI-1086-sensitive-amendment-preflight-request-interlock
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1086-sensitive-amendment-preflight-request-interlock
---

[简体中文](WI-1086-sensitive-amendment-preflight-request-interlock.zh-CN.md) · [日本語](WI-1086-sensitive-amendment-preflight-request-interlock.ja.md)

# WI-1086 — sensitive amendment preflight request interlock

This Work Item is in progress. Runtime created its normal before-edit checkpoint on 2026-10-16. Tests, implementation, formal verification and acceptance remain pending. lastVerifiedBy identifies this active projection, not a verified close.

## Benefit

Developers can obtain a current sensitive-amendment review request without waiting on the confirmation that requires that request. Original blocking evidence and strict identity, Contract, decision and snapshot checks remain intact.

## Design and acceptance

The approved v2 plan at commit f49a001ece01209ed9e4cf743f879b7a5931143c limits this repair to the complete native journal/current invalidation binding, request-generation admission and state/outcome/blockers preservation. Cover both documentation-first and governance_red reproductions through CLI and MCP. Extra receipt validation defects require a concrete RED and Raydot scope review.

## Execution boundary

Serial only: one worker and isolated target/wi1086-sensitive-amendment-preflight-request-interlock, at most 900 seconds per command. Preserve original WI-1085; it is a future consumer of the tested repair, not a dependency that must complete first. Shared source, ledgers, builds and lifecycle actions do not run concurrently. No invented lease, human confirmation, Runtime installation, Sentinel operation, merge, close or cleanup.
