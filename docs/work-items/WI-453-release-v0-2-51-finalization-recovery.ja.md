---
author: AI Cockpit maintainers
title: "WI-453 — v0.2.51 リリース finalization recovery"
workItemId: WI-453-release-v0-2-51-finalization-recovery
description: "provider context が provisional のまま archive された v0.2.51 リリース Work Item を復旧する。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-453-release-v0-2-51-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-453-release-v0-2-51-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-453-release-v0-2-51-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-453-release-v0-2-51-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-453-release-v0-2-51-finalization-recovery.close.json
---

# WI-453 — v0.2.51 リリース finalization recovery

この recovery Work Item は WI-452 の immutable な archive bytes を保持し、v0.2.51 公開前に実際の reviewed provider context を bind します。WI-452 は PR #422 の作成前に archive されたため必要な経路であり、predecessor receipt を書き換えたり偽造したりしません。

[English](WI-453-release-v0-2-51-finalization-recovery.md) · [简体中文](WI-453-release-v0-2-51-finalization-recovery.zh-CN.md)

## Scope

- WI-452 の recovery decision と predecessor digest を保持・bind する。
- 独立した reviewed PR を使い、verification と archive の前に完全な context を bind する。
- immutable な v0.2.51 tag を作成する前に recovery lineage を close する。
- 公開後はダウンロードした release artifact だけで adopter acceptance を実行する。

## Boundary

対象リポジトリは変更しません。WI-452 の archived Contract、Summary、Outcome、Events、verification evidence は byte-for-byte immutable のままです。source checkout、workspace binary、偽造 PR、手編集の生成 receipt は release evidence として認めません。

## Verification

- `cargo test --locked --workspace`
- release 文書、workflow、source archive、version consistency gate
- reviewed recovery PR に bind した Runtime verification と provider finalization
- source fallback を使わないダウンロード済み v0.2.51 artifact adopter acceptance
