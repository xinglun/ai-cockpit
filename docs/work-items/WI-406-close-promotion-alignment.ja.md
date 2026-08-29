---
author: AI Cockpit maintainers
title: WI-406 — Close 済みドキュメント昇格の整合
description: Runtime の finalReport evidence binding と closed Work Item ドキュメント昇格を整合させます。
workItemId: WI-406-close-promotion-alignment
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-406-close-promotion-alignment
---

# WI-406 — Close 済みドキュメント昇格の整合

## Intent

Runtime が有効とする `finalReport` evidence binding を closed Work Item の
ドキュメント昇格器で受け入れ、形式不正または不完全な close 記録は
fail-closed のままにします。

## 範囲

- `finalReport.bindings` で結び付いた verification 参照を受け入れる。
- 構造化された human decision の参照を非空かつ監査可能に保つ。
- この Work Item の三言語ドキュメントと parity 登録を保持する。

## Evidence

- Archive Contract: `.ai/work-items/archive/WI-406-close-promotion-alignment.contract.json`
- Verification: `.ai/evidence/WI-406-close-promotion-alignment.verification.json`
- Pull Request: [#371](https://github.com/xinglun/ai-cockpit/pull/371)

## 境界

この Work Item は過去の Runtime evidence を書き換えず、Runtime lifecycle の
意味も変更しません。reviewed merge と close evidence が揃った後にだけ、
終端ドキュメントを昇格します。
