---
author: AI Cockpit maintainers
title: "WI-520 — v0.2.64 release と object adopter 互換性受入れ"
description: "merge 済みの historical finalization 互換修正を公開し、object repository を変更せず public artifact を検証します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-520-release-v0-2-64
lastVerifiedBy: WI-520-release-v0-2-64
---

[English](WI-520-release-v0-2-64.md) · [简体中文](WI-520-release-v0-2-64.zh-CN.md)

## Goal

review 済みで同期された default branch から v0.2.64 を公開します。この release
には WI-518 の historical direct-merge apply path と正確な診断が含まれます。
公開後の adopter acceptance は download した public artifact だけを使い、object
repository は read-only のまま object team が実行します。

## Scope

- 三言語の current release documentation と workspace package/Cargo.lock version。
- release workflow、public adopter acceptance、N-1 acceptance と cleanup/isolation wrapper。
- この Work Item の三言語 documentation と parity 登録。
- immutable tag、public Release assets、checksum、SBOM、provenance、download binary
  の install/health check。

`/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator` は明示的に read-only
です。.ai/ record を手編集したり、PR identity を作ったりしません。global Agent/MCP 設定、
source-build fallback、無関係な Runtime behavior は対象外です。

## Acceptance

- workspace package と Cargo.lock が v0.2.64 を示し、過去の release fact を書き換えない。
- 同期済み `main` の hosted checks が通過してから annotated v0.2.64 tag を作成する。失敗した
  publication tag は再利用しない。
- public Release manifest、5 archives、5 SBOMs、SHA256SUMS、provenance が同じ tag、bytes、
  targets、digest を bind する。
- public adopter と N-1 acceptance は immutable download artifact のみを使い、HOME/XDG isolation、
  TMPDIR/CARGO_HOME の分類、cleanup、`first-adopter-smoke=not_ready` を証明する。
- public binary を本 repository に install し、`inspect`、`status`、`doctor`、Agent doctor、
  documentation promotion check を通過する。
- release 完了を宣言する前に、visible Outcome、archive、finalization、close、正確な branch/worktree
  cleanup を記録する。

## Verification

```text
cargo metadata --locked --format-version 1
cargo test --locked --workspace
tests/release/version_consistency.sh --repo <repo>
tests/release/action_runtime_policy.sh .github/workflows/ci.yml .github/workflows/release.yml
tests/release/adopter_acceptance_test.sh
tests/release/adopter_upgrade_acceptance_test.sh
tests/docs/promote_closed_work_item.py --repo <repo> --check-all
tests/docs/documentation_acceptance.sh
tests/docs/parity_status_check.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
```

publication と post-release acceptance は別の事実です。public acceptance に失敗した場合は
`releasePublished: true` と `adopterAcceptance: failed` を記録し、Release truth を書き戻しません。
immutable Release の成立後に final adopter receipt と object team 向け手順を提示します。
