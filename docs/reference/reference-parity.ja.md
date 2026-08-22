---
author: AI Cockpit maintainers
title: "Reference source parity"
description: "Maintainer と reviewer 向けの evidence-based product boundary 比較。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - reference_parity
---

# Reference source parity

これは audit comparison であり、adopter の操作手順ではありません。Rust Runtime が
reference product boundary と一致する部分、partial/deferred の部分、external responsibility
を記録します。一般利用者は [Current reader route](../current/README.ja.md) から開始してください。

## Truth state

matrix は次の 4 state だけを使います。

- **Implemented** — 記載した boundary が実装され、current evidence で確認できる。
- **Partial** — core boundary はあるが、reference surface または assurance の方が広い。
- **Deferred** — 現在の Runtime boundary には意図的に含めない。
- **External boundary** — Agent host、provider、organization、外部 system が担当する。

## Parity matrix

| Reference concern | Rust Runtime status | Evidence と boundary |
| --- | --- | --- |
| Reader-first entry と language switching | Implemented | root と route README は English、Simplified Chinese、日本語で相互リンクする。 |
| Purpose、problem、architecture、capability overview | Implemented | philosophy、architecture、capability route が current Runtime と責任範囲を説明する。 |
| Shared Runtime と request-scoped repository context | Implemented | 明示的な `--repo` binding と repository isolation tests で context/evidence を分離する。 |
| Repository attach と minimum scaffold | Implemented | `attach` は repository-owned Protocol scaffold を作り、Runtime の copy を repository 内に install しない。 |
| Explicit Agent Discovery / Adapter layer | Implemented | Agent install は explicit、owned、reversible、repository-local である。 |
| Work Item lifecycle と governance decision | Partial | core lifecycle と human decision record はあるが、reference の広い status、cost、recovery projection は一つの adopter interface に統合されていない。 |
| Contract preflight human-review gate | Implemented | 不完全な scaffold Contract は明示的な `reviewState` 付き yellow となり、repository/Contract/snapshot binding を保存し、human confirmation なしでは checkpoint を越えない。 |
| Bounded verification と fail-closed evidence reuse | Implemented | Runtime identity、snapshot/toolchain/environment binding、receipt、fail-closed validation を記録する。 |
| MCP repository binding | Implemented | repository-bound stdio MCP が explicit binding で同じ governed service を公開する。 |
| Human-facing MCP projection | Implemented | Runtime が OutcomeV2 を検証し localized `humanHandoff` を生成する。Agent または conversation layer は選択・表示・伝達を担当するが、presentation をガバナンス権限として扱わない。 |
| Public Release と fresh-adopter acceptance | Partial | v0.2.10 の complete post-release adopter baseline は `x86_64-unknown-linux-gnu` のみ。他の target は build/smoke evidence である。 |
| Second-technology-stack adopter acceptance | Deferred | current harness は Cargo adopter を使い、第二の technology stack は future work とする。 |
| Runtime-only upgrade と repository migration | Implemented | compatibility check と explicit migration が historical record を保持し Runtime identity を bind する。 |
| N-1 old-adopter upgrade acceptance | Implemented | public-artifact harness が old-schema detection、approval、history preservation、continued operation を確認する。 |
| Adopter capability manifest と status projection | Deferred | `capability show` と `status` は truthful な Runtime/repository view であり、reference の full adopter manifest/status projection ではない。 |
| Recovery state machine と rich recovery projection | Partial | stop と recovery guidance はあるが、paused/blocked/stale/cancelled/rollback の広い surface は reference より狭い。 |
| Multilingual semantic parity gate | Partial | CLI human output は localize されるが、全 report の field-by-field semantic parity は CI gate ではない。 |
| Legacy evidence boundary | Implemented | legacy evidence は historical input のままで、fresh green verification に昇格しない。 |
| Contract source language | Implemented | Contract の intent、scope、acceptance、authority は source text のまま保持し、翻訳で bytes を変更しない。 |
| Installation と provider configuration | External boundary | binary delivery と provider/global configuration は repository governance state の外部で分離される。 |

この matrix は working core と full reference surface parity を意図的に区別します。1 行の
green はその boundary だけを証明し、external identity、provider authorization、branch protection、
production readiness、organization approval を与えるものではありません。

## 現在の境界

1 つの installed Runtime は複数の独立した repository を治理できます。Protocol、Work Item、evidence、
knowledge、adapter record は repository ごとに分離されます。今後も explicit repository binding、
evidence isolation、human-owned decision、Runtime delivery と repository state の分離を維持します。
