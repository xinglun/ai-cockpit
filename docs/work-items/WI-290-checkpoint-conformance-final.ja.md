---
author: AI Cockpit maintainers
title: "WI-290 — Checkpoint conformance final delivery"
workItemId: WI-290-checkpoint-conformance-final
description: "最新の remote default base から predecessor bytes を書き換えず、bounded checkpoint conformance batch を再配信する。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-290-checkpoint-conformance-final
authority: canonical
---

# WI-290 — Checkpoint conformance final delivery

## 目的

WI-287、WI-288、WI-289 は hosted gate が delivery binding の問題を検出した
ため、不変の recovery history として保持する。この successor は同じ bounded
implementation を最新の remote default base から再配信し、verify 前に三言語の
lifecycle evidence を完全に登録する。

## 境界

- WI-287、WI-288、WI-289 の archive、evidence、recovery、finalization bytes を保持する。
- Rust-native checkpoint と Agent-rule implementation は変更しない。
- archive 前に三言語 documentation/parity lifecycle path を完全に登録する。
- verify 前に新しい Provider PR を bind し、hosted checks、finalization、close、
  exact resource cleanup を完了する。

## adopter との一致

この repository と fresh adopter は、同じ installed Runtime、explicit
repository context、fail-closed lifecycle、human-visible Outcome で統治する。

## 検証

`cargo test --locked --workspace`、conformance inventory、documentation/
governance integrity gates、hosted PR checks、Provider finalization verify、
close、close 後の status/doctor を実行する。
