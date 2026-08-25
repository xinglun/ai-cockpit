---
author: AI Cockpit maintainers
title: "WI-289 — Checkpoint conformance terminal recovery"
workItemId: WI-289-checkpoint-conformance-terminal
description: "Hosted documentation-truth gate の拒否後も predecessor bytes を書き換えず、bounded checkpoint conformance batch を再配信する。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-289-checkpoint-conformance-terminal
authority: canonical
---

# WI-289 — Checkpoint conformance terminal recovery

## 目的

WI-288 は、archive 後も recovered WI-287 の documentation status が
`in_progress` のままだと hosted quality が検出したため、不変の recovery
history として保持する。この successor は同じ bounded implementation を
維持し、verify 前に三言語の documentation truth を修正する。

## 境界

- WI-287 と WI-288 の archive、evidence、recovery、finalization bytes を保持する。
- Rust-native checkpoint と Agent-rule implementation は変更しない。
- archive 前に三言語 documentation/parity projection を修正する。
- verify 前に新しい Provider PR を bind し、hosted checks、finalization、close、
  exact resource cleanup を完了する。

## adopter との一致

この repository と fresh adopter は、同じ installed Runtime、explicit
repository context、fail-closed lifecycle、human-visible Outcome で統治する。

## 検証

`cargo test --locked --workspace`、conformance inventory、documentation/
governance integrity gates、hosted PR checks、Provider finalization verify、
close、close 後の status/doctor を実行する。
