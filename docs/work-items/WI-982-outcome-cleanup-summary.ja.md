---
author: AI Cockpit maintainers
title: "WI-982 — Outcome cleanup summary"
description: "検証済み cleanup の件数、正確な resource identity、保持理由、evidence 参照、次の action を共有 human Outcome 本文へ投影する。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:user
workItemId: WI-982-outcome-cleanup-summary
lastVerifiedBy: WI-982-outcome-cleanup-summary
---

[English](WI-982-outcome-cleanup-summary.md) · [简体中文](WI-982-outcome-cleanup-summary.zh-CN.md)

# WI-982 — Outcome cleanup summary

この Work Item は、検証済みの resource-finalization receipt から生成される
human-readable cleanup summary を改善する。共有 projection は deleted、retained、
unknown の件数、pull request・branch・worktree の正確な identity、receipt に束縛
された理由と evidence 参照、既存の実行可能な次の action を示す。CLI と MCP は同じ
完全な本文を利用し、別の証拠がない provider display confirmation は unknown のままにする。

旧 finalization JSON は additive optional field と default により読み取り可能なまま
保つ。この Work Item は authorization、verification、release、provider display の
意味を変更しない。
