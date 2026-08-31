---
author: AI Cockpit maintainers
title: "WI-452 — v0.2.51 Release"
workItemId: WI-452-release-v0-2-51
description: "不変な公開 artifact から v0.2.51 Runtime を公開・検証する。"
audience: [adopter, maintainer, reviewer]
status: recovered
authority: human-authorized
lastVerifiedBy: WI-452-release-v0-2-51
---

# WI-452 — v0.2.51 Release

レビュー済みで同期された `main` から v0.2.51 を公開し、不変な公開 artifact
だけを使って install と adopter acceptance を行う。この Work Item は provider
context が provisional のまま archive されたため、immutable な記録を書き換えず
WI-453 が recovery します。失敗した過去の tag は不変履歴として保持し、再利用しない。

[English](WI-452-release-v0-2-51.md) · [简体中文](WI-452-release-v0-2-51.zh-CN.md)

## Scope

- workspace package identity を v0.2.51 に更新する。
- 現在の三言語 release、distribution、architecture、versioning ドキュメントを同期する。
- 公開前に release policy、source archive、checksum/SBOM、locked workspace gate を実行する。
- merge 後は不変 tag を公開し、download artifact だけで install と adopter acceptance を行う。

## Boundary

object repository、ユーザー全体の Agent/MCP 設定、失敗した release tag は変更しない。
source checkout と workspace binary は release acceptance の入力にしない。

## Verification

- `cargo test --locked --workspace`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/release/workflow_policy.sh .github/workflows/release.yml`
- `bash tests/release/source_archive_policy_test.sh`
- `bash tests/release/version_consistency_test.sh`
