---
author: AI Cockpit maintainers
title: "WI-533 — v0.2.66 release と direct-merge recovery acceptance"
description: "bundled historical direct-merge 互換修正を含む Runtime を公開し、公開 artifact 境界を検証する。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
workItemId: WI-533-release-v0-2-66
lastVerifiedBy: WI-533-release-v0-2-66
---

[English](WI-533-release-v0-2-66.md) · [简体中文](WI-533-release-v0-2-66.zh-CN.md)

## Goal

review 済みで同期された default branch から v0.2.66 を公開します。この Release
には historical direct-merge recovery の互換修正を含め、実際の merge parent と
archive Contract base を分離します。これにより Pull Request を捏造せず、履歴を
書き換えずに bundled merge を記録できます。

## Scope and boundary

- Workspace version、lockfile、release workflow、release documentation、三言語 parity 登録。
- immutable release archive、manifest、checksum、SBOM、provenance、download artifact の adopter/N-1 acceptance。
- 公開後の Runtime install と自己 repository の health check。

対象 repository `/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`
は read-only です。.ai/ を変更せず、PR identity を捏造しません。global Agent/MCP
設定と source-build fallback は対象外です。

## Acceptance

- package/lockfile は v0.2.66 を示し、historical release fact を変更しない。
- 同期済み `main` から annotated v0.2.66 tag を作成する前に hosted check が通る。
- 公開 artifact は同一 tag、commit、bytes、SHA256SUMS、SBOM、provenance subject を bind する。
- public/N-1 acceptance は immutable download artifact のみを使い、repository isolation と一時 root cleanup を証明し、`first-adopter-smoke=not_ready` を保持する。
- 公開 binary の install 後、inspect/status/doctor/Agent doctor と docs check が通る。
- 完了前に visible human Outcome、archive、finalization、close、正確な branch/worktree cleanup を記録する。

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

Publication と post-release acceptance は独立した事実です。post-release が失敗しても
`releasePublished: true` と `adopterAcceptance: failed` を記録し、公開 Release truth は書き換えません。
