---
author: AI Cockpit maintainers
title: "設計思想"
description: "AI Cockpit が repository の事実を人間が確認できる bounded decision に変換する理由。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - design_philosophy
keywords: [ai-cockpit, design-philosophy, evidence, human-control]
---

# 設計思想

## 目的

このページは、**なぜ AI Cockpit が自律 Agent や workflow engine ではなく governance
layer として設計されているのか**を説明します。

## 対象読者

自分の開発プロセスに適合するか判断するとき、またはなぜチェックが推測で進まず停止する
のかを知りたいときに読んでください。

## 読後の理解

runtime の背後にある原則と、AI Cockpit がローカルで証明できる事実、外部システムが提供
しなければならない evidence の境界を理解できます。

## North Star

AI Cockpit は校正された human-agent trust を支えます。意図した変更、許可された範囲、
repository の事実、verification の結果、そして人間が決める事項を見える状態にします。

```text
Evidence → Governance Decision → Human Control
```

## 原則

1. **自己申告より evidence。** command、Agent のメッセージ、ローカル flag だけでは証明に
   なりません。型付きの repository facts と記録された evidence から decision を導きます。
2. **境界を明示する。** Work Item は実装前に intent、scope、除外、authority、acceptance、
   required evidence を宣言します。
3. **一つの snapshot を観測する。** Git 状態、設定、関連ファイルを一度観測し、immutable
   input として再利用します。その後の変更は新しい fact であり、古い decision に黙って混ぜません。
4. **fail closed。** 欠落、stale、矛盾、改ざんされた evidence は `unknown` または `blocked`
   になります。都合のよい pass にはしません。
5. **リスクに比例した control。** 低リスクのローカル inspection は軽くし、protected gate
   には強い identity、evidence、human authority を要求します。
6. **human control を実効的に保つ。** AI Cockpit は次の安全な action を説明できますが、
   未検証の変更を承認したり、外部 actor を認証したり、review を代替したりしません。
7. **adapter は薄く保つ。** CLI と MCP は request/response を変換するだけで、governance
   rule は共有 application service と pure core にあります。

## 実際の動作

ユーザーが Agent に「ドキュメントを更新して」と依頼しても、AI Cockpit はそれを無制限の
workflow として扱いません。bounded Work Item を要求し、repository baseline を記録し、
宣言された check を実行し、人間が proceed、investigate、approve、block、recover を選べる
decision を提示します。

## どこに置くか

request、scope、repository state、verification、human decision は governed Work Item に
置きます。provider signature、SBOM、vulnerability scan、production approval などの専門的な
証明は、それを生成できる tool/service に任せます。evidence はリンクしますが、所有権を重複
して主張しません。

## 停止条件

effect に境界がない、evidence の所有権が曖昧、protected operation 中に snapshot が変化した、
またはローカル記録を外部 control の証明として使おうとした場合は停止します。欠けた link を
調査し、推測で埋めません。

## 次に読むもの

1. [Architecture](architecture.ja.md) — runtime path と evidence ownership。
2. [機能一覧](capabilities.ja.md) — reader-first の機能 overview と詳細。
3. [製品境界](architecture/product-boundary.ja.md) — 外部に残る責任。

## 技術的な深さ

実装は Repository Protocol、型付き Work Item lifecycle、immutable repository snapshot、
deterministic governance decision、bounded verification plan、content-addressed evidence、
共有 CLI/MCP service によってこれらの原則を表現します。これは review を支援する仕組みであり、
一般的な semantic-risk detector、identity provider、sandbox、compliance certificate ではありません。
