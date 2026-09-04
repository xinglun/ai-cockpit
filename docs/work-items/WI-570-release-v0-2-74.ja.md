---
author: AI Cockpit maintainers
title: "WI-570 — v0.2.74 release と公開 artifact の受入れ"
description: "次の immutable AI Cockpit Runtime release を公開して検証する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-570-release-v0-2-74
lastVerifiedBy: WI-570-release-v0-2-74
---

[English](WI-570-release-v0-2-74.md) · [简体中文](WI-570-release-v0-2-74.zh-CN.md)

# WI-570 — v0.2.74 release と公開 artifact の受入れ

## 目的

レビュー済みで同期された default branch から v0.2.74 を公開し、immutable Runtime baseline
を作る。その後、source checkout や workspace fallback を使わず、公開 download binary が
この repository を governance できることを証明する。

## 範囲と境界

- 三言語の Cargo metadata、lockfile、release/versioning guidance を v0.2.74 に揃える。
- default branch に既にある reference comparison と documentation promotion の close 済み
  記録に release を bind する。
- identity-bound な五 target archive、manifest、checksum、SBOM、provenance、attestation、
  Runtime identity を生成・検証する。
- immutable な download artifact のみで public adopter と N-1 acceptance を隔離 root 上で
  実行し、forbidden root と一時 run root の cleanup を証明する。

Runtime 実装、対象 repository、global Agent/MCP 設定、reference source 実装のコピー、失敗 tag の
書き換え、無関係な過去記録は対象外とする。

## 受入れ

1. Cargo metadata、lockfile、三言語の release/versioning page が v0.2.74 を示し、v0.2.73 を
   直前の公開 baseline として保持する。
2. Release CI が v0.2.74 の identity-bound 五 target artifact と supply-chain receipt を生成する。
3. public adopter と N-1 acceptance が download 済み v0.2.74 artifact だけを使い、隔離・cleanup
   と同一 binary による repository 検証を証明する。
4. release は同期済み ready default branch から始まり、Runtime、対象 repository、global 設定、
   無関係な過去 evidence を変更しない。

## 検証境界

Contract の acceptance は作成言語を authoritative とし、localized page は表示だけを変える。
公開 Release は immutable asset と adopter receipt を検証して初めて受入れ済みとなる。対象
repository の acceptance は外部の read-only handoff であり、このページでは主張しない。

## 検証

- `tests/release/version_consistency.sh`
- `tests/release/workflow_policy.sh`
- `tests/release/action_runtime_policy.sh`
- `tests/release/source_archive_policy_test.sh`
- `tests/release/adopter_acceptance_test.sh`
- `tests/release/adopter_upgrade_acceptance_test.sh`
- `tests/docs/documentation_acceptance.sh`
- `tests/docs/parity_status_check.sh`
- `git diff --check`
