---
author: AI Cockpit maintainers
title: WI-677 — WI-657 ライフサイクル並行性・復旧 successor
description: 現在の default branch から P2-C を再検証し、並行ライフサイクル commit を直列化する。
workItemId: WI-677-wi657-lifecycle-concurrency-successor
audience:
  - contributor
  - maintainer
  - reviewer
status: recovered
authority: human-authorized
lastVerifiedBy: WI-677-wi657-lifecycle-concurrency-successor
---

# WI-677 — WI-657 ライフサイクル並行性・復旧 successor（recovered）

この predecessor は P2-C を配信したが、remote default branch の進行後に archive 済み
PR が conflict になった。WI-681 が新しい base から同じ境界を再配信する。履歴と
証拠は書き換えない。アーカイブ済み
WI-657 branch は同一プロセスの一時ファイル名衝突を修正したが、default branch
から遅れており、rollback が競合した成功を上書きし得る gap も記録していた。
その歴史的証拠は変更せず、本 Work Item で再検証する。

## 境界

`finish`、`archive`、`close`、recovery decision の記録、active artifact
reconciliation は、Work Item ごとの OS advisory lock を共有する。lock は Git
管理外の `.ai/locks/` runtime directory に安定した inode として残し、handle または
process の終了時に OS が解放する。これにより同一 process の thread と独立した
CLI process の両方を直列化し、database、global cache、新しい crate は導入しない。
`atomic_write` も process id と既存の atomic sequence counter を組み合わせ、同一
process 内で一時 pathname を再利用しない。

lock は `finish` の失敗 projection の保存まで覆う。敗者は commit 済み状態を読み、
business-level rejection を返すため、古い snapshot で成功した sibling の状態を
復元できない。既存の JSON schema、lifecycle 名、authorization 検査、archive layout
は維持する。

## 制御された検証

`lifecycle_concurrency.rs` は次を検証する。

- 同じ Work Item を二つの thread が同時に finish すること。
- 二つの独立した test process が同時に finish すること。
- archive と close の競合で各操作に commit が一つだけ存在すること。
- active projection の欠落と archive projection の破損が commit 前に fail closed すること。

競合後の summary、outcome、archive manifest、close decision を JSON として解析し、
raw filesystem race error を拒否する。将来の crash-consistency の制限は明示的な
recovery または unknown として残し、projection から authorization を推測しない。

## 検証と残余リスク

Rust、format、Clippy、documentation、governance の各検査を successor の正確な
head に対して実行する。この lock が保護するのは本 repository 実装を利用する process
だけであり、API を迂回する外部 writer の安全性は主張しない。
