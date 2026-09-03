---
author: AI Cockpit maintainers
title: "WI-526 — v0.2.65 release and object-adopter recovery acceptance"
description: "direct-merge recovery context 互換修正を公開し、対象リポジトリを変更せず immutable artifact を検証する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-526-release-v0-2-65
lastVerifiedBy: WI-526-release-v0-2-65
terminalArchive: .ai/work-items/archive/WI-526-release-v0-2-65.contract.json
terminalVerification: .ai/evidence/WI-526-release-v0-2-65.verification.json
terminalFinalization: .ai/decisions/WI-526-release-v0-2-65.finalize.json
terminalDecision: .ai/decisions/WI-526-release-v0-2-65.close.json
---

[English](WI-526-release-v0-2-65.md) · [简体中文](WI-526-release-v0-2-65.zh-CN.md)

## 目的

レビュー済みで同期された default branch から v0.2.65 を公開する。本リリースには
direct-merge recovery context 互換修正と終端ドキュメント投影の修正を含める。対象リポジトリは
read-only とし、公開後は対象チームが自身で受け入れ検証を行う。

## 範囲

- 三言語の workspace package/lockfile version と現行リリース文書。
- リリースフローと immutable 公開 artifact の証跡。
- 本 Work Item の三言語文書と parity 登録。
- 不変の archive、verification、finalization、close evidence に基づき、クローズ済み WI-527 の terminal documentation を昇格する。
- download binary の install、health、isolation、adopter acceptance。

対象リポジトリ `/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator` は明示的に
read-only。`.ai/` の手編集や PR identity の捏造は禁止する。グローバル Agent/MCP 設定と
source-build fallback は範囲外。

## 受入れ

- package と lockfile が v0.2.65 を示し、履歴の release fact を保持する。
- 同期済み `main` から annotated v0.2.65 tag を作成する前に hosted checks が通過する。
- 公開 archive、SHA256SUMS、SBOM、provenance、manifest が同じ tag と bytes を束縛する。
- download artifact のみで adopter/N-1 acceptance を行い isolation と一時領域 cleanup を証明する。
- 公開 binary を install し repository health/documentation checks がすべて成功する。
- 可視 Outcome、archive、finalization、close、正確な branch/worktree cleanup を記録する。

## 検証

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
python3 tests/ci/governance_integrity_gate.py --repo <repo> --report <report>
```

公開と公開後 acceptance は別の事実である。公開後に失敗した場合は
`releasePublished: true` と `adopterAcceptance: failed` を記録し、release truth は書き換えない。
