---
author: AI Cockpit maintainers
title: WI-419 — v0.2.44 post-release adopter 基線
description: 公開 v0.2.44 adopter acceptance receipt と Runtime identity を保存する。
workItemId: WI-419-release-v0-2-44-adopter-baseline
audience: [adopter, contributor, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-419-release-v0-2-44-adopter-baseline
terminalArchive: .ai/work-items/archive/WI-419-release-v0-2-44-adopter-baseline.contract.json
terminalVerification: .ai/evidence/WI-419-release-v0-2-44-adopter-baseline.verification.json
terminalFinalization: .ai/decisions/WI-419-release-v0-2-44-adopter-baseline.finalize.json
terminalDecision: .ai/decisions/WI-419-release-v0-2-44-adopter-baseline.close.json
---

# WI-419 — v0.2.44 post-release adopter 基線

[English](WI-419-release-v0-2-44-adopter-baseline.md) · [简体中文](WI-419-release-v0-2-44-adopter-baseline.zh-CN.md)

## Intent

公開 v0.2.44 Release の post-release adopter acceptance baseline を再現可能な形で
保存する。receipt は download 済み archive と binary を Release identity、adopter
repository identity、lifecycle、isolation、evidence reuse、temporary-run cleanup に bind する。

## Evidence boundary

完全な public-binary harness 出力は
`.ai/evidence/WI-419-release-v0-2-44-adopter-acceptance/` に保存した。`runtime.json`
（archive と binary SHA-256）、`repository.json`、attach、profile、Agent doctor、
`state: not_ready` の `first-adopter-smoke`、evidence reuse、完全な Work Item lifecycle、
isolation manifest、`cleanup.json`、`acceptance.json`、`SHA256SUMS` を含む。immutable な
公開 v0.2.44 `aarch64-apple-darwin` archive のみを download し、temporary run root の削除を検証した。

## Acceptance

- `acceptance.json` は `releasePublished: true`、`adopterAcceptance: passed` を示し、全 step と cleanup validation が pass する。
- Runtime は `0.2.44`、binary digest は
  `sha256:69d28c970c2b89534e63cb685c6cc02a2f135d3067b6a84feaabce2adce1d5e5`、adopter
  repository identity は `sha256:26301b33fabbb72aaacb48c8f9ccac335be8ca5aa42b9e98941324d2108a8df1` である。
- lifecycle verification evidence は schema 2 で、evidence reuse は新しい process spawn なしで成功し、structured close decision を記録した。
  保存した checksum が全 artifact を検証する。

## Non-claims

これは public artifact acceptance の記録であり、新しい Runtime governance authority、source build、V1 fixture、第二の technology-stack acceptance を意味しない。
過去の receipt は immutable のまま保持する。
