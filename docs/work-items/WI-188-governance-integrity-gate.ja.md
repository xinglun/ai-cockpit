---
author: AI Cockpit maintainers
title: "WI-188 — Governance integrity gate"
description: "current Work Item、evidence、decision、Outcome、documentation、CI coverage を動的かつ fail-closed に検査します。"
audience:
  - maintainer
  - reviewer
workItemId: WI-188-governance-integrity-gate
status: current
authority: canonical
lastVerifiedBy: WI-188-governance-integrity-gate
---

# WI-188 — Governance integrity gate

WI-188 は固定された WI-177〜WI-186 parity list を repository inventory に置き換え
ます。gate は active と archived Contract を自動検出し、Cargo metadata と Contract/archive
作成時刻から current release cycle を導出して、current Summary、archive、
verification、terminal decision、Outcome、三言語 parity binding を検査します。
以前の record は historical または legacy として監査可能なまま保持し、未知の
current problem は fail closed になります。

archive 済み feature branch Work Item は、Runtime finalize receipt が unmerged PR、
branch present、clean worktree、`blocked` disposition、唯一の failure code
`unmerged_pull_request`、unknown code なし、`awaiting_merge_close` audit token で始まる reason を証明する
場合だけ `awaiting_merge_close` になります。この receipt は terminal closure では
ないため、default branch では正確な close または recovery decision が引き続き必須です。
この例外は repository identity、archived Contract の raw SHA-256、verification Runtime
identity、実際の remote default branch も bind し、PR、branch、worktree、Contract
resource context の identity が内部的に一致することを必須とします。

CI は単一 manifest から documentation、workflow、conformance、performance、
release gate を実行します。Workspace package test は `cargo metadata` から導出し、
直列実行して deterministic JSON receipt に bind します。

[English](WI-188-governance-integrity-gate.md) ·
[简体中文](WI-188-governance-integrity-gate.zh-CN.md)
