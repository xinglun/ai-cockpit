---
author: AI Cockpit maintainers
title: "WI-345 — governance cost / performance documentation batch 15"
workItemId: WI-345-reference-governance-cost-batch-15
description: "5 つの pinned governance cost/complexity/performance document を比較し、source tooling を発明せず Rust の bounded counterpart を記録します。"
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
terminalArchive: .ai/work-items/archive/WI-345-reference-governance-cost-batch-15.contract.json
terminalVerification: .ai/evidence/WI-345-reference-governance-cost-batch-15.verification.json
terminalFinalization: .ai/decisions/WI-345-reference-governance-cost-batch-15.finalize.json
terminalDecision: .ai/decisions/WI-345-reference-governance-cost-batch-15.close.json
capabilityClaims:
  - reference_parity
---

# WI-345 — governance cost / performance documentation batch 15

## Intent と boundary

この Work Item は pinned reference の 5 document を一つずつ比較します。governance complexity（English/Japanese）、governance cost metrics、governance performance budgets、profile/cost separation について、adopter が継承できる governance boundary を保ちつつ、source Python/Make maintenance tooling、timing evidence、cost を authority として持ち込みません。

範囲は inventory、tri-language comparison/parity page、新しい reader-facing reference page、本 Work Item record に限定します。Runtime code、source script/guard file、global Agent/MCP configuration、immutable history、hard performance target は対象外です。

## File-by-file decision

| Pinned reference path | Classification | Bounded target decision |
| --- | --- | --- |
| `docs/reference/governance-complexity.ja.md` | `reference-only` | target page は boundary と immutable archive/integrity rule を記録しますが、source Python/Make scanner、threshold、同等 metric は主張しません。 |
| `docs/reference/governance-complexity.md` | `reference-only` | `inspect`、`status`、`doctor`、repository integrity gate が target fact を提供し、source complexity report は portable でない maintenance material とします。 |
| `docs/reference/governance-cost-metrics.md` | `implemented-different-by-design` | `diagnose` と typed verification cost estimate/observation が identity-bound advisory fact を提供します。Source JSONL phase/wait aggregation と report wire shape は Rust requirement ではありません。 |
| `docs/reference/governance-performance-budget.md` | `implemented-different-by-design` | Identity-bound `PerformanceBaseline` sample と explicit regression budget が invalid/regressed measurement を拒否し、required verification を省略せず P95/profile authority を作りません。 |
| `docs/reference/governance-profile-cost-separation.md` | `implemented-different-by-design` | light/standard/strict route、operation/stage escalation、`VerificationTier`、`EvidenceAssurance`、advisory cost を直交させます。 |

これは semantic/documentation parity であり、source command や JSON-wire compatibility ではありません。Object/adopter boundary は shared Runtime、明示的 `--repo`、repository-local evidence、policy-owned route requirement、global current project を作らないことです。

## Acceptance と verification

- 5 path は inventory に各 1 回だけ存在し、上記 classification で deferred/migrate-gap はありません。
- English、Simplified Chinese、Japanese の reference/parity page が同じ decision と current count を示します。
- Reader-facing page は利用できない source detail を明示し、CLI command、profile decision、metric、assurance を発明しません。
- Cost/performance output は advisory と明記し、timing が `VerificationTier`、`EvidenceAssurance`、policy、protected check の代替にならないことを確認します。
- inventory、documentation、governance、format、lint、locked workspace verification が成功します。

Pinned source commit は `e5acb677da6621004d96f0ef353c58fe8d3acfbf`、target base は `747cf3d9f846aac52b2a592ec61a874511c18b81` です。

[English](WI-345-reference-governance-cost-batch-15.md) ·
[简体中文](WI-345-reference-governance-cost-batch-15.zh-CN.md)
