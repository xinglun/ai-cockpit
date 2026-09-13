---
author: AI Cockpit maintainers
title: "WI-833 — リリーススクリプトの出所"
description: "リリースのオーケストレーションと不変なソース識別子を分離します。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-833-release-script-provenance
lastVerifiedBy: WI-833-release-script-provenance
terminalArchive: .ai/work-items/archive/WI-833-release-script-provenance.contract.json
terminalVerification: .ai/evidence/WI-833-release-script-provenance.verification.json
terminalFinalization: .ai/decisions/WI-833-release-script-provenance.finalize.json
terminalDecision: .ai/decisions/WI-833-release-script-provenance.close.json
---

[English](WI-833-release-script-provenance.md) · [简体中文](WI-833-release-script-provenance.zh-CN.md)

# WI-833：リリーススクリプトの出所

## 意図

リリース受け入れは、レビュー済みの workflow commit にあるオーケストレーション
コードを実行しなければなりません。不変な対象 Release tag は成果物とソースの
識別子だけに使い、受け入れスクリプトのバージョン選択には使いません。

## 決定

staged/public adopter の 4 つの job は workspace ルートを `github.sha` に checkout
し、要求された `to_tag` を `release-source` に別 checkout します。受け入れスクリプト
はルートから実行し、`--source-repo` には `release-source` を渡します。これにより
WI-832 のコマンド再利用修正を受け入れ処理でも利用しつつ、不変な tag と成果物を
変更しません。

## 検証

adopter 受け入れ回帰テストは両方の checkout 識別子を検証し、対象 tag をスクリプト
checkout に使う、または別のソース checkout を用意しない job を拒否します。正式な
Runtime verification は回帰結果と終了コードを保存します。

## 対象外

Runtime の再利用セマンティクス、製品ビルド、不変な tag と Release、過去の Work Item、
無関係な branch や worktree の清掃は対象外です。
