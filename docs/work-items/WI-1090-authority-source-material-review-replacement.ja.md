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
contractDigest: sha256:1cc7ec218c2033ed48feb6b1ac298b392485dc898dcefac58f0fee14d06895c4
---

# WI-1090-authority-source-material-review-replacement — 分割された現在の材料レビュー置換

人間 Ray がレビュー済みのネイティブ計画を確認し、checkpoint1 の後に実装を完了した。前項の宣言された七つのネイティブ検証は成功し、自身のネイティブ finish と archive も完了した。[PR #1050](https://github.com/xinglun/ai-cockpit/pull/1050) は HEAD `cba3ea1661749ebbf1929bc523c5b86ff10eb7e7` の Draft のままであり、CI `39344530319` は Windows checkout と文書ゲートで失敗した。ネイティブ検証はホスト CI の受け入れを証明しない。close は保留中である。

H は不変の旧 receipt、レビュー済み R→A ネイティブ履歴、許可された四つの履歴パスと不変の profile を検証する。P は隣接 A→B が authoritySource だけを変更することを証明する。C は独立した正確な現在の決定を結び付ける。旧 receipt は引き続き無効であり、現在の Unknown を解消しない。

置換は共通 projection/ロック内証明、不変の sealed prepare と sidecar、および単一の原子的 Summary コミットと正確な pending recovery を使用する。evidenceRefs の構文は人間を認証せず、self_declared の保証水準は変わらない。ネイティブ相当の合成 fixture は実際の 20-operation 混合履歴の形を保持し、隣接するソース結合の前に現在の材料を独立して変更する。物理的な複数ファイルの原子性は証明していない。

選択された文書後継 `WI-1090-evidence-docs-recovery` は元の二つの Rust ファイルと履歴ネイティブ記録を変更しない。明示的な locator で八つの Agent コピーの保存パスを修復し、三言語の派生投影を訂正する。選択記録は `.ai/decisions/WI-1090-authority-source-material-review-replacement.recovery.json` である。両 Work Item の close、マージ、リリース、Runtime 採用、実際の consumer 受け入れ、Sentinel 復旧をここで証明しない。

アーカイブ済み [Contract](../../.ai/work-items/archive/WI-1090-authority-source-material-review-replacement.contract.json): `sha256:1cc7ec218c2033ed48feb6b1ac298b392485dc898dcefac58f0fee14d06895c4`。ネイティブ検証: `.ai/evidence/WI-1090-authority-source-material-review-replacement.verification.json`。Runtime の実際の証拠が正本となる。

M6 コピー locator: `.ai/evidence/WI-1090-authority-source-material-review-replacement/m6-short-20261020/path-map.actual.json`。元 manifest の論理キーとバイト列は不変である。元の仕様: `.ai/evidence/WI-1090-authority-source-material-review-replacement/segmented-v4-review/segmented-authority-source-replacement.v4.proposal-only.json`。
