---
author: AI Cockpit maintainers
title: "WI-234 — merge 後ガバナンス cleanup と stale-close 防止"
workItemId: WI-234-post-merge-governance-cleanup
description: "merge 後のガバナンス loop を閉じ、stale merged receipt を防止して、次のバッチを clean environment から開始します。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-234-post-merge-governance-cleanup
---

# WI-234 — merge 後ガバナンス cleanup と stale-close 防止

## Intent

次の逐次ファイル比較バッチを開始する前に、merge 後のガバナンス loop を閉じます。
immutable な失敗・recovery history は保持し、不要な branch、worktree、temporary
checkout を除去して、次のバッチを clean environment から開始します。

## この Work Item が必要な理由

最近の hosted failure から、次の二つの反復する process gap が判明しました。

- reviewed head が同期済み default branch に存在しているのに、pre-merge
  finalization receipt が `unmerged` のまま残る。
- merge 後に生成された evidence が元の release Contract scope 外になり、実際の
  merge を close できず recovery Work Item が必要になる。

この Work Item ではこれらを workflow control として固定します。既存の失敗 PR、
Contract、evidence は immutable のまま、cleanup は履歴を書き換えず disposition を記録します。

## Scope

- deterministic な `stale_awaiting_merge_close` governance-gate finding と regression
  fixture を追加します。
- English、Chinese、Japanese の parity ledger に WI-222、WI-227、WI-230 と本 Work
  Item の最終 disposition を反映します。
- WI-230 の append-only historical transition を保持し、現在の Work Item から recovery
  binding を作ります。WI-222 は linked immutable history のままとし、二つ目の successor
  edge は作りません。
- WI-189/WI-193/WI-222/WI-223/WI-224/WI-225/WI-228 の dirty worktree bytes と branch
  tip を repository 外の digest archive に保存し、PR の disposition を記録した後で対象の
  obsolete checkout と ref だけを削除します。

## Out of scope

predecessor の Contract、Summary、Outcome、Events、verification、archive、hosted failure
bytes は書き換えません。global Agent/MCP configuration と root worktree の既存ユーザー
ファイルも変更しません。

## Acceptance

1. reviewed head が同期済み default branch に存在し、current-release の pre-merge receipt
   が `unmerged` のままなら、gate は stable な `stale_awaiting_merge_close` finding で拒否します。
2. 三言語 parity ledger が一致し、正確な evidence/decision path を参照します。
3. historical branch/worktree は digest-bound external archive に保存するか、PR を close/
   supersede した後に正確に削除します。
4. root worktree の既存ユーザーファイルは変更しません。
5. finalize 前に installed Runtime の inspect/status/doctor と宣言済み governance/documentation
   checks を通過します。

## Recovery と cleanup のルール

immutable predecessor は Runtime がサポートする一つの実在する successor edge だけで bind
します。二つ目の edge が必要なら、文書に history link を残し、新しい独立 Work Item を作成します。
receipt は捏造しません。cleanup は fail-closed とし、worktree/branch を削除する前に archive
status、untracked bytes、branch tip、PR state、SHA-256 manifest を保存します。

## References

- [English parity ledger](../reference/reference-parity.md)
- [Chinese parity ledger](../reference/reference-parity.zh-CN.md)
- [Japanese parity ledger](../reference/reference-parity.ja.md)
- [Repository governance gate](../../tests/ci/governance_integrity_gate.py)
- [Gate regression](../../tests/ci/governance_integrity_gate_test.sh)
