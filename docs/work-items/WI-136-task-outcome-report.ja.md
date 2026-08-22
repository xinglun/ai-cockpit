---
author: AI Cockpit maintainers
workItemId: WI-136-task-outcome-report
title: Rust-native Task Outcome と Human Benefit report
description: evidence-bound report projection、append-only event source、lifecycle-bound artifact を追加する。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-136-close-verification
---

# WI-136 — Rust-native Task Outcome と Human Benefit report

## Intent

現在の Rust Runtime には narrow な OutcomeV2 と human handoff があります。reference
surface は完了、発見、停止、解決、リスク、不明点、証拠、recovery condition も明示します。
この WI は presentation を authority に変えず、その projection を追加します。

## 境界

- 新しい OutcomeV2 は evidence-bound claim と安定した section 名を持つ strict additive
  `taskOutcomeReport` を含みます。
- `finish` は typed report JSON、Markdown projection、append-only の `<id>.events.jsonl` を書き、
  `archive` は移動して digest を束縛し、`close` は repository-bound decision receipt に
  validated `finalReport` と digest を保存します。
- event identity、repository/Work Item binding、関係順序、安全でない path、secret らしい内容は
  fail closed。過去の bytes は変更しません。
- CLI と MCP は同じ localized renderer を使います。Contract 原文、人間の判断、外部 provider claim、
  release truth は変更しません。

## 対象外

新しい lifecycle state、完全な event-sourced paused/blocked/stale/cancelled/rollback reconstruction、
adopter capability manifest、第二技術 stack acceptance、provider identity、global Agent/MCP 設定、
reference Python/Make/V1 asset のコピーは別 boundary です。

## 受入れ

- Protocol test が strict report schema、unknown field 拒否、claim provenance を確認します。
- Repository test が report/event 生成、malformed/foreign event 拒否、archive digest binding、
  close final report binding を確認します。
- CLI と MCP は同じ report を表示し、三言語の heading と Contract-language の受入れ原文を保持します。
- English、簡体中文、日本語の feature/reference 文書が implemented/deferred boundary を正確に示します。

## 検証

archive された evidence と close decision が現在の検証記録です。

- `.ai/evidence/WI-136-task-outcome-report.verification.json`
- `.ai/work-items/archive/WI-136-task-outcome-report.archive.json`
- `.ai/decisions/WI-136-task-outcome-report.close.json`
