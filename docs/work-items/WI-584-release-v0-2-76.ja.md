---
author: AI Cockpit maintainers
title: "WI-584 — v0.2.76 release と object-adopter recovery handoff"
description: "archived Work Item の再検証に必要な Runtime release を公開・検証する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-584-release-v0-2-76
lastVerifiedBy: WI-584-release-v0-2-76
---

[English](WI-584-release-v0-2-76.md) · [简体中文](WI-584-release-v0-2-76.zh-CN.md)

# WI-584 — v0.2.76 release と object-adopter recovery handoff

## 目的

レビュー済みで同期された default branch から identity-bound な v0.2.76 Runtime
baseline を公開する。この release は object repository の append-only archived
Contract revalidation と successor close acceptance の Runtime 依存であるが、
本 Work Item は object repository を操作しない。

## 範囲と境界

- Cargo metadata、lockfile、三言語の release/versioning guidance を v0.2.76 に揃え、
  v0.2.75 を直前の公開 baseline として保持する。
- identity-bound な release archive、manifest、checksum、SBOM、provenance、
  attestation、Runtime identity を生成・検証する。
- 隔離 root 上で download 済み immutable artifact だけを使い、public adopter と
  N-1 acceptance を実行する。forbidden root と一時 run root の cleanup も証明する。
- object repository の recovery は外部 read-only handoff であることを記録し、そこで
  `.ai/`、source、branch、evidence を変更しない。

Runtime behavior、object repository、global Agent/MCP configuration、reference source
copy、failed tag history、無関係な historical record は対象外とする。

## 受入れ

1. Cargo metadata、lockfile、release/versioning page が v0.2.76 を示し、v0.2.75 を
   直前の公開 baseline として保持する。
2. Release CI が v0.2.76 の identity-bound 五 target artifact と supply-chain receipt
   を生成する。
3. public adopter と N-1 acceptance が download 済み v0.2.76 artifact だけを使い、
   isolation と一時 directory cleanup を証明し、同じ binary で本 repository を検証する。
4. Runtime behavior、object repository、global configuration、failed tag history、
   無関係な evidence を変更しない。
5. object repository team 向けに公開 Runtime identity と正確な command handoff を記録し、
   historical evidence を書き換えたり捏造したりしない。

## 検証境界

Contract の acceptance は作成言語を authoritative とし、localized page は表示だけを変える。
object repository の recovery は外部 read-only handoff であり、本 Work Item はその完了を主張しない。

## 検証

- `tests/release/version_consistency.sh`
- `tests/release/workflow_policy.sh`
- `tests/release/action_runtime_policy.sh`
- `tests/release/source_archive_policy_test.sh`
- `tests/release/adopter_acceptance_test.sh`
- `tests/release/adopter_upgrade_acceptance_test.sh`
- `tests/docs/documentation_acceptance.sh`
- `tests/docs/parity_status_check.sh`
- `cargo test --locked --workspace`
- `git diff --check`
