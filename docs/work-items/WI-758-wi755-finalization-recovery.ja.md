---
author: AI Cockpit maintainers
title: "WI-758 — WI-755 finalization recovery"
description: "不変の履歴を書き換えずに WI-755 の merge 後 governance handoff を修復する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-758-wi755-finalization-recovery
status: implemented
authority: human:xinglun
lastVerifiedBy: WI-758-wi755-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-758-wi755-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-758-wi755-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-758-wi755-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-758-wi755-finalization-recovery.close.json
---

[English](WI-758-wi755-finalization-recovery.md) · [简体中文](WI-758-wi755-finalization-recovery.zh-CN.md)

# WI-758 — WI-755 finalization recovery

## Intent

この successor は WI-755 の merge 後の governance handoff を修復する。WI-755 の archive
Contract、verification evidence、recovery decision、finalization receipt は不変の履歴として保持する。

## 観測された境界

PR #735 は review 済みで hosted checks がすべて成功し、
`8dcac7ecdb6878c6e506d86921e4d82b7e16e68b` として merge された。review 済み PR head は
`06f7d03f6877405a6885cf1412108e68d9c2893a` である。一方、WI-755 の canonical finalization receipt
は以前の中間 head
`251c3867c75d1395b4aa203057174fe1c9b1cb52` を記録している。

Runtime は unbound な finalization head の変更を正しく拒否する。WI-758 はこの差異を記録し、
新しい successor boundary を提供する。古い receipt の意味を変更せず、predecessor archive を書き換えず、
governance 修復を実装承認や安全保証として表現しない。

## Boundary と evidence

- recovery decision: `.ai/decisions/WI-755-p1-observation-context-successor.recovery.ccd6ce5cf8f1c2a563578437cc313079462d08a88b04fb8b61b734f54bf237a2.json`
- predecessor Contract: `.ai/work-items/archive/WI-755-p1-observation-context-successor.contract.json`
- predecessor verification: `.ai/evidence/WI-755-p1-observation-context-successor.verification.json`
- predecessor finalization fact: `.ai/decisions/WI-755-p1-observation-context-successor.finalize.json`
- reviewed PR: `https://github.com/xinglun/ai-cockpit/pull/735`

この successor が追加できるのは、自身の三言語ドキュメント、parity projection、verification evidence、
Runtime が生成する lifecycle records だけである。source code、protocol schema、Outcome wording、
authorization semantics、および WI-755 のすべての履歴 bytes は対象外である。

## Verification と限界

Verification は repository identity、review 済み merge commit に含まれる WI-755 implementation、
predecessor の正確な digest binding、predecessor bytes が書き換えられていないことを確認する。
性能向上、外部ユーザーの認知、release approval は主張しない。

## Terminal evidence

terminal archive、verification、finalization、close の path は、Runtime の `finalize-verify` と構造化
close が成功した後、post-close documentation promotion によって追加される。
