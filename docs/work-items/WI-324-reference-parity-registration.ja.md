---
author: AI Cockpit maintainers
title: "WI-324 — reference parity registration recovery"
workItemId: WI-324-reference-parity-registration
description: "WI-323 の immutable archive 後に hosted docs governance が検出した登録漏れを修正します。"
audience:
  - maintainer
  - reviewer
status: in-progress
authority: canonical
lastVerifiedBy: WI-324-reference-parity-registration
---

# WI-324 — reference parity registration recovery

## Intent と goal

WI-323 の immutable archive 後、hosted `docs_governance_integrity` gate が検出した
tri-language `reference-parity` 登録漏れを修正します。WI-323 の immutable archive
と failed-delivery history は保持し、recovered successor を監査可能で独立した
review 対象にします。

## Scope と boundary

English、Simplified Chinese、日本語の parity ledger に WI-323（immutable predecessor）
と WI-324（bounded successor）を登録します。review 済みの WI-323 inventory、comparison、
Human Benefit pages、conformance generator/test、tri-language Work Item record を引き継ぎ、
WI-324 の tri-language record を追加してから同じ documentation/conformance checks を
新しい PR の前に実行します。

predecessor の archive/evidence/recovery bytes は書き換えません。Runtime feature、CI
policy、source Python/Make artifact の copy、global Agent/MCP configuration の変更も対象外です。

## Acceptance と verification

1. 3 つの parity ledger が WI-323 と WI-324 を一貫した link、status、recovery explanation
   で登録します。
2. clean `origin/main` baseline から引き継いだ inventory/documentation tests が pass します。
3. hosted `docs_governance_integrity` とその他の必須 PR checks がすべて pass します。
4. predecessor archive digest と recovery binding は変更されず、successor Contract/evidence
   は明示的な repository context に bind されます。

## Recovery evidence

predecessor archive と hosted failure は
`.ai/decisions/WI-323-reference-documentation-foundation.recovery.json` が参照します。
predecessor archive 後に漏れが発見されたためだけにこの successor を作成し、新しい
feature scope は追加しません。

[English](WI-324-reference-parity-registration.md) ·
[简体中文](WI-324-reference-parity-registration.zh-CN.md)
