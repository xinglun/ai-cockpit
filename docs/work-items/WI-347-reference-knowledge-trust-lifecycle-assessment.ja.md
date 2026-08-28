---
author: AI Cockpit maintainers
title: "WI-347 — Knowledge、input trust、installed lifecycle、Japanese capability assessment"
workItemId: WI-347-reference-knowledge-trust-lifecycle-assessment
description: "次の 10 個の pinned reference path を比較し、bounded な Rust-native 三言語 mapping を公開します。"
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
capabilityClaims:
  - reference_parity
---

# WI-347 — Knowledge、input trust、installed lifecycle、Japanese capability assessment

[English](WI-347-reference-knowledge-trust-lifecycle-assessment.md) · [简体中文](WI-347-reference-knowledge-trust-lifecycle-assessment.zh-CN.md)

## Intent と boundary

この Work Item は pinned reference commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の 10 path を比較し、implementation Knowledge、input provenance、installed Runtime lifecycle、instruction traceability、human-report semantic、bounded Japanese capability assessment の Rust-native mapping を adopter 向けに追加します。

Target は一つの shared external Runtime と明示的な `--repo` repository context を維持します。Source Python/Make/YAML orchestration、generated assessment bytes、provider-global configuration、source JSON wire compatibility は対象外です。意図的な差分は command/field の同一性を意味しません。

## File-by-file decision

| Pinned reference path | Classification | Bounded target decision |
| --- | --- | --- |
| `docs/reference/human-report-semantic-quality.md` | `implemented-different-by-design` | Decision view の順序と forbidden-claim boundary を human-benefit、task-outcome、Outcome page に対応づけます。 |
| `docs/reference/implementation-knowledge.ja.md` | `implemented-different-by-design` | Typed な read-only Knowledge projection の日本語 reader route を提供します。 |
| `docs/reference/implementation-knowledge.md` | `implemented-different-by-design` | 現在の deterministic CLI/MCP filter を示し、date/commit/supersession dimension が未実装である境界を明記します。 |
| `docs/reference/implementation-knowledge.zh-CN.md` | `implemented-different-by-design` | 同じ filter と evidence boundary の中国語 route を提供します。 |
| `docs/reference/input-trust-dataflow.ja.md` | `implemented-different-by-design` | Provenance guidance を typed Rust origin と traceable derivation に対応づけます。 |
| `docs/reference/input-trust-dataflow.md` | `implemented-different-by-design` | Content/tool output の分類、cross-step preservation、fail-closed injection handling を説明します。 |
| `docs/reference/input-trust-dataflow.zh-CN.md` | `implemented-different-by-design` | 中国語 route と authentication を行わない境界を追加します。 |
| `docs/reference/installed-lifecycle.md` | `implemented-different-by-design` | Shared install、explicit attach、immutable Release acceptance、migration/rollback ownership を mapping します。 |
| `docs/reference/instruction-traceability.md` | `implemented-different-by-design` | Inventory、Work Item evidence、close chain を source の forward/reverse traceability に対応づけます。 |
| `docs/reference/japanese-capability-assessment.json` | `implemented-different-by-design` | 三言語 page と executable presentation/adversarial check に対応づけ、source bytes を import せず general fluency も主張しません。 |

10 行すべてを machine inventory と三言語 comparison ledger に登録します。Adopter boundary も acceptance の一部です。Runtime binary は共有しますが、各 repository の fact、Knowledge、evidence、adapter record、decision は隔離します。

## Acceptance と verification

- 各 pinned path は一度だけ inventory に現れ、上記 classification/reason を持ち、この batch に `deferred-next-batch`/`migrate-gap` を残しません。
- 5 つの新しい reference page は English、Chinese、Japanese link を持ち、semantic/non-wire boundary を記載します。
- Knowledge は未対応の date/commit/supersession filter を宣伝せず、input trust は content を identity/authorization とせず、installation は Runtime install と attach/migration を混同せず、Japanese page は general fluency を主張しません。
- Inventory、documentation metadata/link、governance integrity、comparison/parity check が成功し、source Python/Make/V1 file と global Agent/MCP configuration は追加しません。
- 明示的 repository context で installed Runtime lifecycle を実行します：checkpoint → verify → finish → archive → reviewed PR/merge → close。Visible human Outcome と正確な branch/worktree cleanup を含みます。

Pinned reference commit: `e5acb677da6621004d96f0ef353c58fe8d3acfbf`。
Target base commit: `6ddd41d85b972a663fee85562592fc247749bf49`。
