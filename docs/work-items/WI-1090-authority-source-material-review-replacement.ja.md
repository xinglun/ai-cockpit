---
author: AI Cockpit maintainers
title: "WI-1090 — 分割された現在の材料レビュー置換"
description: "分割された現在の材料レビュー置換"
audience:
  - maintainer
  - reviewer
authority: explicit-user-authorization
workItemId: WI-1090-authority-source-material-review-replacement
status: in_progress
lastVerifiedBy: WI-1090-authority-source-material-review-replacement
contractDigest: sha256:c9a3bda349078f1fc15891e9e789d100952cbadaea12eb464f2024c90617ce9c
---

# WI-1090-authority-source-material-review-replacement — 分割された現在の材料レビュー置換

現在のContract識別子に対するhuman:Rayの実際の確認は2026-10-19T19:18:03Zにagent:Codexがネイティブ記録し、その後checkpoint1を作成しました。実装とテストは進行中で、製品の受入れ成功はまだ主張しません。

Hは不変の旧受領書とレビュー済みR→A履歴を検証し、四種類の履歴変更のみを許可しprofileを維持します。Pは最後の隣接A→BがauthoritySourceだけを変更することを証明します。Cは現在の正確な独立判断を結び付けます。旧受領書はinvalidのままUnknownを解消しません。

投影とロック内recordは同じ限定証明を使います。不変sealed prepareとsidecar、単一のSummary原子的コミット、正確なpending再開を検証します。evidenceRefsの形式は人間を認証せず、self_declared保証は変更しません。

範囲はRust二ファイルと六つの統治文書のみです。現在候補の七つの正式チェックは未実行です。Sentinel、実際の消費者修約、実WI終端、merge、release、Runtime採用は行いません。

Active [Contract](../../.ai/work-items/active/WI-1090-authority-source-material-review-replacement.contract.json): `sha256:c9a3bda349078f1fc15891e9e789d100952cbadaea12eb464f2024c90617ce9c`. Runtime evidence remains authoritative.

Spec: `../../.ai/evidence/WI-1090-authority-source-material-review-replacement/segmented-v4-review/segmented-authority-source-replacement.v4.proposal-only.json`.
