---
author: AI Cockpit maintainers
title: "WI-358 — v0.2.35 release と lifecycle-entry compatibility"
workItemId: WI-358-release-v0-2-35
description: "adopter cleanup-order fix を公開し、legacy close record が新しい Work Item を deadlock させないようにする。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-358-release-v0-2-35
terminalArchive: .ai/work-items/archive/WI-358-release-v0-2-35.contract.json
terminalVerification: .ai/evidence/WI-358-release-v0-2-35.verification.json
terminalFinalization: .ai/decisions/WI-358-release-v0-2-35.finalize.json
terminalDecision: .ai/decisions/WI-358-release-v0-2-35.close.json
capabilityClaims: [release_distribution, lifecycle_entry_compatibility]
---

# WI-358 — v0.2.35 release と lifecycle-entry compatibility

[English](WI-358-release-v0-2-35.md) · [简体中文](WI-358-release-v0-2-35.zh-CN.md)

## Intent

merge 済みの adopter acceptance ordering fix を公開 v0.2.35 Release として公開します。
新しい archive には fail-closed の close gate を維持し、marker のない古い archive bytes は
historical として扱って新しい Work Item の作成を永久に妨げないようにします。

## Scope

- 新しい archive manifest に `closeRequired` marker を追加する。
- 新 entry gate は marker 付きの current archive だけを close 必須とし、marker のない
  historical bytes は保持する。invalid または current の close record は引き続き block する。
- historical/current archive の repository regression test を追加する。
- Cargo version と三言語の release/versioning 文書を揃え、v0.2.34 の publication failure を保持する。
- reviewed hosted release workflow のみで公開し、公開後は実際の artifact を受入れ確認する。

## Boundary

過去の Contract、close、evidence、archive bytes は書き換えません。human decision の推測や
外部 Homebrew tap の変更も行いません。公開後の失敗は `releasePublished: true` と acceptance
failure のまま記録します。

## Acceptance

1. workspace package と `Cargo.lock` は 0.2.35、tag は `v0.2.35` になる。
2. 新しい archive manifest は `closeRequired: true` を持ち、marker 付きで identity-bound
   close がない archive は引き続き block される。
3. marker のない historical archive は新しい Work Item entry を deadlock させず、current
   green Outcome に昇格しない。
4. publication 前の documentation、release policy、version consistency、workspace verification
   がすべて pass する。
5. hosted Release は manifest、`SHA256SUMS`、SBOM、provenance、staged adopter checks を bind
   し、public acceptance は downloaded binary identity、lifecycle、isolation、evidence reuse、
   temporary root cleanup を証明する。

## Verification

Runtime lifecycle evidence、hosted PR checks、release workflow、public binary digest、adopter
acceptance receipt が authoritative record です。terminal lifecycle: archive
`.ai/work-items/archive/WI-358-release-v0-2-35.contract.json`; verification
`.ai/evidence/WI-358-release-v0-2-35.verification.json`; finalization
`.ai/decisions/WI-358-release-v0-2-35.finalize.json`; close
`.ai/decisions/WI-358-release-v0-2-35.close.json`。
