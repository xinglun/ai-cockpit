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
| WI-166 — Release adopter acceptance の resource finalization | Implemented | [Archived Contract](../../.ai/work-items/archive/WI-166-release-acceptance-finalization.contract.json); [verification evidence](../../.ai/evidence/WI-166-release-acceptance-finalization.verification.json)。public と N-1 harness は structured close 前に resource finalization を bind する。v0.2.18 の original workflow failure は immutable な Release history として保持する。 |
| WI-167 — v0.2.19 公開 Release と adopter acceptance | Implemented | [v0.2.19 Release](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.19); immutable な binary と source baseline は `.ai/evidence/WI-167-release-v0-2-19-recovery.verification.json` に bind されています。v0.2.18 の original failure は immutable history として保持します。 |
| WI-168 — N-1 release acceptance finalization correction | Implemented | [Archived Contract](../../.ai/work-items/archive/WI-168-n-minus-one-finalization.contract.json); [verification evidence](../../.ai/evidence/WI-168-n-minus-one-finalization.verification.json)。old/new の N-1 Work Item は structured close 前に `finalize-plan` → `finalize` → `finalize-verify` を実行します。 |
| WI-169 — v0.2.20 公開 Release と adopter acceptance | Implemented | [v0.2.20 Release](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.20); [release workflow](https://github.com/xinglun/ai-cockpit/actions/runs/32617519173); `.ai/evidence/WI-169-release-v0-2-20.verification.json`。public ARM64 binary、adopter acceptance、v0.2.19→v0.2.20 N-1 acceptance は immutable な Runtime identity に bind されています。v0.2.19 の original N-1 failure は immutable history として保持します。 |
| WI-170 — v0.2.20 post-release parity と branch reconciliation | Implemented | [PR #125](https://github.com/xinglun/ai-cockpit/pull/125); `.ai/evidence/WI-170-post-release-parity-branch-reconciliation.verification.json`。archived Contract/Outcome と recovery decision は immutable な predecessor record を保持します。検証済み merged branch は cleanup し、dirty な historical worktree は retained のままです。 |
| WI-171 — finalization reconciliation successor | Implemented | [PR #126](https://github.com/xinglun/ai-cockpit/pull/126); `.ai/evidence/WI-171-finalization-reconciliation.verification.json`; `.ai/decisions/WI-171-finalization-reconciliation.finalize.json`; `.ai/decisions/WI-171-finalization-reconciliation.close.json`。不足していた finalize-plan → finalize → finalize-verify → close chain を記録し、WI-170 と Release truth は書き換えていません。 |
| WI-172 — v0.2.20 parity closure | Implemented | [PR #127](https://github.com/xinglun/ai-cockpit/pull/127); `.ai/evidence/WI-172-parity-closure.verification.json`。WI-170 と WI-171 は三言語 parity document で implemented として統一されています。 |
| WI-173 — v0.2.21 release baseline | Implemented | [PR #129](https://github.com/xinglun/ai-cockpit/pull/129)、merge commit `176e384efef41d2c25919734b1257170b9a13c00`；公開 [v0.2.21 Release](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.21)、workflow [32620133057](https://github.com/xinglun/ai-cockpit/actions/runs/32620133057)、公開 aarch64 macOS archive SHA256 `9438b975fb25531e3b1a7e349779b917ff41c5c5fa2ab62443c472ff5385cea5`、installed public binary SHA256 `38aa88d7976d27647a9ae4419f57d309df2a08717fedccb4e9a613b370433e88`；public adopter と N-1 acceptance は pass；closure [PR #130](https://github.com/xinglun/ai-cockpit/pull/130)、`.ai/decisions/WI-173-release-v0-2-21.finalize.json`、`.ai/decisions/WI-173-release-v0-2-21.close.json`。 |
| WI-174 — v0.2.21 post-release parity | Implemented | [PR #131](https://github.com/xinglun/ai-cockpit/pull/131)、merge commit `b8b2e7a9b8f36e237fcfe507ed946278a75ba0b7`；installed public v0.2.21 の documentation acceptance と post-release version consistency は pass；closure [PR #132](https://github.com/xinglun/ai-cockpit/pull/132)、`.ai/decisions/WI-174-post-release-parity-v0-2-21.finalize.json`、`.ai/decisions/WI-174-post-release-parity-v0-2-21.close.json`。 |
| WI-175 — v0.2.22 release baseline | Implemented | [PR #133](https://github.com/xinglun/ai-cockpit/pull/133)、merge commit `b75b828be99e5ddd1510d323ca3f72698d5666a7`；公開 [v0.2.22 Release](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.22)、workflow [32622398424](https://github.com/xinglun/ai-cockpit/actions/runs/32622398424)、公開 aarch64 macOS archive SHA256 `b74857298bc32b53a8b7a349b5d719cb670c4d9beb25b2414b562e4a7e13a145`；公開 Release workflow の adopter と N-1 acceptance job は pass。WI-176 が post-release の finalization recovery を記録し、closure は [PR #134](https://github.com/xinglun/ai-cockpit/pull/134)。 |
| WI-176 — WI-175 finalization reconciliation | Implemented | [PR #133](https://github.com/xinglun/ai-cockpit/pull/133) と closure [PR #134](https://github.com/xinglun/ai-cockpit/pull/134)；`.ai/evidence/WI-176-release-finalization-reconciliation.verification.json`；`.ai/decisions/WI-176-release-finalization-reconciliation.finalize.json`；過去の WI-175 bytes は保持し、不足していた finalize-plan → finalize → finalize-verify → close chain を記録しました。 |
| WI-177 — v0.2.22 public adopter acceptance baseline | Implemented | インストール済み公開 v0.2.22 binary SHA256 `fff455d0d88d9ca4fa96b5caba85d8a6a198e131bd6ecc5a33dd9bc5cc180ab2`；公開 archive SHA256 `b74857298bc32b53a8b7a349b5d719cb670c4d9beb25b2414b562e4a7e13a145`；runtime identity、attach/profile/agent doctor、not_ready scaffold、evidence reuse、lifecycle、isolation、cleanup を含む adopter evidence は `.ai/evidence/WI-177-post-release-adopter-v0-2-22/` に保持されています。WI-178 の finalization 完了後、この predecessor は historical として置換・close されました。 |
| WI-178 — v0.2.22 adopter finalization reconciliation | Implemented | [PR #135](https://github.com/xinglun/ai-cockpit/pull/135)、closure [PR #136](https://github.com/xinglun/ai-cockpit/pull/136)；`.ai/evidence/WI-178-post-release-adopter-finalization-reconciliation.verification.json`；`.ai/decisions/WI-178-post-release-adopter-finalization-reconciliation.finalize.json`；`.ai/decisions/WI-178-post-release-adopter-finalization-reconciliation.close.json`。merge 済み feature branch は削除し、共有 main worktree は明示的に retained/clean とし、公開 v0.2.22 で finalize-verify を通過しました。 |
| WI-179 — v0.2.22 post-release parity correction | Implemented | [PR #137](https://github.com/xinglun/ai-cockpit/pull/137)、closure [PR #138](https://github.com/xinglun/ai-cockpit/pull/138)；`.ai/evidence/WI-179-post-release-parity-v0-2-22.verification.json`；`.ai/decisions/WI-179-post-release-parity-v0-2-22.finalize.json`；`.ai/decisions/WI-179-post-release-parity-v0-2-22.close.json`。インストール済み公開 v0.2.22 の documentation と post-release consistency check は pass しました。 |
| WI-180 — parity status closure correction | Implemented | [PR #139](https://github.com/xinglun/ai-cockpit/pull/139)、closure [PR #140](https://github.com/xinglun/ai-cockpit/pull/140)；`.ai/evidence/WI-180-parity-status-closure-correction.verification.json`；`.ai/decisions/WI-180-parity-status-closure-correction.finalize.json`；`.ai/decisions/WI-180-parity-status-closure-correction.close.json`。最終 self-check で発見した WI-179 の stale status を記録する corrective Work Item であり、三言語の修正と再発防止 evidence を bind しています。 |
| WI-181 — parity evidence binding correction | Implemented | [PR #141](https://github.com/xinglun/ai-cockpit/pull/141)、closure [PR #144](https://github.com/xinglun/ai-cockpit/pull/144)；`.ai/evidence/WI-181-parity-evidence-binding.verification.json`；`.ai/decisions/WI-181-parity-evidence-binding.finalize.json`；`.ai/decisions/WI-181-parity-evidence-binding.close.json`。closed row に監査可能な evidence binding がない場合、parity gate は fail closed します。 |
| WI-182 — parallel lease atomic publication correction | Implemented | [PR #142](https://github.com/xinglun/ai-cockpit/pull/142)、closure [PR #143](https://github.com/xinglun/ai-cockpit/pull/143)；`.ai/evidence/WI-182-parallel-lease-atomic-install.verification.json`；`.ai/decisions/WI-182-parallel-lease-atomic-install.finalize.json`；`.ai/decisions/WI-182-parallel-lease-atomic-install.close.json`。parallel lease JSON を atomic に公開し、first-use EOF race を防止します。 |
| WI-183 — v0.2.23 release baseline | Implemented | [PR #145](https://github.com/xinglun/ai-cockpit/pull/145)、merge `1778e3c`；`.ai/evidence/WI-183-release-v0-2-23.verification.json`；`.ai/work-items/archive/WI-183-release-v0-2-23.archive.json`；`.ai/decisions/WI-183-release-v0-2-23.recovery.json`。三言語の current release baseline を v0.2.23 に更新し、v0.2.22 を隣接する N-1 として保持します。公開 Release と adopter evidence は post-release acceptance で bind します。 |
| WI-184 — v0.2.23 release finalization reconciliation | Implemented | [PR #146](https://github.com/xinglun/ai-cockpit/pull/146)、merge `aabff99`；`.ai/evidence/WI-184-release-v0-2-23-finalization-reconciliation.verification.json`；`.ai/decisions/WI-184-release-v0-2-23-finalization-reconciliation.finalize.json`；`.ai/decisions/WI-184-release-v0-2-23-finalization-reconciliation.close.json`。predecessor の recovery/finalization binding と、公開前の正確な branch cleanup を記録する corrective Work Item です。 |
| WI-185 — v0.2.23 parity closure | Implemented | `.ai/evidence/WI-185-release-v0-2-23-parity-closure.verification.json`；`.ai/work-items/archive/WI-185-release-v0-2-23-parity-closure.archive.json`；`.ai/decisions/WI-185-release-v0-2-23-parity-closure.finalize.json`；`.ai/decisions/WI-185-release-v0-2-23-parity-closure.close.json`。三言語 parity gate は公開前に WI-184 まで拡張されました。 |
| WI-186 — v0.2.23 post-release public adopter acceptance | Implemented | [Work Item](../work-items/WI-186-release-v0-2-23-post-release-acceptance.ja.md)；[v0.2.23 Release](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.23)；[release workflow](https://github.com/xinglun/ai-cockpit/actions/runs/32629400996)；`.ai/evidence/external/v0.2.23/release-adopter-acceptance/acceptance.json`；`.ai/evidence/external/v0.2.23/adopter/acceptance.json`；`.ai/evidence/external/v0.2.23/upgrade/acceptance.json`；`.ai/evidence/WI-186-release-v0-2-23-post-release-acceptance.verification.json`；`.ai/decisions/WI-186-release-v0-2-23-post-release-acceptance.recovery.json`。不変な公開 ARM64 binary、新しい adopter lifecycle、N-1 upgrade、isolation、evidence reuse、cleanup を Release truth を書き換えずに bind し、WI-187 が immutable recovery linkage を記録します。 |
| WI-187 — finalization-before-archive guard | Implemented | [Work Item](../work-items/WI-187-finalization-before-archive.ja.md)；`.ai/evidence/WI-187-finalization-before-archive.verification.json`；`.ai/work-items/archive/WI-187-finalization-before-archive.archive.json`；`.ai/decisions/WI-187-finalization-before-archive.close.json`。corrective delivery は WI-186 recovery を保持し、fail-closed な finalization-plan ordering guard を追加します。 |
| WI-188 — dynamic governance integrity gate | Implemented | [Work Item](../work-items/WI-188-governance-integrity-gate.ja.md)；`.ai/evidence/WI-188-governance-integrity-gate.verification.json`；`.ai/decisions/WI-188-governance-integrity-gate.finalize.json`；`.ai/decisions/WI-188-governance-integrity-gate.close.json`。active と current release cycle record、evidence、terminal decision、Outcome、parity row、quality gate、Cargo workspace package coverage を固定 WI range なしで検出し、厳密に有効な pre-merge finalize receipt は `awaiting_merge_close` としてのみ報告します。 |
| WI-190 — finalization-plan order recovery | Implemented | `.ai/evidence/WI-190-finalization-plan-order.verification.json`；`.ai/work-items/archive/WI-190-finalization-plan-order.archive.json`；`.ai/decisions/WI-190-finalization-plan-order.finalize.json`；`.ai/decisions/WI-190-finalization-plan-order.close.json`。Runtime-validated blocked receipt は PR #150 の `awaiting_merge_close` を記録しましたが、append-only の merge と cleanup transition は default branch で検証され close されています。 |
| WI-191 — append-only finalization transition chain | Implemented | `.ai/evidence/WI-191-finalization-transition-chain.verification.json`；`.ai/decisions/WI-191-finalization-transition-chain.finalize.json`；`.ai/decisions/WI-191-finalization-transition-chain.close.json`。merge 後の transition と正確な cleanup head は append-only で記録され、default branch で close されます。 |
| WI-191H — finalization receipt head binding | Implemented | `.ai/evidence/WI-191H-finalization-head-binding.verification.json`；`.ai/decisions/WI-191H-finalization-head-binding.finalize.json`；`.ai/decisions/WI-191H-finalization-head-binding.close.json`。receipt commit の head drift は、検証済み merge observation と正確な governance receipt append の場合だけ許可されます。 |
| WI-192 — governance-integrity finalization-order recovery | Implemented | `.ai/decisions/WI-188-governance-integrity-gate.recovery.json`；`.ai/evidence/WI-192-governance-integrity-finalization-order.verification.json`；`.ai/work-items/archive/WI-192-governance-integrity-finalization-order.archive.json`；`.ai/decisions/WI-192-governance-integrity-finalization-order.finalize.json`；`.ai/decisions/WI-192-governance-integrity-finalization-order.close.json`。この clean successor は v0.2.23 compatibility finding を保持し、preflight と verification の前に正確な PR resource context を bind します。pre-merge の blocked receipt は merge 後の close まで `awaiting_merge_close` のままです。 |
| WI-193 — release acceptance isolation（履歴 predecessor） | Recovered | `.ai/decisions/WI-193-release-acceptance-isolation.recovery.json`；`.ai/evidence/WI-193-release-acceptance-isolation.verification.json`；immutable archive `.ai/work-items/archive/WI-193-release-acceptance-isolation.archive.json`。blocked predecessor は赤色/immutable のまま保持し、green completion とは扱いません。 |
| WI-194 — release acceptance isolation recovery（履歴 predecessor） | Recovered | `.ai/decisions/WI-194-release-acceptance-isolation-recovery.recovery.json`；`.ai/evidence/WI-194-release-acceptance-isolation-recovery.verification.json`；immutable archive `.ai/work-items/archive/WI-194-release-acceptance-isolation-recovery.archive.json`。source-built/無効な provider history は変更せず、current delivery は WI-195 で継続します。 |
| WI-195 — governance-integrity recovery gate（履歴 predecessor） | Recovered | [Work Item](../work-items/WI-195-governance-recovery-gate.ja.md)；`.ai/decisions/WI-195-governance-recovery-gate.recovery.json`；`.ai/evidence/WI-195-governance-recovery-gate.verification.json`。finish 後に見つかった parity correction は immutable history として保持し、WI-196 で継続します。 |
| WI-196 — governance-integrity recovery gate retry（履歴 predecessor） | Recovered | [Work Item](../work-items/WI-196-governance-recovery-gate-retry.ja.md)；`.ai/evidence/WI-196-governance-recovery-gate-retry.verification.json`；`.ai/decisions/WI-196-governance-recovery-gate-retry.recovery.json`；immutable history は WI-197 で継続します。 |
| WI-197 — governance gate PR closure（historical predecessor） | Recovered | [Work Item](../work-items/WI-197-governance-gate-pr-closure.ja.md)；`.ai/evidence/WI-197-governance-gate-pr-closure.verification.json`；`.ai/decisions/WI-197-governance-gate-pr-closure.recovery.json`；immutable な pre-merge delivery を保持し、default-branch discovery の修正を WI-198 で継続します。 |
| WI-198 — governance gate default-branch discovery（historical predecessor） | Recovered | [Work Item](../work-items/WI-198-governance-gate-default-branch-discovery.ja.md)；`.ai/evidence/WI-198-governance-gate-default-branch-discovery.verification.json`；`.ai/decisions/WI-198-governance-gate-default-branch-discovery.recovery.json`；immutable な gate 修正を保持し、実際の PR context binding を WI-199 で継続します。 |
| WI-199 — governance gate actual PR context（historical predecessor） | Recovered | [Work Item](../work-items/WI-199-governance-gate-actual-pr-context.ja.md)；`.ai/evidence/WI-199-governance-gate-actual-pr-context.verification.json`；`.ai/decisions/WI-199-governance-gate-actual-pr-context.recovery.json`；immutable な finalization を保持し、GitHub 確認済み head binding を WI-200 で継続します。 |
| WI-200 — governance gate GitHub head binding（historical predecessor） | Recovered | [Work Item](../work-items/WI-200-governance-gate-github-head-binding.ja.md)；`.ai/evidence/WI-200-governance-gate-github-head-binding.verification.json`；`.ai/decisions/WI-200-governance-gate-github-head-binding.finalize.json`；`.ai/decisions/WI-200-governance-gate-github-head-binding.recovery.json`；immutable な pre-merge head binding を保持し、post-merge reconciliation は WI-201 で継続します。 |
| WI-201 — governance gate post-merge reconciliation（historical predecessor） | Recovered | [Work Item](../work-items/WI-201-governance-gate-post-merge-reconciliation.ja.md)；`.ai/evidence/WI-201-governance-gate-post-merge-reconciliation.verification.json`；`.ai/decisions/WI-201-governance-gate-post-merge-reconciliation.finalize.json`；`.ai/decisions/WI-201-governance-gate-post-merge-reconciliation.recovery.json`；immutable な post-merge 記録を保持し、published Runtime の互換性は WI-202 で継続します。 |
| WI-202 — v0.2.24 release and transition compatibility（historical predecessor） | Recovered | [Work Item](../work-items/WI-202-release-v0-2-24.ja.md)；`.ai/evidence/WI-202-release-v0-2-24.verification.json`；`.ai/decisions/WI-202-release-v0-2-24.finalize.json`；`.ai/decisions/WI-202-release-v0-2-24.recovery.json`；immutable な v0.2.24 release preparation bytes と公開前 gate failure を保持し、予約済み v0.2.24 tag は再利用しません。 |
| WI-203 — v0.2.25 release and transition compatibility（historical predecessor） | Recovered | [Work Item](../work-items/WI-203-release-v0-2-25.ja.md)；`.ai/evidence/WI-203-release-v0-2-25.verification.json`；`.ai/decisions/WI-203-release-v0-2-25.recovery.json`；不変の archive と公開前の blocked Outcome を保持し、必要な resource context binding は WI-204 で継続します。 |
| WI-204 — v0.2.25 release and transition compatibility（historical predecessor） | Recovered | [Work Item](../work-items/WI-204-release-v0-2-25.ja.md)；`.ai/evidence/WI-204-release-v0-2-25.verification.json`；`.ai/decisions/WI-204-release-v0-2-25.recovery.json`；不変の archive と誤った base の finalization attempt を保持し、正しい default-branch baseline は WI-205 で継続します。 |
| WI-205 — v0.2.25 release and transition compatibility recovery（historical predecessor） | Recovered | [Work Item](../work-items/WI-205-release-v0-2-25.ja.md)；`.ai/evidence/WI-205-release-v0-2-25.verification.json`；`.ai/decisions/WI-205-release-v0-2-25.finalize.json`；immutable な v0.2.25 公開失敗 history を保持し、release boundary は WI-206→WI-210 で継続します。 |
| WI-206 — release tag の pending-close governance boundary（recovered） | Implemented | [Work Item](../work-items/WI-206-release-tag-pending-close.ja.md)；`.ai/evidence/WI-206-release-tag-pending-close.verification.json`；`.ai/decisions/WI-206-release-tag-pending-close.recovery.json`；Release tag の ancestor proof と通常 branch の fail-closed regression は不変に保持し、missing finalization order は WI-208 で継続します。 |
| WI-207 — release tag finalization order（recovered） | Implemented | [Work Item](../work-items/WI-207-release-tag-pending-close-finalization.ja.md)；`.ai/evidence/WI-207-release-tag-pending-close-finalization.verification.json`；`.ai/decisions/WI-207-release-tag-pending-close-finalization.recovery.json`；verify-before-finalize の順序違反を不変の history として保持します。 |
| WI-208 — release tag finalization order（recovered） | Implemented | [Work Item](../work-items/WI-208-release-tag-pending-close-finalization.ja.md)；`.ai/evidence/WI-208-release-tag-pending-close-finalization.verification.json`；`.ai/decisions/WI-208-release-tag-pending-close-finalization.recovery.json`；誤った PR base identity は不変の history として保持し、WI-209 で修正します。 |
| WI-209 — release tag finalization order | Implemented | [Work Item](../work-items/WI-209-release-tag-pending-close-finalization.ja.md)；`.ai/evidence/WI-209-release-tag-pending-close-finalization.verification.json`；`.ai/decisions/WI-209-release-tag-pending-close-finalization.finalize.json`；PR #158 を同期済み default-branch base に verification 前に bind し、`53211dc` として merge 済みです。public finalize/close は WI-210 で完了します。 |
| WI-210 — v0.2.26 immutable release と adopter acceptance（公開失敗 history） | Recovered | [Work Item](../work-items/WI-210-release-v0-2-26.ja.md)；`.ai/evidence/WI-210-release-v0-2-26.verification.json`；`.ai/decisions/WI-210-release-v0-2-26.finalize.json`；immutable tag と source-quality failure を保持し、public Release はありません。release boundary は v0.2.27 で継続します。 |
| WI-211 — governance fixture の event context 分離（recovered） | Recovered | [Work Item](../work-items/WI-211-hermetic-governance-fixture.ja.md)；`.ai/evidence/WI-211-hermetic-governance-fixture.verification.json`；`.ai/decisions/WI-211-hermetic-governance-fixture.recovery.json`；immutable な source-quality 修正を保持し、finalization boundary は WI-212 で継続します。 |
| WI-212 — WI-211 finalization recovery | Implemented | [Work Item](../work-items/WI-212-release-fixture-finalization-recovery.ja.md)；`.ai/evidence/WI-212-release-fixture-finalization-recovery.verification.json`；`.ai/decisions/WI-211-hermetic-governance-fixture.recovery.json`；`.ai/decisions/WI-212-release-fixture-finalization-recovery.finalize.json`；`.ai/decisions/WI-212-release-fixture-finalization-recovery.finalize.13201203b2bc4c5ad0c2185d97c549e2a3901f23584186eb3a682a974cb65405.json`；`.ai/decisions/WI-212-release-fixture-finalization-recovery.finalize.c57751b722400115553edc6d1c66c4452335c06e0a736c54c1a344e1c0ca4818.json`；`.ai/decisions/WI-212-release-fixture-finalization-recovery.close.json`；PR #160 は `b5b521e` として merge 済みで、正確な remote branch/worktree cleanup を検証しました。historical evidence は不変のまま、current Runtime による再検証までは historical として投影されます。 |
| WI-213 — v0.2.27 immutable release と adopter acceptance | Implemented | [Work Item](../work-items/WI-213-release-v0-2-27.ja.md)；`.ai/evidence/WI-213-release-v0-2-27.verification.json`；`.ai/decisions/WI-213-release-v0-2-27.finalize.json`；`.ai/decisions/WI-213-release-v0-2-27.finalize.36a0e04421064517281e64709c4fd103f75758c78a523f83a82e1af88f8fbe44.json`；`.ai/decisions/WI-213-release-v0-2-27.finalize.708014f642b04450d86e517d87d05ff773ca996fc741d5ef325c8206ed734d7f.json`；`.ai/decisions/WI-213-release-v0-2-27.close.json`；`.ai/evidence/external/v0.2.27/adopter-aarch64-apple-darwin/`；`.ai/evidence/external/v0.2.27/upgrade-v0.2.23-to-v0.2.27/`；PR #161 は `baf78f9` として merge 済みで、public Release [v0.2.27](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.27) と workflow [32657788976](https://github.com/xinglun/ai-cockpit/actions/runs/32657788976) は成功しました。public binary の adopter lifecycle と v0.2.23→v0.2.27 upgrade acceptance は cleanup receipt を含めて成功しています。 |
| WI-159 — Runtime resource finalization integration | Implemented | `.ai/evidence/WI-159-resource-finalization-runtime.verification.json`; `.ai/decisions/WI-159-resource-finalization-runtime.close.json`; finalization receipt history は `.ai/evidence/external/WI-159-finalization/`。 |
| WI-160 — Resource finalization と branch/worktree closure の baseline | Implemented | [Work Item](../work-items/WI-160-resource-finalization-baseline.ja.md); `.ai/evidence/WI-160-resource-finalization-baseline.verification.json`; `.ai/work-items/archive/WI-160-resource-finalization-baseline.archive.json`; `.ai/decisions/WI-160-resource-finalization-baseline.close.json`。Runtime command/receipt 統合は WI-159、historical-runtime close compatibility は WI-161 で実装する。 |
| WI-161 — Historical Runtime evidence close compatibility | Implemented | [Work Item](../work-items/WI-161-historical-runtime-close.ja.md); archived evidence は不変のまま foreign Runtime bytes を historical として投影する。Regression evidence: `.ai/evidence/WI-161-historical-runtime-close.verification.json` |
| WI-162 — Historical snapshot compatibility after archive | Implemented | `.ai/evidence/WI-162-historical-snapshot-compat.verification.json`; archived plan receipt は記録時 snapshot に bind され、history は書き換えません。 |
| WI-163 — Historical Outcome projection | Implemented | `.ai/evidence/WI-163-historical-outcome-projection.verification.json`; historical evidence を current verification failure として表示しません。 |
| WI-164 — Historical Outcome human rendering | Implemented | `.ai/evidence/WI-164-historical-outcome-render.verification.json`; tri-language handoff は historical evidence の missing-evidence recovery wording を抑止します。 |

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

WI-191 は append-only finalization transition chain を追加します。不変の canonical blocked evidence は履歴を書き換えずに merge observation と exact cleanup を経て進み、verification と close は一意な最新 head を束縛します。WI-191H は receipt commit の head binding を閉じます。最初の merge observation で governance receipt append を明示し、Git が証明した場合に限り、観測済みの `70c17e4` archive head を `8f5a025` に進められます。任意の head drift は引き続き拒否されます。
