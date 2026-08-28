---
author: AI Cockpit maintainers
title: "WI-366 — N-1 identity root-fix 後の v0.2.38 release preparation"
workItemId: WI-366-release-v0-2-38
description: "v0.2.37 N-1 Git identity root-fix 後の release を準備し、immutable public artifact の受入れを後続 Work Item に引き継ぐ。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-366-release-v0-2-38
terminalArchive: .ai/work-items/archive/WI-366-release-v0-2-38.contract.json
terminalVerification: .ai/evidence/WI-366-release-v0-2-38.verification.json
terminalFinalization: .ai/decisions/WI-366-release-v0-2-38.finalize.json
terminalDecision: .ai/decisions/WI-366-release-v0-2-38.close.json
capabilityClaims: [release_distribution, adopter_acceptance]
---

# WI-366 — N-1 identity root-fix 後の v0.2.38 release preparation

[English](WI-366-release-v0-2-38.md) · [简体中文](WI-366-release-v0-2-38.zh-CN.md)

## Intent

v0.2.37 の N-1 upgrade acceptance root-fix が review・merge された後、同期済み
`main` から v0.2.38 を準備します。public artifact の install と adopter acceptance
は post-release の successor Work Item の境界であり、本 WI では未公開の結果を主張しません。

## Scope と boundary

- workspace package metadata、lockfile、現在の三言語 release/versioning 文書を
  v0.2.38 に揃えます。
- tag 前に review 済み release policy、documentation、parity、staged adopter
  regression check を実行します。
- immutable public artifact の download と adopter/N-1 acceptance を行う successor
  Work Item への明示的な handoff を残します。
- 未公開 v0.2.37 candidate の失敗は immutable history として保持し、tag を移動・再利用しません。

Runtime behavior、historical evidence の書換え、global Agent/MCP config、source
fallback、第二 technology-stack adopter は本 WI の範囲外です。

## Acceptance

1. Cargo metadata と lockfile が一貫して 0.2.38 を報告します。
2. immutable tag 前に hosted CI と release policy gate がすべて pass します。
3. v0.2.37 N-1 Git identity failure は repository-local identity regression でカバーし、
   global Git configuration を要求しません。
4. post-release public artifact、install 済み binary、adopter isolation、N-1 acceptance
   は successor Work Item に明示的に引き継ぎ、publication 前には主張しません。
5. merge、finalization、close、正確な branch/worktree cleanup 後も repository は
   同期済み default branch で継続可能です。
6. close 済み WI-365 の三言語 Work Item projection は terminal evidence と parity row
   に一致して Implemented を報告します。

## Verification boundary

Runtime lifecycle は Contract、checkpoint、verification、archive、finalization、close
evidence を記録します。successor Work Item が immutable published tag に bind し、hosted
workflow、install、adopter receipt を記録するまで public artifact を主張しません。失敗した
v0.2.37 candidate は変更せず、install source にはしません。
