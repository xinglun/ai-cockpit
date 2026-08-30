---
author: AI Cockpit maintainers
title: WI-416 — v0.2.43 post-release adopter 基線
description: 公開 v0.2.43 adopter acceptance receipt と Runtime identity を保存する。
workItemId: WI-416-release-v0-2-43-adopter-baseline
audience: [adopter, contributor, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-416-release-v0-2-43-adopter-baseline
terminalArchive: .ai/work-items/archive/WI-416-release-v0-2-43-adopter-baseline.contract.json
terminalVerification: .ai/evidence/WI-416-release-v0-2-43-adopter-baseline.verification.json
terminalFinalization: .ai/decisions/WI-416-release-v0-2-43-adopter-baseline.finalize.json
terminalDecision: .ai/decisions/WI-416-release-v0-2-43-adopter-baseline.close.json
---

# WI-416 — v0.2.43 post-release adopter 基線

[English](WI-416-release-v0-2-43-adopter-baseline.md) · [简体中文](WI-416-release-v0-2-43-adopter-baseline.zh-CN.md)

## Intent

公開 v0.2.43 Release の post-release adopter acceptance baseline を再現可能な形で
保存する。evidence は download 済み archive と binary を Release identity、adopter
repository identity、lifecycle、isolation、temporary run cleanup に bind する。

## Evidence boundary

完全な harness 出力は
`.ai/evidence/WI-416-release-v0-2-43-adopter-acceptance/` に保存した。
`runtime.json`（archive digest と binary SHA-256）、`repository.json`、attach、profile、
Agent doctor、`state: not_ready` の `first-adopter-smoke`、evidence reuse、lifecycle、
isolation manifest、`cleanup.json`、`acceptance.json`、`SHA256SUMS` を含む。harness は
公開された immutable v0.2.43 archive のみを使用し、temporary run root を削除した。

## Acceptance

- `acceptance.json` は `releasePublished: true`、`adopterAcceptance: passed` を示し、
  全 step と cleanup validation が pass する。
- lifecycle verification evidence は schema 2 で adopter `repositoryId` を bind し、
  Runtime `0.2.43` と binary digest
  `sha256:d6334275904868d7e7e46a569e4198d75057d25f22997781df1a7097a3e70533` を記録する。
- 保存した checksum が全 artifact を検証し、過去の receipt は変更しない。

## Non-claims

これは public artifact acceptance の記録であり、新しい Runtime governance authority、
source build、第二の technology-stack acceptance を意味しない。
