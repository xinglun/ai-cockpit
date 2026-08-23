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
を記録します。一般利用者は [Current reader route](../current/README.ja.md) から開始し、field-level
mapping は [Contract と Summary の fields](contract-fields.ja.md) を参照してください。

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
| Explicit Agent Discovery / Adapter layer | Implemented | Agent install は explicit、owned、reversible、repository-local であり、生成 guidance は Contract-first/pause/Summary/Outcome/closure を伝える。Cursor の新規 target は `.cursor/rules/ai-cockpit.mdc`、managed な legacy `.md` は保持する。 |
| Work Item lifecycle と governance decision | Partial | core lifecycle と human decision record はあるが、reference の広い status、cost、recovery projection は一つの adopter interface に統合されていない。 |
| Resource finalization と正確な branch/worktree closure | Implemented | Runtime は `finalize-plan`、`finalize`、`finalize-verify` を提供し、typed receipt に repository、Work Item、Contract、PR、branch、worktree、Runtime identity を bind する。欠落/unknown cleanup は fail-closed、Runtime upgrade 後の archived evidence は明示的に historical として扱う。 |
| Task Outcome と Human Benefit report | Partial | WI-136 は Rust-native strict projection、append-only event stream、archive binding、close final report を追加する。完全な recovery/event reconstruction はこの境界外。Evidence: `.ai/evidence/WI-136-task-outcome-report.verification.json`。 |
| Archive 済み Outcome の path projection | Implemented | WI-148 は manifest を束縛する前に、新規 archive の生成 report reference と `changedPaths` を active から archive へ投影する。既存の historical archive bytes は不変のまま。 |
| Contract preflight human-review gate | Implemented | 不完全な scaffold Contract は明示的な `reviewState` 付き yellow となり、repository/Contract/snapshot binding を保存し、human confirmation なしでは checkpoint を越えない。 |
| Contract V2 の structured intent と strict schema | Implemented | WI-121 は structured intent、typed sources/verification、strict な unknown-field/duplicate-key fail-closed、`humanDecisionRequest`、preflight/checkpoint gate を提供する。 |
| Contract の cross-field dimensions（intent/scope/evidence/decision）validation | Implemented | WI-122 は high-risk scenario coverage、stable acceptance evidence、intent alignment、参照源と同じ 20 dimension の final receipt を検証する。任意の `fourPillarProjection` は表示用であり、literal `4D` protocol field はない。 |
| Contract parallel boundary と slot | Implemented | WI-123 は repository-local boundary validation、保守的な overlap 判定、exclusive slot lease を提供し、unknown または malformed state は fail-closed になる。 |
| Bounded verification と fail-closed evidence reuse | Implemented | Runtime identity、snapshot/toolchain/environment binding、receipt、fail-closed validation を記録する。 |
| MCP repository binding | Implemented | repository-bound stdio MCP が explicit binding で同じ governed service を公開する。 |
| Human-facing MCP projection | Implemented | Runtime が OutcomeV2 を検証し localized `humanHandoff` を生成する。Agent または conversation layer は選択・表示・伝達を担当するが、presentation をガバナンス権限として扱わない。 |
| Public Release と fresh-adopter acceptance | Partial | v0.2.16 の complete post-release adopter baseline は `x86_64-unknown-linux-gnu` のみ。他の target は build/smoke evidence である。 |
| Second-technology-stack adopter acceptance | Deferred | current harness は Cargo adopter を使い、第二の technology stack は future work とする。 |
| Runtime-only upgrade と repository migration | Implemented | compatibility check と explicit migration が historical record を保持し Runtime identity を bind する。 |
| N-1 old-adopter upgrade acceptance | Implemented | public-artifact harness が old-schema detection、approval、history preservation、continued operation を確認する。 |
| Adopter capability manifest と status projection | Deferred | `capability show` と `status` は truthful な Runtime/repository view であり、reference の full adopter manifest/status projection ではない。 |
| Recovery state machine と rich recovery projection | Partial | blocked Outcome、append-only recovery receipt、predecessor に bind された retry/successor decision、人間/MCP projection を追加したが、paused/stale/cancelled/rollback の広い surface は reference より狭い。 |
| Multilingual semantic parity gate | Partial | CLI human output は localize されるが、全 report の field-by-field semantic parity は CI gate ではない。 |
| Legacy evidence boundary | Implemented | legacy evidence は historical input のままで、fresh green verification に昇格しない。 |
| Contract source language | Implemented | Contract の intent、scope、acceptance、authority は source text のまま保持し、翻訳で bytes を変更しない。 |
| Installation と provider configuration | External boundary | binary delivery と provider/global configuration は repository governance state の外部で分離される。 |

