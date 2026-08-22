---
author: AI Cockpit maintainers
title: "Human Benefit レポート"
description: "1 つの Task Outcome を evidence から人間向けに要約する projection。"
audience:
  - adopter
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-136
capabilityClaims:
  - human_benefit_report
---

# Human Benefit レポート

Human Benefit レポートは、validated Task Outcome を人間向けに投影したものです。
完了内容、発見、停止、解決、残存リスク、不明点、安全な次の action を示します。

これは別の authority source ではありません。各 claim は Task Outcome の evidence
reference に束縛され、未宣言の利用者効果は unknown のままです。Runtime が生成する
label だけを localize し、Contract の受入れ基準の原文は保持します。

handoff には `work-item outcome` または MCP の `work_item_outcome` を使います。
機械処理用には `--json` で OutcomeV2 と任意の `taskOutcomeReport` を取得します。

[Task Outcome Report](task-outcome-report.ja.md) | [Outcome reference](../reference/outcome-report.ja.md) |
[English](human-benefit-report.md) | [中文](human-benefit-report.zh-CN.md)
