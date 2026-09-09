---
author: AI Cockpit maintainers
title: "WI-718 — WI-717 Work Item documentation projection"
description: "Closed WI-717 に不足している三言語 documentation projection を補完します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-718-wi717-doc-projection
lastVerifiedBy: WI-718-wi717-doc-projection
---

[English](WI-718-wi717-doc-projection.md) · [简体中文](WI-718-wi717-doc-projection.zh-CN.md)

# WI-718 — WI-717 Work Item documentation projection

## Intent

すでに closed となった WI-717 に不足している三言語 Work Item pages を補完し、
WI-717 の Runtime-generated finalization と close receipts を書き換えずに保持します。

## Boundary

狭い documentation と governance-record projection です。Runtime behavior、
Outcome semantics、machine JSON、exit codes、authorization semantics、
historical archive/evidence bytes は変更しません。

## Acceptance

- 三言語 WI-717 pages が immutable terminal evidence と approved close を示します。
- WI-717 Runtime-generated finalization と close receipts が byte-for-byte で保持されます。
- closed Work Item promotion、documentation acceptance、governance integrity checks が pass します。
