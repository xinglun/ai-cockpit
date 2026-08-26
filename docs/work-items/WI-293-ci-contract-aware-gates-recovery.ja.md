---
author: AI Cockpit maintainers
title: "WI-293 — CI Contract-aware quality gate recovery"
workItemId: WI-293-ci-contract-aware-gates-recovery
description: "最新の remote default base から parity を verification 前に登録して bounded CI Contract-aware gate を再配信する。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-294-lifecycle-recovery-state-machine
authority: canonical
---

# WI-293 — CI Contract-aware quality gate recovery

## 目的

WI-293 は immutable な recovered history として保持します。マージされた CI gate
は PR #253 に記録され、マージ後に見つかった lifecycle recovery の欠陥は bounded
successor WI-294 が担当します。どちらも predecessor bytes を書き換えません。

## Boundary

- WI-293 の archive、evidence、blocked finalization、recovery bytes を保持する。
- Rust を Contract gate authority としつつ Python/Cargo shadow checks を維持し、この batch では既存 CI policy を削除しない。
- 最終 verification 前に実際の provider PR を bind し、hosted checks、finalization、close、
  exact branch/worktree cleanup を完了する。

## adopter との一致

同じ installed Runtime、explicit `--repo` context、fail-closed evidence、human-visible
Outcome が merge delivery を治理しました。WI-294 は closure 中に見つかった recovery
boundary を記録します。

## 検証

`cargo test --locked --workspace`、CI/conformance と documentation gate、hosted PR checks、
provider finalization verification、close、close 後の status/doctor を実行します。
