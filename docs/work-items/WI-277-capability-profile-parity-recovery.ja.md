---
author: AI Cockpit maintainers
title: "WI-277 — capability profile parity recovery"
workItemId: WI-277-capability-profile-parity-recovery
description: "WI-276 の復旧後にホスト側 parity 登録を復元し、adopter repository が capability/profile を継承できることを検証する。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-277-capability-profile-parity-recovery
terminalArchive: .ai/work-items/archive/WI-277-capability-profile-parity-recovery.contract.json
terminalVerification: .ai/evidence/WI-277-capability-profile-parity-recovery.verification.json
terminalFinalization: .ai/decisions/WI-277-capability-profile-parity-recovery.finalize.26a5046378afcc467b75e703bb6b7dd83d53f665d76605695f7f28a6b9b8f564.json
terminalDecision: .ai/decisions/WI-277-capability-profile-parity-recovery.close.json
authority: canonical
---

# WI-277 — capability profile parity recovery

## Intent

先行 Work Item の欠落していた三言語 reference-parity 登録を復元し、
repository-bound CLI と MCP 投影から strict capability/profile 宣言を
利用できることを確認する。

## Scope

- WI-276 の不変な recovery linkage を保持する。
- 検証より前に英語・日本語・中国語の parity 行を登録する。
- 二つの repository の分離、malformed/stale 宣言の拒否、read-only 投影を検証する。
- reviewed PR、merge observation、正確な cleanup、close decision を束ねる。

## Boundary

WI-276 の archive/evidence bytes、capability semantics、global Agent/MCP 設定、
後続の architecture cleanup は変更しない。

## Acceptance and verification

- Rust、documentation、conformance、governance の gate を一回の bounded Runtime 実行で通過する。
- hosted quality、Windows Runtime、V1 behavioral oracle を通過する。
- merge と branch/worktree cleanup を Runtime receipt で記録する。

