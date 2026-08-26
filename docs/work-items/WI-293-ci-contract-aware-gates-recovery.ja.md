---
author: AI Cockpit maintainers
title: "WI-293 — CI Contract-aware quality gate recovery"
workItemId: WI-293-ci-contract-aware-gates-recovery
description: "最新の remote default base から parity を verification 前に登録して bounded CI Contract-aware gate を再配信する。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-293-ci-contract-aware-gates-recovery
authority: canonical
---

# WI-293 — CI Contract-aware quality gate recovery

## 目的

WI-291 は hosted quality が late parity projection を拒否したため immutable
recovery history として保持します。この successor は最新の remote default
branch から同じ bounded Rust gate を再配信し、verification evidence 作成前に
三言語 parity と Work Item documentation を登録します。

## Boundary

- WI-291 の archive、evidence、blocked finalization、recovery bytes を保持する。
- Rust を Contract gate authority としつつ Python/Cargo shadow checks を維持し、この batch では既存 CI policy を削除しない。
- 最終 verification 前に実際の provider PR を bind し、hosted checks、finalization、close、
  exact branch/worktree cleanup を完了する。

## adopter との一致

この repository と fresh adopter は、同じ installed Runtime、explicit `--repo` context、
fail-closed evidence、human-visible Outcome で統治します。

## 検証

`cargo test --locked --workspace`、CI/conformance と documentation gate、hosted PR checks、
provider finalization verification、close、close 後の status/doctor を実行します。

