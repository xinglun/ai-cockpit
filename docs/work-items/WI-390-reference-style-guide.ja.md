---
workItemId: WI-390-reference-style-guide
title: "Reference Work Item スタイルガイド"
author: AI Cockpit maintainers
description: "固定した Work Item style guidance の semantic comparison record。"
audience:
  - maintainer
  - reviewer
authority: canonical
status: implemented
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
lastVerifiedBy: WI-390-reference-style-guide
terminalArchive: .ai/work-items/archive/WI-390-reference-style-guide.contract.json
terminalVerification: .ai/evidence/WI-390-reference-style-guide.verification.json
terminalFinalization: .ai/decisions/WI-390-reference-style-guide.finalize.b0a9c123b5f157c327a4068001f478d05b6d39e152363bc167945e0dc83fe423.json
terminalDecision: .ai/decisions/WI-390-reference-style-guide.close.json
---

# WI-390 — Reference Work Item style guide

[English](WI-390-reference-style-guide.md) · [简体中文](WI-390-reference-style-guide.zh-CN.md)

## Intent

固定した `docs/work-item-style-guide.md` を section ごとに比較し、読者向けの governance semantics だけを
Rust-native documentation に引き継ぎます。installer と Runtime implementation はコピーしません。

## Scope

- 固定 source: `docs/work-item-style-guide.md`
- Rust counterpart: `docs/reference/work-item-style-guide.*`
- この比較に必要な index、parity、inventory の同期

## Acceptance

- 結果を先に書くこと、問題/境界/non-goal、検証可能な acceptance、人が所有する decision、実行可能な
  verification、proportional な process、documentation-before-schema を表現します。
- shared Runtime と明示的な `--repo` repository isolation を説明し、installer command や source Runtime code は再現しません。
- 三言語 link と比較 record が同期します。

## Verification boundary

これは semantic/documentation parity であり、source command、JSON wire、provider state compatibility ではありません。
object/adopter repository は自分の `.ai/` と adapter から reader-facing rule を継承し、Contract、evidence、knowledge、
repository identity は repository ごとに分離されます。

## Evidence

Runtime の terminal evidence は次に記録されます。

- `.ai/evidence/WI-390-reference-style-guide.verification.json`
- `.ai/work-items/archive/WI-390-reference-style-guide.*`
- `.ai/decisions/WI-390-reference-style-guide.close.json`
