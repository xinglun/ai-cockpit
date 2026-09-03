---
author: AI Cockpit maintainers
title: "WI-545 — v0.2.68 release と公開 artifact acceptance"
description: "検証済み Runtime を公開し、公開インストールの証拠を結び付ける。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
workItemId: WI-545-release-v0-2-68
lastVerifiedBy: WI-545-release-v0-2-68
terminalArchive: .ai/work-items/archive/WI-545-release-v0-2-68.contract.json
terminalVerification: .ai/evidence/WI-545-release-v0-2-68.verification.json
terminalFinalization: .ai/decisions/WI-545-release-v0-2-68.finalize.json
terminalDecision: .ai/decisions/WI-545-release-v0-2-68.close.json
---

[English](WI-545-release-v0-2-68.md) · [简体中文](WI-545-release-v0-2-68.zh-CN.md)

# WI-545 — v0.2.68 release と公開 artifact acceptance

## Intent と goal

レビュー済みで同期済みの default branch から v0.2.68 を公開し、source や
workspace binary に fallback せず immutable な公開 artifact を install して受入れできることを確認する。

## Scope

- Cargo package identity と三言語の release/versioning ページを更新する。
- reference-parity ledger に release を登録し、Work Item の証跡パスを保持する。
- 公開前の quality/policy check と、公開後の artifact、adopter、N-1、installed Runtime、cleanup acceptance を実行する。

## Acceptance boundary

公開 Release、archive/SBOM/provenance digest、installed binary は immutable tag と一致しなければならない。人間所有の Contract fields が入力されるまで `first-adopter-smoke` は `not_ready` のままにする。install は repository を attach せず、この Work Item は object repository や global Agent/MCP configuration を変更しない。

## Verification

active Contract が command と evidence の authority である。terminal handoff には status、unknowns、evidence、human decision、next action を含む可視の human Outcome、documentation promotion、正確な branch/worktree cleanup を含める。
