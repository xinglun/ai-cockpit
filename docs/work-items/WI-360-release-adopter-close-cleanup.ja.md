---
author: AI Cockpit maintainers
title: "WI-360 — Release adopter の close cleanup"
workItemId: WI-360-release-adopter-close-cleanup
description: "staged/N-1 adopter acceptance の resource finalization と一時 run root cleanup を修正する。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-360-release-adopter-close-cleanup
authority: canonical
---

# WI-360: Release adopter の close cleanup

## 目的

staged と N-1 の Release adopter acceptance harness が、`close` 前に Runtime
lifecycle の resource finalization を完了し、feature branch や worktree を
retained のまま残さないようにします。

## 範囲

- `tests/release/adopter_acceptance.sh`
- `tests/release/adopter_upgrade_acceptance.sh`
- 両 harness の static regression wrapper
- 三言語の release distribution 文書

これは post-release acceptance の修正です。Runtime の resource-finalization
規則を緩めず、immutable な `v0.2.36` staged failure の事実も変更しません。

## 設計

各 fixture は存続する control checkout と専用の lifecycle checkout を
使います。archive 後、harness は生成された archive 記録を commit し、control
checkout へ fast-forward し、対象 lifecycle checkout と branch を削除して
`disposition: deleted` の finalization receipt を記録します。その後、存続する
control checkout から `finalize`、`finalize-verify`、`close` を実行します。

EXIT trap は、検証済みの一時 `run_root` を削除する前に acceptance receipt と
checksum を書き出します。success、failure、interrupt の全経路で cleanup 状態を
記録し、cleanup failure では receipt を保持したまま non-zero を返します。

## 受入れ

- staged adopter lifecycle が `disposition: deleted` で `close` まで成功する；
- N-1 の old/new lifecycle が同じ cleanup を実行する；
- 実行していない retained resource 状態を示す receipt がない；
- static test が retained close receipt を拒否し、branch/worktree 削除を要求する；
- 三言語文書が control-worktree 遷移と immutable な `v0.2.36` staged failure を説明する；
- source checkout と write forbidden の HOME/XDG root が変化しない；
- success/failure の両経路で対象の一時 run root が削除される。

## 検証 evidence

まず static wrapper で release harness を検証し、公開 Release artifact による
staged/N-1 acceptance job で実動作を検証します。receipt は Runtime identity、
repository identity、lifecycle 出力、isolation manifest、cleanup 状態、checksum
を記録します。post-release acceptance の失敗は Release の公開事実を書き換えません。
