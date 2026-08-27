---
author: AI Cockpit maintainers
title: "WI-314 — finalization reconciliation redelivery"
workItemId: WI-314-finalization-reconciliation-redelivery
description: "immutable な hosted quality failure 後に cleanup-before-close と append-only finalization reconciliation を再配信する。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-314-finalization-reconciliation-redelivery
---

# WI-314 — finalization reconciliation redelivery

## Intent と boundary

WI-312 は immutable な履歴 delivery として保持します。retained provider
finalization と conditional parity projection が merge 後に順序の欠陥を示しました。
最初の Runtime correction は WI-313 で実装しましたが、PR #277 は merge 前の hosted
documentation gate によって正しく拒否されました。本 successor は同期済み default
branch から bounded correction を再配信し、どちらの predecessor も書き換えず W312 の
明示的 recovery を記録します。

## Scope と acceptance

- 新しい Work Item は provider finalization が retained、blocked、unknown の場合 close
  できず、identity-bound な deleted result だけが close を満たします。
- legacy の closed record に対する append-only deleted transition は、immutable
  predecessor、repository、Runtime、sequence、正確な cleanup postcondition が一致する場合に
  一度だけ許可します。
- W312 は `Recovered` として表示し、元の Contract、evidence、archive、finalization、close
  bytes は不変にします。有効な recovery/reconciliation binding のない conditional terminal
  parity row は引き続き失敗します。
- English、Simplified Chinese、Japanese の parity/work-item projection を verification
  前に同期し、正確な evidence link を保持します。

## Verification

finalization/documentation の targeted regression、`cargo fmt`、warning deny の clippy、
locked full workspace test を実行します。merge 前に、review 済みの正確な branch が hosted
CI を通過する必要があります。Governance interface は installed Runtime を使用し、source
build は release acceptance の代替にしません。

## Related history

- W312: 本 successor が recovery する immutable な merged delivery。
- W313 / PR #277: immutable な hosted failure delivery。branch と archive は外部監査履歴として保持し、復活させません。

[English](WI-314-finalization-reconciliation-redelivery.md) ·
[简体中文](WI-314-finalization-reconciliation-redelivery.zh-CN.md)
