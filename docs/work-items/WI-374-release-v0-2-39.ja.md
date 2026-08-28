---
author: AI Cockpit maintainers
title: "WI-374 — v0.2.39 release と exact verification reuse の受入れ"
description: "復旧 parity projection を修正し、dynamic verification-reuse Runtime を公開して隔離 repository で受け入れる。"
workItemId: WI-374-release-v0-2-39
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-374-release-v0-2-39
capabilityClaims: [release_distribution, verification_performance, adopter_acceptance]
---

# WI-374 — v0.2.39 release と exact verification reuse の受入れ

[English](WI-374-release-v0-2-39.md) · [简体中文](WI-374-release-v0-2-39.zh-CN.md)

## Intent

レビュー済みで同期された `main` から v0.2.39 を公開し、identity-bound な dynamic exact verification reuse を本 repository と adopter repository の両方で利用可能にする。公開前に WI-370 と WI-371 の復旧 receipt の parity projection を修正する。

## Scope と境界

- 三言語の Cargo metadata、lockfile、versioning、release、distribution 文書を v0.2.39 に揃える。
- 三つの parity ledger で digest-suffixed な authoritative recovery receipt を参照し、predecessor evidence は書き換えない。
- strict release workflow が生成した immutable、checksum、SBOM、provenance-bound artifact だけを公開する。
- 公開 artifact をダウンロードして本 repository と新しい隔離 adopter にだけインストールし、Runtime、repository isolation、exact reuse の evidence を残す。

Runtime semantics、historical evidence の書換え、global Agent/MCP 設定、source-build fallback、第二技術 stack adopter は本 Work Item の範囲外である。

## Acceptance

1. Cargo metadata と lockfile が v0.2.39 で一致する。
2. 復旧 parity 行が authoritative recovery receipt を参照し、strict documentation/governance gate が通る。
3. 公開 Release が target archive、manifest、SHA256SUMS、target-bound SBOM、Formula、provenance evidence を含む。
4. 公開 binary の version と digest が受入れ receipt に結び付き、source/workspace fallback を使わない。
5. 本 repository と新しい adopter で valid な exact evidence を再利用し、変更・stale・unknown 入力は再実行または fail-closed で停止する。
6. HOME/XDG isolation、許可された Runtime write root、cleanup、lifecycle evidence、失敗時に Release truth を変更しないことを証明する。
7. レビュー済み merge、finalization、close、default branch 同期、正確な branch/worktree cleanup 後に `ready_on_base` となる。

## Verification boundary

公開前は strict repository gate manifest と staged artifact acceptance を使う。公開後は immutable な v0.2.39 artifact だけをダウンロードし、tag、archive digest、binary digest、platform、source を記録する。最適化は exact-match reuse のみであり、初回または無効化された検証は必ず実行する。測定した効果をそれらの経路へ外挿しない。
