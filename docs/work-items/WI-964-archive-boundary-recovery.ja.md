---
author: AI Cockpit maintainers
title: "WI-964 — immutable archive boundary recovery"
description: "archive format を生成元で修正し、mutable whitespace validation と immutable archive integrity を分離する。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:sei-rinn
workItemId: WI-964-archive-boundary-recovery
lastVerifiedBy: WI-964-archive-boundary-recovery
---

[English](WI-964-archive-boundary-recovery.md) · [简体中文](WI-964-archive-boundary-recovery.zh-CN.md)

# WI-964 — immutable archive boundary recovery

この successor は WI-963 の失敗した verification record を保持する。task-report
Markdown の EOF format を生成時に修正し、whitespace validation は mutable candidate
path のみに適用する。immutable archive bytes は digest と archive-integrity check で
引き続き管理し、本 WI はそれらを書き換えず、release または closure 完了も主張しない。