この matrix は working core と full reference surface parity を意図的に区別します。1 行の
green はその boundary だけを証明し、external identity、provider authorization、branch protection、
production readiness、organization approval を与えるものではありません。

## 現在の実装 baseline

現在の `main` branch には、次の Contract と governance boundary が含まれます。Work Item
document は利用者向けの範囲を示し、repository evidence path は各 boundary の machine-readable
verification record です。

| Work Item | Current Runtime status | Evidence と document |
| --- | --- | --- |
| WI-121 — Contract V2 | Implemented | [Work Item](../work-items/WI-121-contract-v2.ja.md); `.ai/evidence/WI-121-contract-v2.verification.json` |
| WI-122 — Scenario、Acceptance、最終 dimensions | Implemented | [Work Item](../work-items/WI-122-scenarios-acceptance-final-dimensions.ja.md); `.ai/evidence/WI-122-scenarios-acceptance-final-dimensions.verification.json` |
| WI-123 — Parallel Contract boundary と slot | Implemented | [Work Item](../work-items/WI-123-parallel-contract-boundary.ja.md); `.ai/evidence/WI-123-parallel-contract-boundary.verification.json` |
| WI-125 — Contract V2 schema boundary | Implemented | [Work Item](../work-items/WI-125-contract-schema.ja.md); `.ai/evidence/WI-125-contract-schema.verification.json` |
| WI-126 — Read-only status と human handoff | Implemented | [Work Item](../work-items/WI-126-status-outcome.ja.md); `.ai/evidence/WI-126-status-outcome.verification.json` |
| WI-128 — Release acceptance cleanup | Implemented | [Work Item](../work-items/WI-128-release-acceptance-cleanup.ja.md); `.ai/evidence/WI-128-release-acceptance-cleanup.verification.json` |
| WI-129 — Reference parity completeness | Implemented | [Work Item](../work-items/WI-129-parity-gate.ja.md); `.ai/evidence/WI-129-parity-gate.verification.json` |
| WI-130 — Closed Work Item status projection | Implemented | [Work Item](../work-items/WI-130-status-closed-projection.ja.md); `.ai/evidence/WI-130-status-closed-projection.verification.json`; `.ai/decisions/WI-130-status-closed-projection.close.json` |
| WI-131 — 検証証拠 timestamp の fail-closed 検査 | Implemented | [Work Item](../work-items/WI-131-evidence-timestamp.ja.md); `.ai/evidence/WI-131-evidence-timestamp.verification.json`; `.ai/decisions/WI-131-evidence-timestamp.close.json` |
| WI-132 — Agent adapter と provider surface の parity | Implemented | [Work Item](../work-items/WI-132-agent-adapter-parity.ja.md); `.ai/evidence/WI-132-agent-adapter-parity.verification.json`; `.ai/decisions/WI-132-agent-adapter-parity.close.json` |
| WI-133 — Documentation truth の整合 | Implemented | [Work Item](../work-items/WI-133-docs-truth.ja.md); `.ai/evidence/WI-133-docs-truth.verification.json`; `.ai/decisions/WI-133-docs-truth.close.json` |
| WI-135 — Repository に束縛された retention と close evidence | Implemented | [Work Item](../work-items/WI-135-repository-bound-evidence.ja.md); `.ai/evidence/WI-135-repository-bound-evidence.verification.json`; `.ai/decisions/WI-135-repository-bound-evidence.close.json` |
| WI-136 — Task Outcome と Human Benefit report | Implemented | [Work Item](../work-items/WI-136-task-outcome-report.ja.md); `.ai/evidence/WI-136-task-outcome-report.verification.json`; `.ai/decisions/WI-136-task-outcome-report.close.json` |
| WI-140 — Verification semantics と Artifact archive integrity | Implemented | [Work Item](../work-items/WI-140-verification-semantics.ja.md); `.ai/evidence/WI-140-verification-semantics.verification.json`; `.ai/decisions/WI-140-verification-semantics.close.json` |
| WI-141 — Policy-driven verification planner | Implemented | [Work Item](../work-items/WI-141-policy-planner.ja.md); `.ai/evidence/WI-141-policy-planner.verification.json`; `.ai/decisions/WI-141-policy-planner.close.json` |
| WI-142 — Affected verification と dependency confidence | Implemented | [Work Item](../work-items/WI-142-affected-verification.ja.md); `.ai/evidence/WI-142-affected-verification.verification.json`; `.ai/decisions/WI-142-affected-verification.close.json` |
| WI-143 — Intent scenario and stage binding | Implemented | [Work Item](../work-items/WI-143-intent-scenario-binding.ja.md); `.ai/evidence/WI-143-intent-scenario-binding.verification.json`; `.ai/decisions/WI-143-intent-scenario-binding.close.json` |
| WI-144 — Work Item 間の物理実行再利用 | Implemented | [Work Item](../work-items/WI-144-cross-work-item-dedup.ja.md); `.ai/evidence/WI-144-cross-work-item-dedup.verification.json`; `.ai/decisions/WI-144-cross-work-item-dedup.close.json` |
| WI-145 — CI Runtime verification shadow | Implemented | [Work Item](../work-items/WI-145-ci-runtime-shadow.ja.md); `.ai/evidence/WI-145-ci-runtime-shadow.verification.json`; `.ai/decisions/WI-145-ci-runtime-shadow.close.json` |
| WI-146 — Verification コスト観測 | Implemented | [Work Item](../work-items/WI-146-verification-cost-observation.ja.md); [参考文書](verification-cost.ja.md); `.ai/evidence/WI-146-verification-cost-observation.verification.json`; `.ai/decisions/WI-146-verification-cost-observation.close.json` |
| WI-147 — Verification route convergence | Implemented | [Work Item](../work-items/WI-147-verification-route-convergence.ja.md); [参考文書](verification-route.ja.md); `.ai/evidence/WI-147-verification-route-convergence.verification.json`; `.ai/decisions/WI-147-verification-route-convergence.close.json` |
| WI-148 — Archive 済み Outcome の path projection | Implemented | [Work Item](../work-items/WI-148-outcome-archive-path.ja.md); [参考文書](outcome-report.ja.md); `.ai/evidence/WI-148-outcome-archive-path.verification.json`; `.ai/decisions/WI-148-outcome-archive-path.close.json` |
| WI-149 — 構造化された Release adopter decision | Implemented | [Work Item](../work-items/WI-149-release-decision-acceptance.ja.md); [Release distribution](../release/distribution.ja.md); `.ai/evidence/WI-149-release-decision-acceptance.verification.json`; `.ai/decisions/WI-149-release-decision-acceptance.close.json` |
| WI-150 — v0.2.16 Release baseline | Implemented | [Work Item](../work-items/WI-150-release-v0-2-16.ja.md); [v0.2.16 Release](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.16); `.ai/evidence/WI-150-release-v0-2-16.verification.json` |
| WI-151 — v0.2.16 post-release self-governance acceptance | Implemented | [Work Item](../work-items/WI-151-post-release-v0-2-16-self-governance.ja.md); `.ai/evidence/WI-151-post-release-v0-2-16-self-governance.verification.json`; `.ai/decisions/WI-151-post-release-v0-2-16-self-governance.close.json` |
| WI-152 — v0.2.16 documentation parity correction | Implemented | [Work Item](../work-items/WI-152-documentation-parity-after-v0-2-16.ja.md); `.ai/evidence/WI-152-documentation-parity-after-v0-2-16.verification.json`; `.ai/decisions/WI-152-documentation-parity-after-v0-2-16.close.json` |
| WI-153 — Historical evidence projection | Implemented | [Work Item](../work-items/WI-153-historical-evidence-projection.ja.md); `.ai/evidence/WI-153-historical-evidence-projection.verification.json`; `.ai/decisions/WI-153-historical-evidence-projection.close.json` |
| WI-154 — Policy に束縛された Runtime verification route | Implemented | [Work Item](../work-items/WI-154-policy-bound-runtime-route.ja.md); [verification route](verification-route.ja.md); `.ai/evidence/WI-154-policy-bound-runtime-route.verification.json`; `.ai/decisions/WI-154-policy-bound-runtime-route.close.json` |
| WI-155 — CI/release gate の収束 | Implemented | [Work Item](../work-items/WI-155-ci-release-gate-convergence.ja.md); [Release distribution](../release/distribution.ja.md); `.ai/evidence/WI-155-ci-release-gate-convergence.verification.json`; `.ai/decisions/WI-155-ci-release-gate-convergence.close.json` |
| WI-156 — 物理実行と Work Item 証拠レシート | Implemented | [Work Item](../work-items/WI-156-physical-execution-receipt.ja.md); `.ai/evidence/WI-156-physical-execution-receipt.verification.json`; `.ai/decisions/WI-156-physical-execution-receipt.close.json` |
| WI-157 — v0.2.17 Release と adopter acceptance | Implemented | [Work Item](../work-items/WI-157-release-v0-2-17-adopter-acceptance.ja.md); [公開 Release](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.17)、`.ai/evidence/external/v0.2.17/adopter/`、`.ai/evidence/external/v0.2.17/upgrade/`、`.ai/evidence/WI-157-release-v0-2-17-adopter-acceptance.verification.json`。 |
| WI-160 — Resource finalization と branch/worktree closure の baseline | Implemented | [Work Item](../work-items/WI-160-resource-finalization-baseline.ja.md); `.ai/evidence/WI-160-resource-finalization-baseline.verification.json`; `.ai/work-items/archive/WI-160-resource-finalization-baseline.archive.json`; `.ai/decisions/WI-160-resource-finalization-baseline.close.json`。Runtime command/receipt 統合は WI-159、historical-runtime close compatibility は WI-161 で実装する。 |
| WI-161 — Historical Runtime evidence close compatibility | Implemented | [Work Item](../work-items/WI-161-historical-runtime-close.ja.md); archived evidence は不変のまま foreign Runtime bytes を historical として投影する。Regression evidence: `.ai/evidence/WI-161-historical-runtime-close.verification.json` |

