---
author: AI Cockpit maintainers
title: "WI-1076 — governance recovered predecessor projection"
description: "厳密に検証された superseded predecessor について、governance integrity と status consistency の投影を一致させます。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1076-governance-recovered-predecessor-projection
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1076-governance-recovered-predecessor-projection
contractDigest: sha256:dadae347c9c827447d1869b674f4572507a677516321ad3d389aeab11e67e442
---

[English](WI-1076-governance-recovered-predecessor-projection.md) · [简体中文](WI-1076-governance-recovered-predecessor-projection.zh-CN.md)

# WI-1076 — governance recovered predecessor projection

このページは active な [WI-1076 Contract](../../.ai/work-items/active/WI-1076-governance-recovered-predecessor-projection.contract.json) の投影であり、`sha256:dadae347c9c827447d1869b674f4572507a677516321ad3d389aeab11e67e442` に結び付いています。准入、検証、ライフサイクル状態は Runtime の記録を正とします。

## 期待する効果

同じ有効な superseded-predecessor lineage に対して、両方の governance gate が同じ predecessor 状態を示すようにします。digest-bound の recovery、close、repository、successor の各 binding がすべて有効な場合に限り、predecessor を `Recovered` と表示できます。未完了の successor は `awaiting_successor_close` のまま表示し、終端状態として扱いません。

## 範囲と境界

実装ファイルは `tests/ci/governance_integrity_gate.py` と `tests/ci/governance_integrity_gate_test.sh` に限定します。三言語ページと parity 行は必須の Work Item governance projection です。追加された三ページは、レビュー済み WI-1070 文書コミット `b2402611267b5a8c65c15e834e462ea74d0b42ad` に基づき、WI-1068 の安定版リリース履歴を保持します。WI-1070 の governance 記録と release acceptance 状態はこの Work Item の範囲外です。status-consistency checker、Runtime、製品コード、release workflow、tag、publication、provider resource は範囲外です。既存の acceptance、archive、close 要件は引き続きハードゲートです。

## 受け入れ条件と検証

有効な superseded predecessor と、identity-bound で未完了の successor の組み合わせを通すこと、無効な recovery、close、repository、lineage binding を拒否すること、successor の未完了状態を保持することを確認します。検証済みで archive 済みかつ close 確認済みの successor だけが predecessor を Implemented にできます。

Contract に宣言された確認コマンドは `bash tests/ci/governance_integrity_gate_test.sh`、`bash tests/docs/work_item_status_consistency_test.sh`、`bash tests/docs/parity_status_check.sh` です。確認結果は Work Item に結び付いた Runtime の正式検証レシートに記録され、このページはその証拠を代替しません。
