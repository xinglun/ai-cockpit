---
author: AI Cockpit maintainers
title: "WI-678 — P1 historical finalization inventory 再配信"
description: "最新の default branch から bounded な historical finalization candidate 再利用を再検証する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-678-p1-historical-inventory-redelivery
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-678-p1-historical-inventory-redelivery
---

[English](WI-678-p1-historical-inventory-redelivery.md) · [简体中文](WI-678-p1-historical-inventory-redelivery.zh-CN.md)

# WI-678 — P1 historical finalization inventory 再配信

## Intent と仮説

P0 の計測では historical finalization inventory が status 経路の主な bottleneck
でした。1 回の immutable observation の中で decisions directory を一度だけ読み、
transition candidate list を再利用すれば、governance semantics を変えずに I/O の
重複を減らせるという仮説です。

この Work Item は `origin/main` の
`c1f1f1d9f17ec2242a59f287a4368bd6bda5993e` から再配信します。draft PR #671 は
hosted quality gate に失敗したため、監査コンテキストとしてのみ保持し、review や
merge authority とはみなしません。

## 境界

対象は historical inventory/resolver 経路、bounded counter test、三言語文書、
三つの reference-parity 行です。candidate data は同一 immutable observation 内だけ
で再利用します。process-global cache、cross-request snapshot、authorization の変更、
evidence rebinding、無関係な refactor は行いません。

## Authorization record

2026-09-08、`human:user-request` により、user-requested pause/resume boundary を含む
provider または lifecycle interruption 後も Agent が継続することを認可されました。
正確な scope と preserved evidence は Contract に記録します。これは GitHub review や
approval の偽造ではありません。

## Acceptance と verification

- 同一 toolchain、scenario、environment、修正済み order-preserving benchmark semantics
  で baseline と candidate の fresh raw samples を取得します。
- warm-up 数、sample 数、percentile method、environment、取得不能な metric を保持し、
  p95 は 20 warm samples 以上、100 samples 未満の p99 は unknown とします。
- decisions directory scan の bounded reuse を示し、finalization、malformed record、
  filename digest、fork、missing directory、isolation、error、evidence binding、output
  order、exit code を保持します。
- Runtime lifecycle と documentation/parity checks を実行します。fresh target-path
  evidence と correctness equivalence が揃った場合だけ benefit を受け入れ、そうで
  なければ reject を記録して benefit を主張しません。

## Evidence と status

- predecessor audit: PR #671, `https://github.com/xinglun/ai-cockpit/pull/671`
- base: `origin/main` の `c1f1f1d9f17ec2242a59f287a4368bd6bda5993e`
- Runtime evidence: `.ai/evidence/WI-678-p1-historical-inventory-redelivery.verification.json`
- terminal records は到達時に Runtime が `.ai/work-items/archive` と `.ai/decisions` に生成します。

fresh measurements、correctness checks、hosted review、terminal Runtime Outcome が結び
付くまで Work Item は `in_progress` です。
