---
author: AI Cockpit maintainers
title: "Reference file comparison"
description: "固定した baseline で reference source を file 単位に比較する方法。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - reference_parity
---

# Reference file comparison

このページは Rust project と公開 reference source を file ごとに比較する方法を説明します。
Reference は specification と behavior corpus であり、Rust Runtime にコピーする directory ではありません。

## 固定 baseline

- Reference: [spirex-ds-dev/ai-cockpit-template](https://github.com/spirex-ds-dev/ai-cockpit-template)、commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`。
- Rust baseline: [xinglun/ai-cockpit](https://github.com/xinglun/ai-cockpit) の `origin/main`、commit `b159deb4b1976befb0d1cc547c99c40a3bc3b13c`。
- 比較に使う Runtime: `ai-cockpit 0.2.31`、binary SHA256 `1064f61154168149aebb63a4ad15374d50fc729c8699142c7a193c22eb6fb8f9`。

古い documentation fixture との互換性のため、歴史的なマーカー
`487f01970c49e2b85d17b0cb0536f9d60c8f05e0` と `689` を残します。これらは現在の比較基線や延期数ではありません。
現在の基線は `b159deb4b1976befb0d1cc547c99c40a3bc3b13c`、現在の延期数は `687` です。

Machine-readable ledger は
[`reference_file_inventory.json`](../../tests/conformance/reference_file_inventory.json) です。
Regression check は tracked reference path のすべてに一つだけ classification があることを確認し、
first batch の未分類 file を拒否します。Target checkout metadata は dirty/untracked な
working-tree file ではなく pinned commit から導出します。

## Classification

- **implemented-equivalent** — 同じ reader/governance responsibility が同じ boundary で存在する。
- **implemented-different-by-design** — responsibility はあるが、Rust Protocol、shared external Runtime、
  または explicit Agent adapter が別の path/abstraction で担当する。
- **migrate-gap** — accepted counterpart がなく、bounded remediation が必要。
- **not-applicable** — 現在の Runtime product boundary の外。
- **reference-only** — 説明または conformance material としてのみ保持する。
- **generated-history** — immutable history または generated projection。コピーも静かな書き換えもしない。
- **deferred-next-batch** — 登録済みだが semantic comparison は後続 batch。parity や omission を意味しない。

## First batch: governance entrypoints

First batch は root Agent rules、`.ai` entrypoint と terminology、reader-facing README/architecture route、
reference governance configuration entrypoint を対象にします。Rust project は重要な boundary を維持しますが、
reference の Python Runtime、Makefile target、YAML guard tree、provider-global rules、generated history はコピーしません。

| Reference surface | Rust result | Boundary |
| --- | --- | --- |
| `AGENTS.md`、`CLAUDE.md`、`GEMINI.md`、Cursor rule | 意図した別実装 | Attached adapter と explicit provider install を使います。Shared Runtime は外部にあり、比較による provider-global config 注入はありません。 |
| `.ai/README.md`、glossary、cockpit workflow/adoption guide | 意図した別実装 | `.ai/README.md`、`.ai/glossary.md`、`docs/reference/agent-workflow.*`、getting-started route が Rust request-scoped Runtime workflow を担います。 |
| Reference guard、policy、quality、trust schema | 意図した別実装 | Typed Rust Protocol/Runtime service、repository test、CI manifest、reference docs が対応します。source YAML/JSON はコピーしません。 |
| Root と documentation README route | 意図した別実装 | 三言語 route は相互リンクし、shared Runtime と repository context isolation を説明します。 |
| `SECURITY.md` | 等価（Rust boundary を追加） | Security policy entrypoint を維持し、Runtime deployment/patch boundary を追加します。 |
| `CONTRIBUTING.md` | この batch で補完 | Explicit `--repo` lifecycle、fail-closed evidence、visible Outcome、reviewed PR、merge 後の exact cleanup を説明します。 |
| Reference の generated Work Item、decision、evidence、audit、release history | Generated history | これらの bytes は reference history として保持し、Rust repository にはコピーしません。 |

従って first batch で見つかった唯一の concrete entrypoint gap（`CONTRIBUTING.md`）は補完しました。
Second governance system は作らず、残りは ledger に明示して後続の semantic batch に送ります。

## WI-270：Contract semantic file-by-file batch

WI-270 は次の 27 reference path を一つずつ確認しました。ledger はすべてを
`implemented-different-by-design` と分類しています。責任は Rust Runtime または
repository-bound の docs/test に存在しますが、Python module、Make target、generated file、
provider-global path はコピーしません。Counterpart は evidence index であり、byte-level
identity の主張ではありません。

| Reference path | Classification | Rust counterpart / boundary |
| --- | --- | --- |
| `docs/concepts/decision-states.ja.md` | 意図した別実装 | Japanese Contract/Outcome docs と typed decision test |
| `docs/concepts/decision-states.md` | 意図した別実装 | Contract/Outcome docs と typed decision test |
| `docs/concepts/decision-states.zh-CN.md` | 意図した別実装 | Chinese Contract/Outcome docs と typed decision test |
| `docs/features/work-item-parallelism.ja.md` | 意図した別実装 | WI-123、Japanese configuration route、boundary/lease test |
| `docs/features/work-item-parallelism.md` | 意図した別実装 | WI-123、configuration route、boundary/lease test |
| `docs/features/work-item-parallelism.zh-CN.md` | 意図した別実装 | WI-123、Chinese configuration route、boundary/lease test |
| `docs/reference/safe-parallel-verification.md` | 意図した別実装 | Rust bounded executor、`verify --workers`、argv/evidence test |
| `docs/reference/work-item-intelligence-interface.md` | 意図した別実装 | request-scoped status/intelligence は実装済み；cost/wait/index-version aggregate は後続 boundary |
| `docs/reference/work-item-state-machine.md` | 意図した別実装 | typed lifecycle/recovery/finalization；provider PR state は external evidence |
| `docs/reference/work-item-status-interface.md` | 意図した別実装 | Rust status/Outcome projection と test が generated Python status を置換 |
| `scripts/ai_acceptance_policy.py` | 意図した別実装 | `governance_controls.rs` の acceptance ID/evidence validation |
| `scripts/ai_check_scenario_coverage.py` | 意図した別実装 | Runtime scenario coverage と Contract/Summary binding |
| `scripts/ai_check_work_item.py` | 意図した別実装 | typed Contract scope、authority、unknown、execution、concurrency、lifecycle validation |
| `scripts/ai_decision_protocol.py` | 意図した別実装 | repository-bound typed preflight decision receipt |
| `scripts/ai_intent_policy.py` | 意図した別実装 | Runtime intent alignment と intent/scenario binding |
| `scripts/ai_parallel_verification.py` | 意図した別実装 | Rust bounded execution、worker cap、deterministic result、scope safety |
| `scripts/ai_preflight_review.py` | 意図した別実装 | typed preflight state、humanDecisionRequest、confirmation、recovery condition |
| `scripts/ai_scenario_policy.py` | 意図した別実装 | risk-sensitive scenario policy と fail-closed unknown |
| `scripts/ai_work_item_state.py` | 意図した別実装 | Rust lifecycle state machine と recovery receipt |
| `tests/test_acceptance_policy.py` | 意図した別実装 | Rust Contract schema/preflight regression |
| `tests/test_ai_parallel_verification.py` | 意図した別実装 | Rust CLI/executor verification regression |
| `tests/test_checkpoint_intent.py` | 意図した別実装 | Rust preflight/checkpoint intent regression |
| `tests/test_contract_and_policy.py` | 意図した別実装 | Rust strict Contract/policy regression |
| `tests/test_intent_policy.py` | 意図した別実装 | Rust intent alignment regression |
| `tests/test_parallel_lifecycle_contract.py` | 意図した別実装 | Rust parallel boundary、lease、lifecycle、isolation regression |
| `tests/test_preflight_review.py` | 意図した別実装 | Rust preflight/review regression |
| `tests/test_scenario_coverage_gate.py` | 意図した別実装 | Rust required-scenario と invalid-status regression |

この slice では未記録の Contract semantic implementation gap は見つかりませんでした。
Intelligence interface は意図的に bounded です。request-scoped status と evidence-derived
Outcome は実装済みですが、reference の広い aggregate/cost/wait dimension は後続 batch であり、
complete parity とは扱いません。

## 現在の ledger snapshot

固定した v0.2.31 comparison baseline の ledger は 5,119 records です。内訳は
4,262 `generated-history`、169 `implemented-different-by-design`、1
`implemented-equivalent`、687 `deferred-next-batch` です。Deferred record は予定された
比較であり parity claim ではありません。capability/profile slice に `migrate-gap` は残っていません。

1. `.ai/project/adopter-capability-manifest.json` は Runtime registry で表現し、installer-surface は external boundary とします。
2. `.ai/project/capabilities.json` は strict な Rust-native declaration と明示的な operation mapping で表現します。
3. `.ai/project/success_criteria.json` は authority を持たない snapshot-bound visibility projection です。
4. `.ai/project_profile.yaml` は `.ai/project.json` と strict JSON `profile-policy.json` projection で表現します。

Governance entrypoint、getting-started route、CI/release boundary、capability/profile
projection はこの baseline で review 済みです。上記 4 件は bounded な Rust-native counterpart として登録済みで、
687 deferred semantic comparison は後続作業として残ります。

WI-274 は target checkout metadata と canonical comparison snapshot だけを、レビュー済み
default branch commit に再バインドします。WI-273 は immutable な failed-delivery record として
保持します。最初の commit では parity registration が verification evidence より先だったことを
証明できないため、successor はその履歴を書き換えずに分離して redelivery します。

## Batch order

後続 batch は次の順序で比較し、必要な差分だけを実装します。

1. Contract field、intent、scenario/acceptance dimension、parallel slot、preflight review。
2. CI quality routing、dynamic verification tier、evidence assurance。
3. Runtime lifecycle、Outcome/MCP projection、recovery、knowledge、repository isolation。
4. Conformance、adversarial case、performance、release、adopter acceptance。

各 batch は独立した Contract と evidence を持ちます。review と publish 後、次の batch は published Runtime で
再度 acceptance を実施し、working-tree code を release behavior と取り違えないようにします。

## WI-286 file-level Agent Risk と checkpoint batch

WI-286 は reference の Agent Risk/checkpoint responsibility を一つずつ比較します。
Source の Python/YAML は reference corpus のままとし、Rust の typed Protocol
record と共有 lifecycle validator で bounded semantics を強制します。

| Reference path | Classification | Rust counterpart |
| --- | --- | --- |
| `.ai/guards/agent_risk_policy.yaml` | implemented-different-by-design | typed `checkpointPolicy`、Contract verification declaration、Agent Risk validator、dynamic profile docs。 |
| `scripts/ai_check_agent_risk.py` | implemented-different-by-design | `validate_agent_risk_controls` を lifecycle boundary で共有。 |
| `scripts/ai_checkpoint.py` | implemented-different-by-design | typed `CheckpointEvidence`、amendment CLI、append-only chain、resume-stale binding。 |
| `tests/test_ai_agent_risk.py`、`tests/test_ai_checkpoint.py`、`tests/test_outcome_lifecycle_rules.py` | implemented-different-by-design | Rust protocol/repository lifecycle と static Agent-rule parity test。 |

これは semantic parity であり、直接の JSON-wire parity ではありません。WI-291 は
read-only Rust Contract-aware CI gate を追加し、収束期間中は Python route/manifest を
shadow として残します。完全な workflow と release-preflight parity は deferred のままです。

## WI-287 checkpoint conformance の収束

WI-287 は checkpoint の実装と test source file に残っていた deferred ledger
record を閉じます。Rust は verification 開始後の `before_edit` checkpoint と、
不正な最新 resume timestamp を明示的に拒否します。Reference test の意味は Rust
native lifecycle regression で表現し、Python test や source wire shape は copy
しません。Static Agent-rule test は project rules に同じ terminality と narrow
successor boundary があることを確認します。

Object/adopter boundary は変わりません。shared Runtime は request-scoped、全操作は
明示的な `--repo` を持ち、human Outcome が visible handoff です。CI workflow convergence
と広い adopter surface は別の bounded batch です。

## WI-291 CI Contract-aware quality gate

WI-291 は reference workflow の quality routing と preflight boundary を Rust-native
CI surface と比較します。Python route は `light`/`standard`/`strict` の dynamic planner、
canonical manifest は command list として継続します。standard/strict の Pull Request
command 実行前に、Rust CLI の read-only `gate` が active Contract、repository/base/
snapshot identity、intent/scenario/operation/stage route、Agent-Risk/preflight projection
を検証します。identity-bound な `repository_contract_quality_gate` receipt を出力し、
yellow/red は fail-closed で CI を止めます。gate は `.ai/` record を書き込みません。

この batch は semantic parity であり、source YAML や Python wire の copy ではありません。
CI source-build Runtime identity は診断用で、immutable Release/adopter identity は published
artifact acceptance の境界です。残りの workflow matrix、gate metadata/timeout、release
preflight、多技術 stack adopter は ledger で deferred として扱い、実装済みとは主張しません。