## 現在の境界

1 つの installed Runtime は複数の独立した repository を治理できます。Protocol、Work Item、evidence、
knowledge、adapter record は repository ごとに分離されます。今後も explicit repository binding、
evidence isolation、human-owned decision、Runtime delivery と repository state の分離を維持します。

Work Item を close した後は、同じ release-audit cycle で三言語の文書を finalize します。
status は `implemented`、archived verification/close evidence への link、parity baseline の行を
一致させます。この documentation-truth rule は過去の evidence を書き換えません。

Resource finalization は別の closure 境界です。正確な branch と worktree は
`finalize-plan` → `finalize` → `finalize-verify` を通過してから `close` します。
provider/resource の状態が `unknown` なら open のままにし、保持する resource には明示的で
期限付きの Human Decision が必要です。WI-160 はこの policy と static gate を記録し、WI-159
は Runtime command/receipt を実装します。Runtime upgrade 後も historical verification
evidence は書き換えず current failure として扱いません。close を実行する Runtime に bind
されるのは新しい finalization receipt です。

## Scenario・Acceptance・最終 dimensions の projection

Runtime は次の三つの任意 projection を検証します（ただし内容や証拠を
生成しません）。高リスク Contract では `scenarioCoverage` が必須です。
Summary の各項目には `required`、`status`、`evidence` が必要で、
`status` が `not_applicable` の場合は `reason` も必要です。required な
scenario が未検証のままなら、高リスク Work Item は fail-closed になります。

`A1: ...` のような番号付き Acceptance は stable ID と Summary の
`acceptanceEvidence` mapping を有効にします。番号のない旧 Acceptance は
読み取り可能なまま保持され、Runtime が ID を推測することはありません。
`intentAlignment` は任意の projection で、欠落は `unknown` のままです。
`resolved` または `unresolved` を示す場合は、それぞれ明示的な evidence または
reason が必要です。

最終 acceptance は参照源と同じ 20 個の dimension 名を使用し、decision は
`GO`、`CONDITIONAL_GO`、`NO_GO` のいずれかです。`GO` には検証済みの
`real_adopter` と `provider_evidence` が必要です。欠落・余分・不正形式・
identity 不一致は fail-closed になります。任意の `fourPillarProjection` は
表示用であり、曖昧な文字列 `4D` の protocol field は導入しません。Runtime は
evidence を合成せず、local projection を provider/enterprise assurance として
扱いません。
