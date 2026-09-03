---
author: AI Cockpit maintainers
title: "WI-539 — source governance checker comparison batch 36"
description: "固定した reference governance checker 10 file を一つずつ比較し、Rust-native または external boundary を記録します。"
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: canonical
workItemId: WI-539-reference-file-comparison-batch-36
lastVerifiedBy: WI-539-reference-file-comparison-batch-36
---

# WI-539 — source governance checker comparison batch 36

## Objective

pinned source commit `fde3380f81fea5fd2e288f7a8849f737dc074060` の維持対象 checker 10 file を一つずつ読み、各 current path の evidence-backed semantic classification を記録します。目的は parity と adopter inheritance boundary であり、Python、Make、YAML、source JSON wire format を shared Rust Runtime に copy することではありません。

## File-level result

| Reference path | Decision | Rust boundary |
| --- | --- | --- |
| `scripts/ai_check_guidelines.py` | `implemented-different-by-design` | typed Contract guidelines は human-owned のまま、番号付き acceptance/evidence binding で completion を証明します。untyped `guidelinesCompliance` claim は推論しません。 |
| `scripts/ai_check_pr.py` | `implemented-different-by-design` | archive、recovery、scope、evidence は typed lifecycle gate に分散して検証し、PR identity と hosted check は provider evidence のままです。 |
| `scripts/ai_check_reference_impact.py` | `reference-only` | static AST/text impact scan は source/provider tooling として保持します。Rust の operation-time scope check は fail-closed ですが、caller、external consumer、monitoring は推論しません。 |
| `scripts/ai_check_registry.py` | `implemented-different-by-design` | versioned gate manifest と typed receipt が deterministic な登録、deduplication、unavailable-gate reason を担います。 |
| `scripts/ai_check_review_policy.py` | `implemented-different-by-design` | Contract/preflight と provider PR review が authority を担い、第二の YAML policy や report-only focus list は導入しません。 |
| `scripts/ai_check_scope.py` | `implemented-different-by-design` | repository-relative scope/out-of-scope、dependency、parallel boundary、snapshot check は typed Runtime gate です。 |
| `scripts/ai_check_serial_order.py` | `implemented-different-by-design` | predecessor、merged PR、closure、exact resource cleanup、synchronized base は lifecycle と ready-on-base が検証します。 |
| `scripts/ai_check_status.py` | `implemented-different-by-design` | request-scoped typed status と human Outcome projection が generated `current_status.md` に代わる authority です。 |
| `scripts/ai_check_status_consistency.py` | `implemented-different-by-design` | read-only status は active/archive ownership を導出して ambiguity を拒否し、Runtime は generated status を silent repair しません。 |
| `scripts/ai_check_summary.py` | `implemented-different-by-design` | strict Contract、evidence、archive、Outcome binding が portable boundary を担いますが、source Summary JSON compatibility や human claim の推論は行いません。 |

## Findings and adopter inheritance

この slice に portable implementation omission はありません。reference-impact scanner は明示的に `reference-only` であり、Rust の hidden gap ではありません。caller と external consumer の static fact は adopter/provider または human-owned evidence が供給し、unknown impact は fail-closed のままです。残り 9 件は typed Protocol、repository lifecycle、gate manifest、status、Outcome boundary で表現されます。

各 attached object/adopter project は shared Runtime、明示的な `--repo` binding、isolated Contract/evidence/knowledge、fail-closed lifecycle、人間向け Outcome presentation を継承します。source checker、provider policy value、stack-specific command は継承せず、source と target の JSON wire shape も独立しています。

## Acceptance

- inventory は pinned source commit のこの 10 current path を正確に記録し、各行に non-empty reason と counterpart または明示的 boundary がある。
- selected path に `deferred-next-batch` / `migrate-gap` を残さず、retired history は append-only に保持する。
- English、Simplified Chinese、Japanese の comparison page と本 Work Item page が同じ decision と adopter boundary を示す。
- inventory、documentation、format、lint、workspace verification が Work Item Finish 前に pass する。
