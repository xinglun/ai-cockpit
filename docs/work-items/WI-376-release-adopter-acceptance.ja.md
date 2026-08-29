---
author: AI Cockpit maintainers
title: "WI-376 — v0.2.39 Release adopter 受入検証"
description: "不変な公開 Release を現在のリポジトリと新規独立 adopter で検証する。"
workItemId: WI-376-release-adopter-acceptance
audience: [maintainer, reviewer]
status: completed
authority: human-authorized
capabilityClaims: [release_acceptance, repository_isolation, evidence_reuse]
---

# WI-376 — v0.2.39 Release adopter 受入検証

[English](WI-376-release-adopter-acceptance.md) · [简体中文](WI-376-release-adopter-acceptance.zh-CN.md)

## 目的

不変な v0.2.39 公開 Release が、状態を共有せず source checkout に依存せずに、
現在のリポジトリと新規の独立 adopter を統治できることを証明する。

## 範囲と境界

- 公開 archive、binary digest、manifest、checksum を検証する。
- 現在のリポジトリで v0.2.39 Runtime の継承を検証する。
- 新規 adopter を attach し、証拠と構造化 close decision を含む Work Item
  lifecycle を完了する。
- 完全一致時の evidence reuse、snapshot 変更時の再実行、グローバル領域の隔離を
  証明し、監査可能な受入 receipt を保存してから一時状態を削除する。

Runtime の新機能、source/workspace binary fallback、第二技術スタック、グローバル
Agent/MCP 設定は本 Work Item の範囲外とする。

## 受入基準

1. ダウンロードした v0.2.39 archive と binary が `release-manifest.json`、
   `SHA256SUMS` と一致する。
2. 現在のリポジトリが `COMPATIBLE` かつ `ready_on_base` で、`doctor` が正常、
   `runtimeCodeInRepository` が false、Agent doctor が `VERIFIED` である。
3. 新規 adopter は異なる `repositoryId` を持ち、最小 scaffold のみを受け取る。
4. 新しい Work Item skeleton は `not_ready` のままで、Runtime は人間の intent、
   scope、acceptance、authority を補完しない。
5. adopter lifecycle が repository、snapshot、Work Item、Runtime identity、
   close decision に結び付いた schema-2 evidence を生成する。
6. 完全一致した再検証は実行なしで reuse し、snapshot 変更時は再実行する。
7. 書込み禁止のグローバル領域は不変で、Runtime の書込みは隔離領域内だけである。
8. 受入 artifact に Runtime identity、JSON 出力、reuse/隔離証明、lifecycle evidence、
   checksum が含まれ、終了後に一時 adopter と run root が削除される。

## 検証境界

検証対象は公開済み Release のみとする。受入 receipt は公開後の証拠であり、不変な
Release truth を変更しない。

## 結果

v0.2.39 の公開 archive と binary を release manifest および checksums と照合した。
現行リポジトリは Runtime 0.2.39 を継承し、`inspect`、`status`、`doctor`、Agent
doctor はすべて正常だった。新しい独立 adopter には別の repository identity が
割り当てられ、`first-adopter-smoke` は `not_ready` のまま保持され、schema-2 の
verification/finish/archive/finalize/close lifecycle が完了した。Release と adopter
の全 receipt は `release-adopter-acceptance-artifacts/` に保存し、固定 adopter パスと
隔離 retry root は収集後に削除した。
