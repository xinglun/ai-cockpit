---
author: AI Cockpit maintainers
title: "WI-614 — 初回 direct merge recovery の round-trip"
description: "PR のない歴史的 merge の recovery plan を完全かつ fail-closed な初回 receipt として適用する。"
audience:
  - adopter
  - maintainer
  - reviewer
workItemId: WI-614-direct-merge-no-pr-apply
status: implemented
authority: canonical
lastVerifiedBy: WI-614-direct-merge-no-pr-apply
terminalArchive: .ai/work-items/archive/WI-614-direct-merge-no-pr-apply.contract.json
terminalVerification: .ai/evidence/WI-614-direct-merge-no-pr-apply.verification.json
terminalFinalization: .ai/decisions/WI-614-direct-merge-no-pr-apply.finalize.json
terminalDecision: .ai/decisions/WI-614-direct-merge-no-pr-apply.close.json
---

# WI-614 — 初回 direct merge recovery の round-trip

## Intent

`work-item finalize-recovery-plan --merge-commit <sha>` が、完全な型付き
`direct_merge_no_pr` 初回 receipt を出力するようにします。人が入力するのは
authorization fields だけで、Git parents、repository identity、Contract binding、
Runtime identity、historical low-assurance の cleanup unknown は決定的に監査可能です。

## Boundary

plan は PR を捏造せず、historical assurance を引き上げず、`.ai/` の履歴を
書き換えません。merge facts の欠落や矛盾は引き続き fail-closed です。object
repository は外部 read-only acceptance boundary のままです。

## Verification

Runtime lifecycle evidence は plan→apply round-trip、negative cases、workspace tests、
clippy、documentation、governance integrity、reference inventory checks を対象にします。
terminal evidence は Runtime が生成する archive と receipt に記録します。

[English](WI-614-direct-merge-no-pr-apply.md) · [简体中文](WI-614-direct-merge-no-pr-apply.zh-CN.md)
