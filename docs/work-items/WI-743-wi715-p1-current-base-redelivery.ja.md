---
author: AI Cockpit maintainers
title: "WI-743 — WI-715 current-base performance decision redelivery"
description: "古い未マージ PR を復活させず、WI-715 の declined な large-history 候補を最新 default base から再検証します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-743-wi715-p1-current-base-redelivery
lastVerifiedBy: WI-743-wi715-p1-current-base-redelivery
---

[English](WI-743-wi715-p1-current-base-redelivery.md) · [简体中文](WI-743-wi715-p1-current-base-redelivery.zh-CN.md)

# WI-743 — WI-715 current-base performance decision redelivery

## Intent

最新の remote default base から WI-715 large-history status 候補の decision を再配信します。証拠に基づく decline とその制限、および production code を変更しない境界を新しい review 可能な Work Item に保持します。PR #708 は復活させません。

## Lineage と境界

- Current base: `origin/main` at `838ae745511942cb55dd7ac30c319cc5bc95e74d`.
- Predecessor: `WI-715-p1-large-history-status`。その archive、evidence、decision および PR #708 は predecessor branch 上の immutable audit history として保持されます：<https://github.com/xinglun/ai-cockpit/pull/708>。
- この Work Item は current-base verification、evidence binding、三言語の governance record のみを対象にします。Runtime behavior、performance threshold、measurement semantics、Outcome/schema、exit code、authorization、persistence、他 agent の作業は変更しません。

## Evidence-backed decision

Predecessor の current-base experiment は同じ Runtime line の
`d1141480fb7a045979098480c3770d06002e2a87` で実行されました。forward order の warm status p50 delta は `-1.171%`、reverse order は `-0.302%` でした。登録された threshold は両方向とも `5%` なので、candidate は引き続き declined です。両 gate は filesystem comparison key が利用できないため fail closed しました。p99 と複数の phase/resource metrics は利用できず、zero ではなく unavailable として記録されています。

その後の default-base commits は governance、documentation、test の変更だけで、production Runtime source の変更はありません。この事実は decline の current-base semantic revalidation を支えますが、新しい latency benefit claim ではありません。candidate implementation は merge されていません。

## Current state

Runtime lifecycle は `checkpointed` です。current-base verification、reviewed PR delivery、finalization、archive、human close は pending です。authoritative な current evidence は `.ai/evidence/WI-743-wi715-p1-current-base-redelivery.verification.json` と Contract が列挙する predecessor external records です。

## Decision boundary

Decision: `declined` を保持します。latency、CPU、I/O、memory、resident-MCP の benefit は主張しません。今後の optimization は reviewed default base から開始し、trustworthy な environment comparator と phase-level evidence を先に示す必要があります。
