---
author: AI Cockpit maintainers
title: "操作時ポリシーの再評価"
description: "高リスク操作の直前に新鮮な事実を fail-closed で評価する。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# 操作時ポリシーの再評価

[English](operation-time-policy-reevaluation.md) · [简体中文](operation-time-policy-reevaluation.zh-CN.md)

スクリプト、計画、承認を作成したことは、後の実行を許可しません。実行器が高リスク
操作を行う直前に、adapter は厳格な `OperationTimeRequest` を Rust Core evaluator
へ渡せます。request は次を束縛します。

- requested operation と actual tool call
- target resource と正確な declared scope
- 以前の approval の operation、target、scope
- 現在の帰属可能な authority
- evidence の freshness、destructive impact の分類、input trust

評価器は `allow`、`confirm`、`block` の事実を返します。操作の実行、provider resource
への書込み、provider 権限の付与はしません。未知の操作、未分類の impact、空の scope、
不一致、stale evidence、非権威の入力を自動 allow にすることはありません。

削除、test/CI/branch protection 変更、secret 書込み、push、merge、release、migration、
script 実行、外部 API 書込み、install/upgrade、governance uninstall を high-risk 語彙として
扱います。provider と Agent は、このローカル評価後も自身の権限および protected branch
制御を適用します。

これは共有 Runtime の機能です。各 adopter は外側の command/adapter に明示的な repository
context を渡し、global current project や approval state を作りません。操作時評価はポリシー
入力であり、provider/enterprise approval の証拠ではありません。
