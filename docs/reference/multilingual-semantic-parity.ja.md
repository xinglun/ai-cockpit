---
author: AI Cockpit maintainers
title: "多言語の意味的パリティ"
description: "権威ある Contract の原文を翻訳せず、言語投影でガバナンス事実を保つ。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# 多言語の意味的パリティ

[English](multilingual-semantic-parity.md) · [简体中文](multilingual-semantic-parity.zh-CN.md)

英語、簡体字中国語、日本語は、同じリポジトリ束縛 Runtime 事実の表示投影です。
固定見出し、状態ラベル、停止/次のアクション、リスク信号、制限、人手決定の
フィールドは、三言語で同じ意味を持たなければなりません。

CLI テストは三言語の安定した marker と summary を検証します。投影は次をしては
なりません。

- yellow/red の証拠を green にすること
- approval、benefit、capability、provider/enterprise claim を作ること
- blocker、unknown、必須チェック、安全警告、復旧手順を省くこと
- acceptance criteria、intent、scope など人が所有する Contract 値を翻訳・書換えすること

Contract 値は作成時の言語で保持し、原文であることを示します。Runtime が所有する
表示文だけをローカライズします。Agent adapter が非権威の翻訳を追加する場合でも、
原値と digest/参照がガバナンスのソースです。

これは意味の一致であり、source wire や Python comparator の互換性ではありません。
すべての投影は明示的な `--repo` とリポジトリ内の証拠を使い、言語設定で判断を変えません。
