---
author: AI Cockpit maintainers
title: "WI-397 — v0.2.40 release と公開性能継承"
description: "WI-396 clean-snapshot 最適化を release し、本 repository と新規 adopter で download 済み binary を検証する。"
workItemId: WI-397-release-v0-2-40
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-397-release-v0-2-40
terminalArchive: .ai/work-items/archive/WI-397-release-v0-2-40.contract.json
terminalVerification: .ai/evidence/WI-397-release-v0-2-40.verification.json
terminalFinalization: .ai/decisions/WI-397-release-v0-2-40.finalize.json
terminalDecision: .ai/decisions/WI-397-release-v0-2-40.close.json
capabilityClaims: [release_distribution, verification_performance, adopter_acceptance]
---

# WI-397 — v0.2.40 release と公開性能継承

[English](WI-397-release-v0-2-40.md) · [简体中文](WI-397-release-v0-2-40.zh-CN.md)

## Intent

レビュー済みの `main` から v0.2.40 を公開し、WI-396 の Rust clean-snapshot fast path を共有外部 Runtime から利用可能にする。Release と adopter acceptance は immutable な公開 artifact を使い、repository isolation を保つ。source build は install evidence ではない。

## Boundary

この Work Item は patch version、release workflow、三言語の distribution 文書をそろえ、公開 Release adopter と N-1 acceptance を実行する。governance semantics、historical evidence、global Agent/MCP 設定、performance budget は変更しない。各 adopter は共有 binary を明示的な `--repo` で bind し、独立した `.ai/` state を保持する。

## Acceptance

1. Cargo metadata と lockfile は v0.2.39 から一つだけ patch を進めて v0.2.40 になり、既存 tag/Release を再利用しない。
2. レビュー済み workflow は正確な merge commit から全 target を build し、manifest、Formula、SHA256SUMS、target SBOM、provenance、immutable tag/Release identity を bind する。
3. 公開 Release から download した v0.2.40 binary を checksum 検証し、version、binary digest、platform、Runtime identity を記録する。source/workspace fallback は使わない。
4. 公開 adopter acceptance は attach/profile/Agent doctor、`first-adopter-smoke` の `not_ready`、lifecycle と evidence reuse、isolation、cleanup、repository/runtime identity を証明する。
5. 該当する場合は N-1 acceptance を実行し、historical bytes を保持する。公開後チェック失敗時も `releasePublished: true` を記録する。
6. 本 repository と新規 adopter は共有 Runtime を介して WI-396 の測定済み clean-snapshot 最適化を継承し、global repository や cross-repository cache を導入しない。
7. review merge、finalization、close、同期、正確な cleanup 後に `main` は `ready_on_base` となり、未解決 PR と `codex/*` branch がない。

## Verification boundary

公開前は strict source/staged release gate を使う。公開後は immutable な公開 artifact だけを download し、Runtime、adopter、isolation、cleanup、checksum evidence を保存する。Release は local measurement から provider/enterprise 性能を主張しない。
