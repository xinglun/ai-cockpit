---
author: AI Cockpit maintainers
title: "WI-541 — v0.2.67 release と公開 artifact acceptance"
description: "レビュー済み Runtime を公開し、download した公開 artifact の境界を検証する。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
workItemId: WI-541-release-v0-2-67
lastVerifiedBy: WI-541-release-v0-2-67
terminalArchive: .ai/work-items/archive/WI-541-release-v0-2-67.contract.json
terminalVerification: .ai/evidence/WI-541-release-v0-2-67.verification.json
terminalFinalization: .ai/decisions/WI-541-release-v0-2-67.finalize.json
terminalDecision: .ai/decisions/WI-541-release-v0-2-67.close.json
---

[English](WI-541-release-v0-2-67.md) · [简体中文](WI-541-release-v0-2-67.zh-CN.md)

## Goal

レビュー済みで同期済みの default branch から v0.2.67 を公開し、source や workspace binary に fallback せず、immutable な公開 artifact を install して新しい adopter acceptance を完了できることを検証する。

## Scope と boundary

- Workspace version と lockfile、現行 release/versioning architecture、distribution 説明、三言語 parity 登録。
- Hosted release checks と、公開後の artifact、checksum、SBOM、provenance、isolation、cleanup、installed Runtime evidence。
- 対象 repository `/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator` は read-only。対象 `.ai/`、PR identity、global Agent/MCP 設定は変更しない。

## Acceptance

- Cargo metadata と lockfile が v0.2.67 を示し、過去の release facts は変更しない。
- annotated v0.2.67 tag の作成前にレビュー済み PR と hosted checks が成功し、tag は同期済み `main` から作成する。
- 公開 archive、checksum、SBOM、provenance、release manifest が同じ tag commit と bytes に bind される。
- download した公開 artifact の adopter/N-1 acceptance が隔離 root で成功し、`first-adopter-smoke=not_ready` と temporary-root cleanup を証明する。
- installed public v0.2.67 が repository-bound health checks を通過し、human Outcome、archive、finalization、close、正確な branch cleanup を記録する。

## Verification

```text
cargo metadata --locked --format-version 1
cargo test --locked --workspace
tests/release/version_consistency.sh --repo <repo>
tests/release/action_runtime_policy.sh .github/workflows/ci.yml .github/workflows/release.yml
tests/release/adopter_acceptance_test.sh
tests/release/adopter_upgrade_acceptance_test.sh
tests/docs/documentation_acceptance.sh
tests/docs/parity_status_check.sh
tests/docs/promote_closed_work_item.py --repo <repo> --check-all
```

Publication と post-release acceptance は別の事実である。post-release acceptance が失敗した場合は `releasePublished: true` と `adopterAcceptance: failed` を記録し、公開済み Release の truth を書き換えない。
