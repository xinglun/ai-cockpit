---
author: AI Cockpit maintainers
title: WI-683 — P2-C ライフサイクル並行性とリカバリの再配信
description: 最新の default branch から一意の Work Item identity でライフサイクル並行性とリカバリ境界を再配信する。
workItemId: WI-683-p2c-lifecycle-concurrency-recovery
audience:
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-683-p2c-lifecycle-concurrency-recovery
---

# WI-683 — P2-C ライフサイクル並行性とリカバリの再配信

この Work Item は、以前の再配信が無関係な WI-681 と identity collision を起こした
ため、現在の `origin/main` から P2-C 境界を再配信します。WI-677 の archive、evidence、
recovery decision は不変の履歴記録として保持し、この Work Item が新しい実装と検証を
担当します。

## 境界

`finish`、`archive`、`close`、recovery-decision の記録、active-artifact reconciliation
は、無視対象の `.ai/locks/` runtime directory にある Work Item 単位の OS file lock
を共有します。lock は安定した inode として保持され、handle または process の終了時に
OS が解放します。これにより database、global repository cache、新しい crate を追加せず、
thread と独立した CLI process の両方を直列化します。`atomic_write` は既存の atomic
sequence counter に process ID を組み合わせ、同一 process の temporary pathname 再利用も
防ぎます。

lock は `finish` の failure projection persistence と主遷移の両方を保護します。そのため
敗者は commit 済み状態を観測して business-level rejection を返し、成功した sibling に
古い試行前 snapshot を上書きできません。既存の JSON schema、ライフサイクル名、authorization
check、archive layout、predecessor bytes は変更しません。

## 受け入れた検証

`lifecycle_concurrency.rs` は次を検証します。

- 同じ Work Item を二つの thread が finish するケース。
- 同じ Work Item を二つの独立 test process が finish するケース。
- archive と close の並行呼び出しで、それぞれ commit が一回だけになること。
- active projection の欠落と corrupt archive projection が、新しい commit 前に fail closed すること。

並行実行後に final summary、outcome、archive manifest、close decision を JSON として解析し、
raw filesystem race error を拒否します。crash consistency の制限は明示的な recovery または
unknown state として扱い、projection から authorization を推測しません。

## 検証と残余リスク

Rust、format、Clippy、documentation、governance、hosted check は正確な successor head で
実行します。この lock が保護するのは本 repository implementation を使う process です。
それを迂回する外部 writer の安全性は主張しません。
