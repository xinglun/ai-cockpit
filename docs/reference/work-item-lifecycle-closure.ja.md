---
author: AI Cockpit maintainers
title: Work Item ライフサイクルのクローズ
description: archive、merge、正確な cleanup 後に reviewed Work Item を安全に閉じる。
audience: [adopter, maintainer, reviewer]
status: implemented
authority: translation
canonical: docs/reference/work-item-lifecycle-closure.md
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Work Item ライフサイクルのクローズ

[English](work-item-lifecycle-closure.md) · [简体中文](work-item-lifecycle-closure.zh-CN.md) · [日本語](work-item-lifecycle-closure.ja.md)

クローズは `start → preflight → checkpoint → verify → finish → archive` 後の最終 handoff であり、
branch deletion の近道ではありません。Runtime は reviewed PR、正確な Work Item head、archived
Contract/Summary/evidence、同期済み base、clean worktree、remote branch 不在をすべて検証します。

## 通常の route

```text
verify → finish/archive → push → reviewed PR と hosted checks → merge
→ finalize → finalize-verify → close → synchronize and clean
```

close は PR state、branch/head identity、base fast-forward、archive/decision receipt、clean
worktree、remote branch 不在を順に検証し、その後に対象の local Work Item branch だけを削除します。
provider の auto-delete でこの証明を迂回してはいけません。

`ready_on_base` は呼び出し元 checkout が clean で同期済み default branch 上にある状態です。
`closed_but_current_worktree_detached` は close 済みだが別の検証済み worktree が base を所有する
状態なので、表示された base worktree から継続し、detached checkout を ready と扱いません。

## recovery と歴史境界

欠落、stale、foreign、矛盾した事実は fail closed で retry identity を保持します。provider anomaly
や stacked-PR recovery は別の明示的 receipt を使い、immutable archive を書き換えず、open PR を
merged に変えません。reference source の `make` と Python orchestration は Rust Runtime command
ではありません。Rust は明示的な `--repo` と repository-local evidence で同じ review、archive、
正確な cleanup の意図を保ちます。
