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
- Rust baseline: [xinglun/ai-cockpit](https://github.com/xinglun/ai-cockpit) の `origin/main`、commit `bc8b7e56a98d105cd9f00b3b7300dc8eb0396c7b`。
- 比較に使う Runtime: `ai-cockpit 0.2.33`、binary SHA256 `eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4`。

このページは現在固定した比較 baseline だけを説明します。歴史的な delivery detail は
Work Item archive evidence に保持し、reader-facing route には載せません。

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

<!-- reference-inventory-counts: total=5119 generated-history=4262 implemented-different-by-design=324 implemented-equivalent=1 not-applicable=4 reference-only=47 deferred-next-batch=481 migrate-gap=0 -->

固定した reference comparison baseline の ledger は 5,119 records です。内訳は
4,262 `generated-history`、316 `implemented-different-by-design`、1
`implemented-equivalent`、4 `not-applicable`、47 `reference-only`、489 `deferred-next-batch` です。
Deferred record は予定された比較であり parity claim ではありません。
capability/profile slice に `migrate-gap` は残っていません。

1. `.ai/project/adopter-capability-manifest.json` は Runtime registry で表現し、installer-surface は external boundary とします。
2. `.ai/project/capabilities.json` は strict な Rust-native declaration と明示的な operation mapping で表現します。
3. `.ai/project/success_criteria.json` は authority を持たない snapshot-bound visibility projection です。
4. `.ai/project_profile.yaml` は `.ai/project.json` と strict JSON `profile-policy.json` projection で表現します。

Governance entrypoint、getting-started route、CI/release boundary、capability/profile
projection はこの baseline で review 済みです。上記 4 件は bounded な Rust-native counterpart として登録済みで、
495 deferred semantic comparison は後続作業として残ります。

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

## WI-302 最初の deferred file batch

WI-302 は lexical order の最初の 10 deferred path を pinned source commit と一つずつ比較しました。
8 records は evidence-backed な結論になりました。WI-304 は続けて reference の広い
Python/multi-stack matrix を含む 2 workflow record を比較し、Rust-native の分割と
adopter/external boundary を記録しました。

| Reference path | Classification | Rust counterpart / boundary |
| --- | --- | --- |
| `.ai/cockpit/bandit_low_risk_baseline.json` | not-applicable | Reference Python tooling の生成 Bandit baseline であり、Rust/Bandit の product surface はありません。 |
| `.gitattributes` | implemented-different-by-design | Rust の source-archive boundary と `tests/release/source_archive_policy_test.sh` が governance/build root を除外し Cargo source を保持します。 |
| `.github/CODEOWNERS` | not-applicable | 個人 owner は portable ではありません。Adopter の review owner は external repository/provider の判断です。 |
| `.github/dependabot.yml` | not-applicable | pip/Actions の更新は optional な provider automation です。Rust の dependency facts は `Cargo.toml`/`Cargo.lock` と action pin policy が持ちます。 |
| `.github/workflows/compatibility.yml` | implemented-different-by-design | WI-304 は ShellCheck、lockfile、Python、real/extended/mobile matrix、non-blocking latest probe を比較しました。Rust `ci.yml`、dynamic quality route、canonical gate、public adopter acceptance が Rust product を担当し、source installer/Python/multi-stack coverage は adopter/external boundary です。 |
| `.github/workflows/release.yml` | implemented-different-by-design | Rust release workflow と release tests が target archive、checksum、SBOM/provenance、platform smoke、public/N-1 adopter acceptance を提供します。 |
| `.github/workflows/smoke.yml` | implemented-different-by-design | WI-304 は全 source shard、dispatch input、artifact、dependency edge、release/measurement condition、installer check を比較しました。Rust `ci.yml`、`release.yml`、gate manifest、immutable adopter harness が分割して担当し、source Python/Make/install smoke は external/adopter-owned です。 |
| `.gitignore` | implemented-different-by-design | Rust/Cargo build と governance review path を ignore し、source-archive policy を regression test します。 |
| `LICENSE` | implemented-different-by-design | 両方 MIT です。Copyright と Rust packaging は target 定義であり source の本文は copy しません。 |
| `Makefile` | implemented-different-by-design | Rust CLI、Cargo、明示的な CI/release script が Python Make orchestration を置き換え、request-scoped `--repo` を保ちます。 |

WI-302/WI-304 batch 完了時の ledger snapshot は 4,262
`generated-history`、190 `implemented-different-by-design`、1 `implemented-equivalent`、
3 `not-applicable`、3 `reference-only`、660 `deferred-next-batch` です。2 workflow record は
Rust-native の意図した別実装 boundary として close しましたが、source の Python installer
や multi-stack matrix が Rust Runtime 内で実行されるとは主張しません。

## WI-304 workflow comparison

WI-304 は pinned source commit の `.github/workflows/compatibility.yml` と
`.github/workflows/smoke.yml` を比較し、trigger、permission、concurrency、全 job/matrix、
`needs` edge、dispatch input、artifact upload/download、blocking/non-blocking condition、
release/measurement branch、installer check を確認しました。

`compatibility.yml` の責任は、source `install.sh` の ShellCheck、pinned Python platform と
lockfile reproducibility、real/extended/mobile stack quality matrix、non-blocking latest
ecosystem probe、および blocking/latest aggregate gate の 8 種類です。Rust は意図的に
分割し、`ci.yml` の dynamic `light`/`standard`/`strict` route と canonical gate manifest、
Rust workspace/platform check、published adopter harness が Runtime と Repository Protocol
を検証します。target には `install.sh`、Python lockfile、source Make orchestration がなく、
adopter の toolchain/stack coverage は adopter または hosted provider が設定し evidence を
提供します。これは product parity として暗黙に扱いません。

`smoke.yml` は project-test manifest/core/governance/installer/lifecycle/release shard、
template aggregation、installation smoke、conditional release evidence、最終 CI evidence
receipt を持ちます。Rust target は `ci.yml`（Contract-aware quality、Windows、locked
behavioral oracle）、`release.yml`（archive、SBOM、checksum、provenance、release policy）、
canonical gate manifest、strict public/N-1 adopter acceptance に分割しています。source の
Python test shard、`install.sh`/Make smoke、exploratory latest-toolchain probe に target の
equivalent はなく、external/adopter responsibility と明示します。

これは responsibility の semantic parity であり、workflow byte や source command の
parity ではありません。target shell script には現在 syntax validation がありますが、
target script の ShellCheck gate は、存在しない target installer を検査する source gate と
異なるため、別の CI hygiene decision とします。本 batch は source Python module、Make
target、installer、multi-stack fixture を copy しません。

## WI-305 — architecture、installation、verification の file-level batch

WI-305 は pinned commit の次の 4 つの deferred reference file を一つずつ比較します。
read-only installation detector、任意の 10 段階 Interactive Installer Wizard、stage-aware
lightweight verification、Wizard の input/localization primitive を対象にします。Python
adapter の byte を copy するのではなく、Rust Runtime、adopter の external boundary、または
reference-only を各 file ごとに記録します。

| Reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/architecture/installation-detection-boundary.md` | implemented-different-by-design | `inspect`、`status`、`doctor`、`attach`、`profile propose`、first-calibration 文書と CLI attach/profile test が read-only facts と明示的な write boundary を提供します。immutable Release install と repository onboarding は分離されます。 |
| `docs/architecture/interactive-installation-wizard.md` | reference-only | source の 10-stage wizard、dry-run Installer preview、confirmation UI は Rust Runtime の feature ではありません。target の adopter route は public Release verify の後に `inspect` → `attach` → profile review/confirm → `doctor` を明示的に行います。Agent adapter は conversation UI を持てますが approval を作れません。 |
| `docs/architecture/lightweight-verification-and-soft-gates.md` | implemented-different-by-design | typed stage、policy-driven tier、fail-closed governance decision、明示的な skipped/unknown、request-scoped context、dynamic `light`/`standard`/`strict` route、advisory cost/reuse telemetry を verification route、CI gate、cost test がカバーします。source の `hard`/`soft`/`informational` checker label は generic wire enum として copy せず、documented boundary として明示します。Make/Python checker orchestration も copy しません。 |
| `docs/architecture/wizard-io-and-localization.md` | implemented-different-by-design | CLI/MCP の human Outcome と command presentation は `en`/`zh-CN`/`ja` を localize し、Contract value をそのまま保持し、明示的 command/preflight boundary で fail closed します。target に Interactive Installer Wizard はないため Wizard 専用 TTY back/pause/help は Runtime feature ではなく、conversation control は adapter が所有します。 |

### File-level findings と migration boundary

source detector の `new_adoption`/`upgrade` は、target では Release install と repository-local
attach/profile decision の分離に対応します。target inspection は read-only、`attach` と
profile confirmation は明示的な repository write です。prose や検出した stack から authority
を推論しません。active Work Item、dirty state、conflict、symlink risk、missing facts は stop
または review の理由であり、推測の理由ではありません。

source Interactive Wizard は Python Installer の convenience layer であり、Installer を Rust
repository に持ち込む要件ではありません。10 stages、dry-run、cancel、rollback boundary、
commit/push/PR/merge をしない約束は target installation route の adopter boundary として明示
します。target は第二の transaction authority や Contract/preflight/human decision を迂回する
prompt を提供しません。

source soft-gate の `hard`、`soft`、`informational` は target の generic wire enum として copy
せず、fail-closed governance decision と明示的な advisory observation の boundary に対応します。
stage に適用しない check も理由付きで明示され、trend/cost observation は advisory のままです。
`pre_ci` は hosted CI evidence ではありません。tier と assurance は policy に bind され、
execution speed から推論されません。adopter repository でも shared Runtime、明示的な `--repo`、
provider/enterprise の delegated evidence という境界を保持します。

Localization は presentation のみを対象にします。Runtime が生成する heading、status、unknown、
recovery、next action は設定言語にできますが、path、command、Contract intent、acceptance
criteria、machine evidence は authored value のままです。一般翻訳や source-compatible Wizard UI
を提供するという主張ではありません。この slice に `migrate-gap` はなく、Interactive Wizard は
未記録の omission ではなく明示された reference-only boundary です。

## WI-308 — evidence governance、trust、rollback-corruption の file-level slice

WI-308 は pinned source commit `e5acb677` の 4 file、すなわち visual demo asset、仮想の
rollback-corruption case study、Evidence Governance、Trust Layer を一つずつ比較します。
source の実装や binary asset は Rust repository に copy しません。

| Reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/assets/ai-cockpit-demo.gif` | reference-only | GIF89a、800x435、587,945 bytes、SHA-256 `88838de7221dc859efde7e8e87913d0a23a21466195647ded60612adbad1f795` の固定 visual reference です。binary copy や Runtime contract は主張しません。 |
| `docs/case-study-ai-rollback-corruption.md` | implemented-different-by-design | 三言語 adversarial-validation と typed Contract/scope check が unauthorized path、無関係な変更、controlled recovery を扱います。case は仮想であり、Runtime は auto-rollback、merge approval、business impact 推論を行いません。 |
| `docs/concepts/evidence-governance.md` | implemented-different-by-design | `docs/security/enterprise-governance.*`、`docs/reference/outcome-report.md`、typed Protocol/Repository evidence が Evidence → Governance Decision → Human Control を投影します。provider evidence は delegated で、prose は proof ではありません。 |
| `docs/concepts/trust-layer.md` | implemented-different-by-design | `docs/architecture/product-boundary.md`、`docs/philosophy.md`、enterprise-governance、Runtime capability truth registry が calibrated trust、fail-closed unknown、human control、non-goals を定義します。Source public claim matrix は target gate ではありません。 |

これは semantic responsibility parity であり source wire/byte compatibility ではありません。target
の Contract/evidence schema と shared request-scoped Runtime は source の安全意図を保ちつつ、repository
identity、snapshot、human decision、provider boundary を明示します。GIF は意図的に reference-only
です。Python、Make、installer、binary は copy せず、local evidence を provider/enterprise assurance
へ昇格させません。中国語と英語にも同じ結論と reader route を記載します。
## WI-323 reference documentation foundation

WI-323 は pinned source commit の次の 9 つの deferred documentation path を一つずつ比較します。
source tooling を copy せず、Runtime authority も変更しない documentation batch です。

| Reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/contributing/installation-document-maintenance.md` | implemented-different-by-design | tri-language reader route と documentation acceptance が thin home、link/metadata、version-neutral、no-guess/no-overwrite/no-fallback、separate approval の boundary を保ちます。 |
| `docs/current/README.md` | implemented-different-by-design | `docs/current/README.*`、`.ai/README.md`、`.ai/glossary.md`、`AGENTS.md`、`docs/reference/README.*` が current Agent read route です。source の `make ai-documentation-read-set` は target command ではありません。 |
| `docs/design/harden-work-item-pr-closure.md` | implemented-different-by-design | `docs/reference/agent-workflow.*`、`docs/reference/commands.md`、Rust lifecycle が latest base、dedicated branch、reviewed PR、merge-before-close、synchronization、exact cleanup を強制します。provider PR operation は external です。 |
| `docs/distribution.md` | implemented-different-by-design | target の current route と `docs/release/distribution.*` が compatibility entry、immutable artifact install、post-release adopter boundary を提供します。 |
| `docs/enterprise-security-boundary.md` | implemented-different-by-design | `docs/security/enterprise-deployment-boundary.*`、`enterprise-governance.*`、`SECURITY.md` が repository evidence と delegated identity、sandbox、audit、certification control を分離します。 |
| `docs/examples/trust-layer-demo.sh` | reference-only | offline stop/continue example は explanatory source material のままです。target evidence は typed Runtime preflight、capability、intent、adversarial test であり、shell authority は copy しません。 |
| `docs/features/human-benefit-report.md` | implemented-different-by-design | Rust `OutcomeV2`、`work-item outcome`、MCP `work_item_outcome`、tri-language handoff test が human report order と evidence boundary を保ちます。 |
| `docs/features/human-benefit-report.zh-CN.md` | implemented-different-by-design | Chinese presentation は同じ Rust Outcome/MCP route を使い、Contract acceptance text は authored value のまま machine translation しません。 |
| `docs/features/human-benefit-report.ja.md` | implemented-different-by-design | Japanese presentation は同じ Rust Outcome/MCP route を使い、Contract acceptance text は authored value のまま machine translation しません。 |

Cursor adopter feedback は version を正規化すると、この boundary と整合します。現在の
Runtime は安定した stdout JSON と human handoff を出し、`work-item new`/`start` は未 close
archive と事前変更を拒否し、readiness も明示します。CLI は Cursor の chat panel を展開できない
ため、provider/Agent adapter が human handoff を表示または再生します。診断 remediation、
close-gap convenience command、optional controls scaffold は後続の product decision であり、
この batch の parity として暗黙に claim しません。target に `Makefile.ai` 要件はなく、明示的
`--repo` の CLI/MCP が repository-neutral adopter interface です。

これは semantic responsibility parity であり source wire/byte parity ではありません。source
Make/Python report generator、installer script、trust demo は copy しません。object/adopter
boundary は全 adopter で同じです。shared external Runtime、repository-local `.ai/` state、
explicit repository context、provider-owned conversation presentation を使います。

## WI-326 quality gate、overview、design philosophy、closure plan の file-level batch

WI-326 は pinned reference の次の 9 path を一つずつ比較します。8 path は
implemented-different-by-design、closure hardening plan は internal historical plan であり
current Runtime command contract ではないため reference-only とします。

| Reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/non-make-adaptation.ja.md` | implemented-different-by-design | Installation と Agent workflow route が external Runtime と repository-local adapter boundary を示します。Adopter-owned stack command は Core の外であり、source `Makefile.ai` bridge は copy/require しません。 |
| `docs/operations/quality-gates.ja.md` | implemented-different-by-design | Japanese CI quality-gate/manifest route が gate ownership、evidence、traceability、policy-selected `light`/`standard`/`strict` routing を保ちます。source Make target、Python checker registry、template-maintenance fixture は copy しません。 |
| `docs/operations/quality-gates.md` | implemented-different-by-design | Versioned Rust-native gate manifest と CI route が source quality-gate semantics を保ち、hosted CI と adopter stack check の owner boundary を分けます。 |
| `docs/operations/quality-gates.zh-CN.md` | implemented-different-by-design | Chinese quality-gate/manifest route は同じ evidence と dynamic-routing boundary を保ち、source Make/Python orchestration は target command ではありません。 |
| `docs/overview.ja.md` | implemented-different-by-design | Rust architecture、capabilities、Agent workflow、command route が source five-layer overview を request-scoped/repository-bound governance として保ちます。source status/verification registry は copy しません。 |
| `docs/philosophy/design-philosophy.ja.md` | implemented-different-by-design | Japanese product-boundary、capability、enterprise-governance docs が calibrated trust、evidence over self-declaration、proportional control、human responsibility を保ちます。 |
| `docs/philosophy/design-philosophy.md` | implemented-different-by-design | English product-boundary、capability、enterprise-governance docs が同じ原則を保ちます。Core は Agent Runtime、sandbox、identity provider、compliance certificate ではありません。 |
| `docs/philosophy/design-philosophy.zh-CN.md` | implemented-different-by-design | Chinese product-boundary、capability、enterprise-governance docs が同じ原則と明示的 non-goal を保ちます。 |
| `docs/plans/harden-work-item-pr-closure.md` | reference-only | Source は Python `ai-finish`/`ai-close` の internal historical hardening plan です。Current Rust lifecycle と governance-integrity route は closure intent を保ちますが、obsolete step/command name は current capability ではありません。 |

この batch に `migrate-gap` はありません。これは semantic boundary parity であり source wire/byte
compatibility ではありません。Quality decision は versioned manifest と current Runtime が担当し、
hosted provider check、adopter stack command、enterprise control は delegated のままです。
Dynamic routing は policy が選び、execution speed から stricter tier を推測せず、tier を assurance
level と同一視しません。Published Runtime を object engineering repository で使う場合も明示的な
`--repo` binding が必要です。

## WI-327 adopter、calibration、long-cycle 文書 slice

WI-327 は pinned source commit の次の 9 deferred path を一つずつ比較します。8 path は
implemented-different-by-design、Bandit audit は source Python toolchain に固有の履歴であるため
reference-only とします。

| Reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/adopter-long-cycle-validation.ja.md` | implemented-different-by-design | Published binary の adopter/upgrade acceptance、distribution route、日本語 lifecycle/security docs が isolated install、lifecycle、rollback、cleanup evidence を保ちます。source multi-stack fixture と Make/Python orchestration は copy しません。 |
| `docs/reference/adopter-long-cycle-validation.md` | implemented-different-by-design | Published binary の adopter/upgrade acceptance、distribution route、lifecycle/security docs が isolated install、lifecycle、rollback、cleanup evidence を保ちます。source multi-stack fixture と Make/Python orchestration は copy しません。 |
| `docs/reference/adoption-reality-report.md` | implemented-different-by-design | Runtime capability/profile/status projection と immutable adopter acceptance receipt が template capability、adopter execution、provider evidence、enterprise assurance を分離します。local file を external proof へ昇格させません。 |
| `docs/reference/bandit-synchronization-security-audit.md` | reference-only | Source 固有の historical Bandit finding inventory です。target に Python/Bandit surface はなく、source count/digest を主張しません。Rust-native quality と threat-model boundary は別に記載します。 |
| `docs/reference/calibration-inventory.md` | implemented-different-by-design | Repository-bound profile proposal/confirmation、capability/status projection、explicit unknown が fact/evidence boundary を保ち、source の ten-column Python inventory は copy しません。 |
| `docs/reference/calibration-profiles.ja.md` | implemented-different-by-design | 日本語 calibration guide と strict JSON profile policy が累積 Lite/Standard/Strict control、人の選択、単調な upgrade、明示的 downgrade evidence を保ちます。Work Item quality routing とは別です。 |
| `docs/reference/calibration-profiles.md` | implemented-different-by-design | Calibration guide と strict JSON profile policy が累積 Lite/Standard/Strict control、人の選択、単調な upgrade、明示的 downgrade evidence を保ちます。Work Item quality routing とは別です。 |
| `docs/reference/calibration-profiles.zh-CN.md` | implemented-different-by-design | 中国語 calibration guide と strict JSON profile policy が累積 Lite/Standard/Strict control、人の選択、単調な upgrade、明示的 downgrade evidence を保ちます。Work Item quality routing とは別です。 |
| `docs/reference/calibration-session-model.ja.md` | implemented-different-by-design | Target は calibration proposal、confirmation、repository-bound fact を明示します。汎用 interactive Session や checklist authority は導入せず、unknown と human responsibility を可視化します。 |

これは semantic responsibility parity であり、source wire や command byte parity ではありません。Target は
shared external Runtime、repository-local `.ai/`、明示的な `--repo` を使い、provider identity、hosted
CI、signing、SBOM、provenance、enterprise control は delegated evidence とします。Cursor adopter は
repository-local adapter を明示的に install し、永続化された `work-item outcome` handoff を再生します。
Runtime は IDE chat panel を強制的に expand できないため、現在の output と lifecycle entry gate は
automatic chat posting の主張ではありません。Diagnostic remediation、close-gap convenience command、
automatic controls scaffold は別の product decision として扱います。

## WI-328 calibration と capability-truth の file-level batch

WI-328 は pinned reference の次の 9 path を一つずつ比較します。5 path は
implemented-different-by-design、capability matrix/claim-authoring の 4 path は
Rust target に source public claim checker/matrix がないため reference-only とします。

| pinned reference path | Classification | Rust/adopter の対応と境界 |
| --- | --- | --- |
| docs/reference/calibration-session-model.md | implemented-different-by-design | Repository-bound profile proposal、confirmation、calibration facts が fact/evidence boundary を保ちます。汎用 persisted Session は導入しません。 |
| docs/reference/calibration-session-model.zh-CN.md | implemented-different-by-design | Chinese calibration/profile route も proposal、confirmation、unknown、human authority を同じ境界で扱います。 |
| docs/reference/calibration-session.ja.md | implemented-different-by-design | Source ten-stage Session は target の明示的 profile proposal/confirmation に意味だけを写し、Make/Python orchestration は copy しません。 |
| docs/reference/calibration-session.md | implemented-different-by-design | Source の persisted wizard は source-specific orchestration です。Target calibration は read-only-first、repository-bound で、policy change に human confirmation を要求します。 |
| docs/reference/canonical-terminology.md | implemented-different-by-design | .ai/glossary.md、configuration、Outcome reference が canonical terms を提供します。Governance light と Calibration lite は alias ではなく、release は profile ではなく operation です。 |
| docs/reference/capability-claim-authoring.md | reference-only | Source lexical claim checker と matrix-binding front matter は target Runtime gate ではありません。将来の strict claim/evidence binding は候補 WI-330 の scope です。 |
| docs/reference/capability-evidence-freshness.md | reference-only | Work Item verification freshness はありますが、Capability Truth row expiry/portable-environment matrix は実装していません。拡張は候補 WI-330 で定義します。 |
| docs/reference/capability-truth-matrix.json | reference-only | Source 30-row public matrix は copy しません。capability_truth_registry は observed-capability projection であり public claim authorization や adopter/provider proof ではありません。 |
| docs/reference/capability-truth-matrix.md | reference-only | Current capability/adoption page は observed fact、adopter、provider、enterprise の境界を示し、source matrix/checker を claim しません。 |

4 つの reference-only は明示的な product boundary であり、未登録 omission では
ありません。WI-330 は file ごとの比較を閉じ、source claim checker、row freshness matrix、
public matrix が current Runtime の機能ではないことを記録します。将来 Rust-native
claim/evidence gate を導入するかは任意の product decision であり、別の human-owned scope
なしに Python/V1 asset を昇格させません。

Cursor adopter feedback は external validation input です。Current Runtime の stable
lifecycle JSON、replay 可能な work-item outcome、close-before-next/readiness check、
fail-closed start/verification binding はすでに確認済みです。Runtime は IDE chat panel を
expand できないため adapter/host が durable handoff を表示・再生します。Diagnostic
remediation、controls scaffold、close-gap convenience、Makefile integration は後続の
product decision であり、この batch の parity として claim しません。

## WI-330 capability-truth boundary の決定

WI-330 は pinned source の 4 file を一つずつ再確認し、最終決定を記録します。Target の
`capability show` は repository と snapshot に bind された projection のままです。Public
claim authorization と Capability Truth row expiry は current Runtime の外側です。

| Pinned source path | Final classification | 決定と target counterpart |
| --- | --- | --- |
| docs/reference/capability-claim-authoring.md | reference-only | Source lexical trigger/claim-binding checker は copy しません。文書 metadata は evidence ではなく、public wording は current bounded evidence と limitation に依存します。Counterpart: docs/capabilities.ja.md, crates/cockpit-repository/src/lib.rs。 |
| docs/reference/capability-evidence-freshness.md | reference-only | Work Item receipt freshness はありますが、source Capability Truth row expiry と portable-environment policy はありません。Counterpart: Runtime evidence validation、docs/reference/outcome-report.ja.md。 |
| docs/reference/capability-truth-matrix.json | reference-only | Source 30-row matrix を Rust wire format や authorization source として copy しません。capability_truth_registry は observed fact、adopter state、external exclusion だけを報告します。Counterpart: crates/cockpit-protocol/src/lib.rs, crates/cockpit-repository/src/lib.rs。 |
| docs/reference/capability-truth-matrix.md | reference-only | Target capability/adoption page は observed/evidence/provider/enterprise boundary を説明し、source matrix/checker を宣伝しません。Counterpart: docs/capabilities.ja.md。 |

これは product-boundary の決定であり、未追跡 omission ではありません。将来 claim binding または
row freshness を追加する場合は、human-owned Work Item で Rust-native schema、evidence generation、
stale handling、multilingual scope、adopter acceptance を先に定義してから classification を変更します。

## WI-331 checks catalog と CI/release evidence

WI-331 は pinned source の次の 2 path を一つずつ比較します。いずれも
implemented-different-by-design です。Rust target は source の quality/release
evidence の責任境界を保ちますが、source の Make、Python、V1 runtime はコピーしません。

| Pinned source path | 分類 | Rust counterpart / 境界の決定 |
| --- | --- | --- |
| `docs/reference/checks-catalog.md` | implemented-different-by-design | `docs/reference/checks-catalog.*`、Contract-aware `gate` route、repository gate manifest、Rust workspace checks、conformance/docs checks、release/adopter checks が同じ段階的な quality intent を担います。local check は provider/enterprise assurance と分離し、unknown または release-owned control は dynamic light/standard/strict profile で escalation します。 |
| `docs/reference/ci-release-evidence.md` | implemented-different-by-design | `docs/reference/ci-release-evidence.*`、`.github/workflows/ci.yml`、`.github/workflows/release.yml`、release distribution checks、adopter acceptance harness が provider job、commit/base/head、artifact、checksum、SBOM、provenance、isolation receipt を bind します。skip/failed job は隠さず、PR の prose は evidence になりません。 |

責任境界は明示されています。target Runtime は repository-local Contract と
gate decision を持ち、hosted CI、署名、SBOM/provenance provider、enterprise
audit system は delegated evidence を持ちます。公開 Release truth は immutable
tag と download artifact に bind します。全 command は明示的な `--repo` を要求し、
source Makefile、Python runner、コピーした V1 runtime は target requirement
ではありません。6 つの言語版と inventory assertion がこの batch の omission
防止記録です。

## WI-332 — P0 comprehension-review evidence

WI-332 は pinned source の comprehension-review evidence 3 file を一つずつ読みます。
3 file はすべて `reference-only` です。これは reference repository 固有の過去の desk
review record であり、reviewer、日付、score、言語別の結論を target の evidence に移す
ことはできません。Target は localized home、philosophy、architecture、Agent workflow
と documentation acceptance check で 6 問の reader route を保ちますが、独立した母語
editorial review を捏造したり source evidence bytes を copy したりしません。これは
semantic reader alignment であり、source study の合格を宣言するものではありません。

| Pinned source path | 分類 | Rust counterpart / 境界の決定 |
| --- | --- | --- |
| `docs/reference/comprehension-review-2026-08-14.md` | reference-only | `docs/README.md`、`docs/philosophy.md`、`docs/architecture.md`、`docs/reference/agent-workflow.md`、`tests/docs/documentation_acceptance.sh` が English reader route と構造 check を担います。source reviewer result は移植しません。 |
| `docs/reference/comprehension-review-2026-08-14.zh-CN.md` | reference-only | `docs/README.zh-CN.md`、`docs/philosophy.zh-CN.md`、`docs/architecture.zh-CN.md`、`docs/reference/agent-workflow.zh-CN.md` と documentation acceptance check が Chinese route を担います。母語 reviewer score は claim しません。 |
| `docs/reference/comprehension-review-2026-08-14.ja.md` | reference-only | `docs/README.ja.md`、`docs/philosophy.ja.md`、`docs/architecture.ja.md`、`docs/reference/agent-workflow.ja.md` と documentation acceptance check が Japanese route を担います。母語 reviewer score は claim しません。 |

外部 Cursor adopter feedback は別の validation input として扱います。Runtime の stable
lifecycle JSON、replay 可能な human Outcome、readiness/start gate、verification
invalidation は別 batch で確認済みです。この batch は automatic Cursor chat posting、
`Makefile.ai`、close-gap convenience、controls template を current parity として
黙って追加しません。

### Cursor adopter feedback の評価（v0.2.33）

以下の adopter matrix は現在の保証と明示的な境界を記録するもので、source の wire
互換性を宣言するものではありません。

| Feedback | Current boundary | Decision |
| --- | --- | --- |
| Agent 向け Outcome output | `finish`、`archive`、`close` は stdout に stable lifecycle JSON を出力します。`work-item outcome --json` と repository context 付き MCP `work_item_outcome` は replayable な machine entrypoint です。 | Runtime 実装済み。Cursor chat への handoff 表示は Cursor 側であり、CLI は IDE panel を開けません。 |
| 次の Work Item 前の close | readiness/lifecycle entry は active Work Item、未 close の archive、dirty source path、detached HEAD、未同期 default base を拒否します。 | fail-closed で実装済み。`ready_on_base` は明示状態です。 |
| start timing と base binding | start 前の non-governance change を拒否し、実装前に明示 branch/worktree/base context を bind します。 | fail-closed で実装済み。 |
| finalize/close diagnostics | error は failure boundary と recovery condition を示しますが、専用 `close-gap` remediation command はありません。 | Partial。詳細な診断は将来の bounded product decision です。 |
| controls scaffolding | 宣言済み controls/evidence を検証し、acceptance decision を発明せず完全な controls template も生成しません。 | 意図した decision-free boundary です。 |
| merge 後の close recovery | 明示的な `finalize`、`finalize-verify`、`close` と readiness/status projection が lifecycle をカバーします。 | Current lifecycle が authoritative。`close-gap` alias は任意の host UX です。 |
| Make integration | target は明示的な `--repo` CLI/MCP と provider adapter を使用します。source `Makefile.ai` orchestration は protocol requirement ではありません。 | parity omission ではなく、source Make/Python orchestration はコピーしません。 |
| verification invalidation | lifecycle boundary で source snapshot、Contract、repository identity、evidence binding を検証し、source change 後は fresh verification が必要です。 | fail-closed で実装済み。archive bytes は immutable historical truth です。 |

今後の Runtime 変更は human-owned な bounded Contract、test、三言語 documentation、
published-Runtime acceptance を持つ Work Item で行います。adopter feedback を未追跡の
promise にしません。

## WI-333 — comprehension-validation protocol と participant record

WI-333 は pinned source の comprehension-validation protocol、strict response schema、
匿名化された 6 件の response、bounded result を一つずつ確認しました。12 path はすべて
`reference-only` です。これらは reference repository が所有する外部の reader study であり、
participant response、revision、sample の結論を target の evidence に移すことはできません。
Target は reader-facing documentation route と Runtime evidence validation を participant
research から分離します。response bytes や source result を copy せず、この repository の
comprehension、release、safety、security、enterprise claim を source study から導きません。

| Pinned source path | 分類 | Target counterpart / 境界の決定 |
| --- | --- | --- |
| `docs/reference/comprehension-validation-protocol.md` | reference-only | `docs/README.md`、`docs/reference/agent-workflow.md`、`docs/reference/outcome-report.md`。source の eligibility、consent、interview、review protocol は外部です。 |
| `docs/reference/comprehension-validation-protocol.zh-CN.md` | reference-only | `docs/README.zh-CN.md`、`docs/reference/agent-workflow.zh-CN.md`、`docs/reference/outcome-report.zh-CN.md`。target の participant study を意味しません。 |
| `docs/reference/comprehension-validation-protocol.ja.md` | reference-only | `docs/README.ja.md`、`docs/reference/agent-workflow.ja.md`、`docs/reference/outcome-report.ja.md`。source ethics/eligibility は Runtime policy ではありません。 |
| `docs/reference/comprehension-validation-response.schema.json` | reference-only | `.ai/README.md`、`docs/reference/outcome-report.md`。participant-response schema は Runtime Contract/evidence schema ではありません。 |
| `docs/reference/comprehension-validation-responses/peter_01.en.json` | reference-only | `docs/README.md`、`docs/features/human-benefit-report.md`。歴史的 response、revision、pseudonym は source に bind されます。 |
| `docs/reference/comprehension-validation-responses/peter_02.en.json` | reference-only | `docs/README.md`、`docs/features/human-benefit-report.md`。participant data を `.ai/` に import しません。 |
| `docs/reference/comprehension-validation-responses/tanaka_01.ja.json` | reference-only | `docs/README.ja.md`、`docs/features/human-benefit-report.ja.md`。source response は adopter/Runtime evidence ではありません。 |
| `docs/reference/comprehension-validation-responses/tanaka_02.ja.json` | reference-only | `docs/README.ja.md`、`docs/features/human-benefit-report.ja.md`。source revision-bound fact は外部に保持します。 |
| `docs/reference/comprehension-validation-responses/xiaoli_01.zh-CN.json` | reference-only | `docs/README.zh-CN.md`、`docs/features/human-benefit-report.zh-CN.md`。target の native-language score を claim しません。 |
| `docs/reference/comprehension-validation-responses/xiaoli_02.zh-CN.json` | reference-only | `docs/README.zh-CN.md`、`docs/features/human-benefit-report.zh-CN.md`。raw participant text を copy しません。 |
| `docs/reference/comprehension-validation-results.json` | reference-only | `docs/features/human-benefit-report.*`、`docs/reference/reference-file-comparison.*`。sample count/result は source revision に bind されます。 |
| `docs/reference/comprehension-validation-results.md` | reference-only | `docs/features/human-benefit-report.md`、`docs/reference/outcome-report.md`。source limitation は target verification/release evidence ではありません。 |

この境界は意図的です。adopter repository は target の documentation route、Contract、evidence、
Agent workflow を継承できますが、他 repository の human-subject evidence は継承しません。
将来 study を行う場合は、独立した consent、retention、privacy、evidence Contract が必要です。

## WI-334 — Evidence Binding と reuse の基礎

WI-334 は pinned source `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の 10 path を一つずつ確認しました。
10 path はすべて `implemented-different-by-design` です。Rust target は content、diff、environment、
command、toolchain、policy、profile、Runtime、stage、runner identity を strict な複合
`EvidenceContext` として束縛します。source の Python module は copy せず、source JSON/API 互換も claim しません。

| Pinned source path | 分類 | Rust counterpart / 境界の決定 |
| --- | --- | --- |
| `docs/reference/content-bound-evidence-reuse.md` | implemented-different-by-design | `cockpit-evidence` は content identity を複合 context の一部として扱い、exact binding の場合だけ advisory reuse を検討します。 |
| `docs/reference/diff-bound-evidence-reuse.md` | implemented-different-by-design | `DiffIdentity`、repository snapshot facts、reuse test が base/head と changed-path identity を束縛し、mismatch は rerun です。 |
| `docs/reference/environment-bound-reuse.md` | implemented-different-by-design | Runtime/toolchain/environment/profile/policy/command/stage を明示的に束縛し、process environment 全体は serialize しません。 |
| `docs/reference/evidence-binding-foundation.md` | implemented-different-by-design | versioned `ReusableReceipt` が content-addressed identity、expiry、node、passed を検証し、protected/required check を bypass しません。 |
| `scripts/ai_evidence_binding.py` | implemented-different-by-design | typed Rust struct、deny-unknown-fields、deterministic fail-closed decision が Python builder/validator を置き換えます。 |
| `scripts/ai_diff_bound_reuse.py` | implemented-different-by-design | typed `DiffIdentity` と Git snapshot facts が source helper を置き換え、canonical path/revision mismatch semantics を保ちます。 |
| `scripts/ai_environment_reuse.py` | implemented-different-by-design | 明示的で bounded な environment input と digest field を使い、credential を read/persist しません。 |
| `tests/test_ai_evidence_binding.py` | implemented-different-by-design | Rust evidence/repository test が strict schema、tamper、mismatch、expiry、failed/protected node、rerun decision を検証します。 |
| `tests/test_ai_diff_bound_reuse.py` | implemented-different-by-design | Rust evidence/Git test が clean/changed path、canonical ordering、malformed path、policy mismatch、expiry、immutability を検証します。 |
| `tests/test_ai_environment_reuse.py` | implemented-different-by-design | Rust evidence/executor test が environment/toolchain identity、stale/unknown receipt、protected execution、digest validation を検証します。 |

この batch は semantic responsibility parity であり、source wire parity ではありません。Reuse は
optimization/evidence observation で、exact な fresh binding だけを候補にします。governance、coverage、
security、required-check gate の責任は caller に残ります。Inventory、三言語 ledger、WI-334 evidence がこの判断を束縛し、
source participant、Python、Make、V1 artifact は導入しません。

## WI-336 — 最初の 5 つの governance-documentation path

WI-336 は pinned reference commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の 5 path を
一つずつ読みました。portable な governance responsibility と、source 固有の report、provider
automation、historical cleanup tooling を分離して判定しています。

| Pinned source path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/cross-wi-integration.md` | reference-only | `docs/reference/reference-parity.md`、`docs/reference/outcome-report.md` と Work Item ごとの archive validation が target の audit boundary です。source の WI-04..WI-13 aggregate report と観測不能な conversation receipt は Runtime command ではありません。 |
| `docs/reference/dependabot-intake.md` | not-applicable | Dependabot bot branch intake は provider 固有です。generic delegated provider evidence と明示的な Work Item source binding は `docs/reference/ci-release-evidence.md` にありますが、Dependabot authorization path ではありません。 |
| `docs/reference/deprecated-assets-registry.json` | reference-only | `.ai/README.md`、`docs/reference/agent-workflow.md`、exact resource finalization が reviewed cleanup と immutable history boundary を保持します。source registry や Make scan は提供しません。 |
| `docs/reference/deprecated-assets.md` | reference-only | obsolete chain と registry hygiene の説明は source documentation に限定されます。Rust は明示的な `--repo`、Runtime lifecycle、immutable archive、resource finalization を使い、`check-deprecated-assets` の存在は claim しません。 |
| `docs/reference/derived-artifacts.md` | implemented-different-by-design | `docs/reference/outcome-report.md`、`docs/reference/verification-semantics.md`、`.ai/README.md`、typed Runtime projection が Contract/evidence/archive fact と status/Outcome view を分離します。source Python registry は authority として不要で、読み込みません。 |

これは semantic responsibility comparison であり、source command や wire compatibility の主張ではありません。
Rust は reference Python、Make target、Dependabot workflow、deletion registry、generated history を copy しません。
Work Item archive と human Outcome が authority であり、derived view は後続の決定を authorize できません。残りの ledger record は明示的に deferred のままです。

## WI-343 — reference inventory foundation の reconciliation

WI-339 は次の 5 つの pinned path をすでに一つずつ比較していましたが、machine inventory
はそれらを `deferred-next-batch` のまま残していました。WI-343 は既存の判断を deterministic
に inventory へ登録するだけで、Runtime behavior の変更や source tooling の copy は行いません。

| Pinned source path | Classification |
| --- | --- |
| `docs/reference/cross-wi-integration.md` | `reference-only` |
| `docs/reference/dependabot-intake.md` | `not-applicable` |
| `docs/reference/deprecated-assets-registry.json` | `reference-only` |
| `docs/reference/deprecated-assets.md` | `reference-only` |
| `docs/reference/derived-artifacts.md` | `implemented-different-by-design` |

Tri-language ledger と generated inventory は一致し、240 implemented-different-by-design、
4 not-applicable、30 reference-only、582 deferred、`migrate-gap` は 0 です。これは ledger
reconciliation であり、source command や JSON-wire compatibility の主張ではありません。

## WI-342 — ドキュメント、配布、enterprise boundary の batch

WI-342 は pinned reference commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の 10 path を一つずつ読みました。
8 path は `implemented-different-by-design`、2 path は `reference-only` です。
target は reader route、distribution、authority boundary、enterprise boundary の責任を保持しますが、
source の Python/Make orchestration、source adopter record、provider claim はコピーしません。

| Pinned source path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/distribution.md` | implemented-different-by-design | `docs/release/distribution.*` と public/N-1 adopter acceptance が immutable Release、shared Runtime install、repository binding、checksum/SBOM/provenance、cleanup boundary を担当します。 |
| `docs/reference/distribution.ja.md` | implemented-different-by-design | Japanese route は `docs/release/distribution.ja.md` と同じ target-specific acceptance harness で表現し、source Make/Python の詳細や bytes はコピーしません。 |
| `docs/reference/documentation-architecture.md` | implemented-different-by-design | `docs/current/README.md`、getting-started/reference route、tri-language documentation checks、comparison ledger が canonical layer、reader route、owner、split rule を保持します。 |
| `docs/reference/documentation-architecture.ja.md` | implemented-different-by-design | Japanese current/getting-started/reference route が source の reader map と language boundary を保持し、`.ai/README.md` と明示的 Runtime page が instruction boundary です。 |
| `docs/reference/documentation-authority-boundary.md` | implemented-different-by-design | `.ai/README.md`、`AGENTS.md`、current/reference route、frontmatter、documentation acceptance が current instruction、opt-in reference、historical record を分離します。 |
| `docs/reference/documentation-authority-registry.json` | implemented-different-by-design | 明示的な target route と metadata check が source topic registry を置き換えます。global Agent configuration や未検証の source topic capability は導入しません。 |
| `docs/reference/documentation-context-registry.json` | reference-only | Source plan/context label は source 内部 record であり、portable Runtime authority や adopter evidence ではありません。target は current `.ai` instruction と immutable Work Item/archive history を保持し、registry はコピーしません。 |
| `docs/reference/enterprise-control-checklist.md` | implemented-different-by-design | tri-language enterprise-governance、deployment-boundary、adopter-configuration が repository fact、delegated evidence、retention/audit owner、non-certification claim を分離します。 |
| `docs/reference/enterprise-control-matrix.json` | reference-only | Source observed-control row は portable compliance result ではありません。target は delegated evidence と policy route で current external receipt を要求し、source `not_verified` state はコピーしません。 |
| `docs/reference/external-identity-boundary.md` | implemented-different-by-design | typed Rust authority/approval evidence、policy precedence、external evidence import、Contract field、enterprise page が identity level を保持しますが、person を local に authenticate しません。 |

2 つの `reference-only` record は target capability に昇格しません。source context metadata と source adopter
control observation は evidence として移転できません。これは semantic/documentation parity であり、
JSON-wire parity ではありません。object/adopter boundary は shared Runtime、repository ごとの `.ai/` isolation、
external provider evidence、organization-level identity/compliance を主張しないことです。

この batch 後の ledger は 5,119 record です。4,262 `generated-history`、240
`implemented-different-by-design`、1 `implemented-equivalent`、4 `not-applicable`、30
`reference-only`、582 `deferred-next-batch`、`migrate-gap` は 0 です。582 deferred は後続の逐次比較であり、
parity claim ではありません。

## WI-344 — reference documentation batch 14

WI-344 は pinned reference の次の 5 document を一つずつ確認しました。3 つの責任は
Rust-native reader/Runtime boundary で表現され、2 つは source-specific historical report
として target capability/evidence に昇格しません。

| Pinned reference path | Classification | Rust counterpart / 境界の決定 |
| --- | --- | --- |
| `docs/reference/failure-recovery-usability.md` | implemented-different-by-design | `docs/reference/troubleshooting.md`、`docs/features/task-outcome-report.md`、`docs/reference/outcome-report.md` と typed recovery/Outcome service が repository-bound な failed gate、recovery condition、intervention、stop、resolution、next action を扱います。source の 9 scenario Python report wire shape は別 batch です。 |
| `docs/reference/final-north-star-acceptance.json` | implemented-different-by-design | `docs/reference/final-replacement-acceptance.md`、parity ledger、final-replacement harness が 20 dimension と external adopter/provider limitation を保持し、source decision bytes は import しません。 |
| `docs/reference/final-north-star-acceptance.md` | implemented-different-by-design | Design Philosophy、Product Boundary、Outcome、final-replacement acceptance が North Star を保持し、local check と external evidence を分離します。 |
| `docs/reference/final-wiii-remediation-closure-audit.md` | reference-only | source WIII の PR identity、reviewer、historical closure claim は portable な target evidence ではありません。Rust は自身の Work Item intelligence/parallelism route を持ちます。 |
| `docs/reference/full-remediation-acceptance.md` | reference-only | source WI-01–WI-19 remediation sequence は internal history です。target は自身の evidence-bound acceptance route のみを保持し、source progress/Release claim を公開しません。 |

これは semantic/documentation parity であり、source command や JSON-wire parity ではありません。
source recovery/acceptance script と test は各 file-level comparison で別途扱います。object/adopter
boundary は shared Runtime、repository state isolation、独立した evidence binding です。

現在の ledger は 5,119 record です。4,262 `generated-history`、252
`implemented-different-by-design`、1 `implemented-equivalent`、4 `not-applicable`、34
`reference-only`、566 `deferred-next-batch`、`migrate-gap` は 0 です。deferred は予定作業であり、parity claim ではありません。

## WI-345 — governance cost / performance documentation batch 15

WI-345 は pinned reference commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の 5 document を一つずつ比較しました。2 つの complexity document は Python/Make scanner と source threshold が Rust Runtime behavior ではないため `reference-only` のままです。Cost、performance budget、profile/cost separation は Rust-native な repository-bound projection で表現しますが、advisory boundary を明示します。

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/governance-complexity.ja.md` | reference-only | `docs/reference/governance-complexity.ja.md`、`docs/reference/governance-integrity-gate.ja.md`、immutable archive rule が境界を記録します。source complexity scanner、Make target、threshold はコピーしません。 |
| `docs/reference/governance-complexity.md` | reference-only | `docs/reference/governance-complexity.md`、`docs/reference/governance-integrity-gate.md`、`inspect/status/doctor` が repository fact と archive integrity を保持しますが、source metric equivalence は主張しません。 |
| `docs/reference/governance-cost-metrics.md` | implemented-different-by-design | `ai-cockpit diagnose --repo <repo> [--work-item <id>]`、typed `VerificationCostEstimate`/`VerificationCostObservation`、`docs/reference/verification-cost.md` が identity-bound advisory fact を提供します。source JSONL phase/wait parser と wire shape は Runtime requirement ではありません。 |
| `docs/reference/governance-performance-budget.md` | implemented-different-by-design | typed `PerformanceBaseline`/`PerformanceAssessment`、`tests/performance/regression_gate.sh`、`tests/performance/README.md` が明示的 local budget を扱います。P95 を推測せず、必須 verification を弱めません。 |
| `docs/reference/governance-profile-cost-separation.md` | implemented-different-by-design | `docs/reference/governance-profile-cost-separation.md`、`ci-quality-gates.md`、`verification-route.md` が light/standard/strict、operation/stage escalation、VerificationTier、EvidenceAssurance、cost を分離します。 |

これは semantic/documentation parity であり、source command や JSON-wire compatibility ではありません。Object/adopter boundary は shared Runtime、明示的な `--repo`、repository-local evidence、policy-owned route requirement、弱い governance result を認可できない advisory cost/performance fact です。

WI-345 後の ledger は 5,119 record です。4,262 `generated-history`、246
`implemented-different-by-design`、1 `implemented-equivalent`、4 `not-applicable`、34
`reference-only`、572 `deferred-next-batch`、`migrate-gap` は 0 です。572 deferred は予定された比較であり、parity claim ではありません。WI-346 の現在の結果を次に記録します。

## WI-346 — Governance Profile と Cockpit Status の読み方

WI-346 は pinned reference commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の 6 document を一つずつ比較しました。
6 件すべてを `implemented-different-by-design` とします。target は明確な三言語の reader route を追加しましたが、
Rust Runtime、repository context、CI boundary は source の Make/Python orchestration とは異なります。

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/governance-profiles.ja.md` | implemented-different-by-design | `governance-profiles.ja.md`、`governance-profile-cost-separation.ja.md`、`ci-quality-gates.ja.md`、`verification-route.ja.md` が proportional profile、release escalation、cost/assurance 分離、fail-closed boundary を日本語で説明し、source dispatch bytes はコピーしません。 |
| `docs/reference/governance-profiles.md` | implemented-different-by-design | 英語の profile/cost separation、CI gate、verification route が Light/Standard/Strict、release escalation、mandatory floor、明示的な `gate --repo` boundary を target に写像します。 |
| `docs/reference/governance-profiles.zh-CN.md` | implemented-different-by-design | 中国語ページが profile、tier/assurance、cost、override の境界を保持します。source `make`/Python command を Rust requirement として示しません。 |
| `docs/reference/how-to-read-cockpit-status.ja.md` | implemented-different-by-design | 日本語の status reader、`outcome-report.ja.md`、`commands.ja.md` が人向け handoff を提供し、Contract 原文と evidence を authority とします。 |
| `docs/reference/how-to-read-cockpit-status.md` | implemented-different-by-design | 英語 reader、`outcome-report.md`、`commands.md` が source の reader label を Rust Outcome section、color、停止条件、次の action に対応づけます。 |
| `docs/reference/how-to-read-cockpit-status.zh-CN.md` | implemented-different-by-design | 中国語 reader が同じ安全な読み順と evidence boundary を提供します。自動翻訳で Contract の事実や承認を作りません。 |

6 ページは `VerificationTier`、`EvidenceAssurance`、advisory cost observation を分離します。🟢 は review 可能な evidence、
🟡 は不足または判断待ち、🔴 は停止を示し、どれも merge/release authorization ではありません。`unknown` は可視のまま推測で消しません。
明示的な `--repo`、Contract 原文、MCP/host presentation boundary を説明し、adopter repository が同じ動作を継承できます。

これは semantic/documentation parity であり、source command や JSON-wire parity ではありません。WI-346 後の現在 ledger は
5,119 record、4,262 `generated-history`、252 `implemented-different-by-design`、1 `implemented-equivalent`、4 `not-applicable`、
34 `reference-only`、566 `deferred-next-batch`、`migrate-gap` は 0 です。566 deferred は予定比較であり parity claim ではありません。

## WI-347 — Knowledge、input trust、installed lifecycle、capability assessment

WI-347 は pinned reference commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の 10 path を一つずつ比較しました。
10 件すべてを `implemented-different-by-design` とします。target は Rust-native の reader mapping と明示的な制限を追加しましたが、
source の Python/Make orchestration、generated assessment bytes、provider-global behavior は Runtime の外部です。

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/human-report-semantic-quality.md` | implemented-different-by-design | `docs/features/human-benefit-report.md`、`docs/features/task-outcome-report.md`、`docs/reference/outcome-report.md` が decision view の順序と forbidden-claim boundary を保持します。 |
| `docs/reference/implementation-knowledge.ja.md` | implemented-different-by-design | 日本語 Knowledge page と typed Knowledge record が read-only projection を提供し、source filter/record はコピーしません。 |
| `docs/reference/implementation-knowledge.md` | implemented-different-by-design | Rust Knowledge CLI/MCP は決定的な repository filter と `KnowledgeV2Record` を公開します。date/commit/supersession の広い query surface は明示的な non-claim です。 |
| `docs/reference/implementation-knowledge.zh-CN.md` | implemented-different-by-design | 中国語 Knowledge route が current filter、evidence binding、source query との差分を説明します。 |
| `docs/reference/input-trust-dataflow.ja.md` | implemented-different-by-design | 日本語 provenance guidance は typed `FactOrigin`、traceable derivation、fail-closed observation に対応します。 |
| `docs/reference/input-trust-dataflow.md` | implemented-different-by-design | Typed Rust fact、snapshot observation、input-trust test が source の分類と injection boundary を保持し、source JSON wire parity は主張しません。 |
| `docs/reference/input-trust-dataflow.zh-CN.md` | implemented-different-by-design | 中国語 route が provenance、cross-step、明示的 repository boundary を説明します。 |
| `docs/reference/installed-lifecycle.md` | implemented-different-by-design | Shared Runtime install、explicit attach、immutable Release acceptance、migration/rollback boundary を記載し、source installer Python/Make は reference material に留めます。 |
| `docs/reference/instruction-traceability.md` | implemented-different-by-design | Inventory、comparison/parity page、Work Item evidence、close receipt が structural forward/reverse traceability を提供し、source checker はコピーしません。 |
| `docs/reference/japanese-capability-assessment.json` | implemented-different-by-design | 三言語 capability page と executable presentation/adversarial check が bounded coverage を提供します。source assessment/corpus bytes と general fluency claim は reference-bound です。 |

これは semantic/documentation parity であり、source command や JSON-wire parity ではありません。Object/adopter boundary は shared Runtime、明示的な `--repo`、isolated repository fact/evidence、外部 provider/enterprise assurance のままです。Knowledge、provenance、installation、traceability、language projection は authority、benefit、approval、release evidence を作りません。

WI-347 後の ledger は 5,119 record、4,262 `generated-history`、262 `implemented-different-by-design`、1 `implemented-equivalent`、4 `not-applicable`、34 `reference-only`、556 `deferred-next-batch`、`migrate-gap` は 0 です。556 deferred は予定された比較であり parity claim ではありません。

## WI-348 — verification、operation-time policy、provider boundary batch

WI-348 は pinned commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の十個の path を
一つずつ比較します。七つは Rust で意図した別実装、三つの historical provider/pre-release
record は `reference-only` です。Rust Core に strict な operation-time evaluator を追加しますが、
これは policy input であり executor や provider authority ではありません。

| Pinned reference path | Classification | Rust counterpart / boundary |
| --- | --- | --- |
| `docs/reference/japanese-capability-assessment.md` | implemented-different-by-design | 三言語 Japanese assessment boundary、Outcome、adversarial、installation、documentation check。一般的な fluency は主張しません。 |
| `docs/reference/lightweight-verification-and-soft-gates.md` | implemented-different-by-design | 比例した route、content-bound reuse、決定的な partial dependency、単調な escalation、可視の advisory boundary。 |
| `docs/reference/multilingual-semantic-parity.md` | implemented-different-by-design | 三言語の Runtime-owned label/marker、安全、unknown、decision、limitation、next-action。Contract 値は作成言語を保持します。 |
| `docs/reference/open-pr-issue-reconciliation-662.json` | reference-only | Historical provider inventory。現在の state は新しい external observation が必要で、release/merge を許可しません。 |
| `docs/reference/open-pr-issue-reconciliation-662.md` | reference-only | Historical reconciliation narrative。current status や `.ai/` にコピーしません。 |
| `docs/reference/operation-time-policy-reevaluation.ja.md` | implemented-different-by-design | Rust `OperationTimeRequest`/decision evaluator と strict regression test。source Python trust/provider execution はコピーしません。 |
| `docs/reference/operation-time-policy-reevaluation.md` | implemented-different-by-design | operation、target、scope、authority、freshness、trust、impact を明示する同じ境界。 |
| `docs/reference/operation-time-policy-reevaluation.zh-CN.md` | implemented-different-by-design | 同じ fail-closed evaluator の中国語 reader route。 |
| `docs/reference/performance-diagnosis.md` | implemented-different-by-design | request-scoped `diagnose` と cost observation で execution/reuse を測定し、provider wait/P95/assurance を発明しません。 |
| `docs/reference/pre-release-documentation-alignment.json` | reference-only | Historical generated alignment receipt。target documentation は独自の repository-local check を使います。 |

これは source Python、Make、JSON wire、provider state の parity ではありません。更新後の
ledger は 5,119 records：4,262 `generated-history`、269 `implemented-different-by-design`、
1 `implemented-equivalent`、4 `not-applicable`、37 `reference-only`、546
`deferred-next-batch`、`migrate-gap` は 0 です。すべての object/adopter project は共有 Runtime、
明示的な `--repo`、repository-local evidence、isolation boundary を継承します。

## WI-368 — pre-release、adversarial、adopter、reference-impact batch

WI-368 は pinned commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の 11 path を逐一比較しました。
6 path は `implemented-different-by-design`、5 path は `reference-only` です。

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/reference/pre-release-documentation-alignment.md` | reference-only | Historical generated alignment。current docs は repository-local gate を使います。 |
| `docs/reference/pre-release-documentation-review.json` | reference-only | Historical five-strategy review。source finding は target release を authorize しません。 |
| `docs/reference/project-test-timing-baseline.json` | implemented-different-by-design | identity-bound performance sample と advisory budget。timing は verification を下げません。 |
| `docs/reference/provider-backed-governance-validation.md` | implemented-different-by-design | provider/hosted control は delegated evidence のままです。local check は証明になりません。 |
| `docs/reference/real-absurd-injection-cases.{md,zh-CN.md,ja.md}` | implemented-different-by-design | canonical manifest と Rust test で 15 structured case / 12 named RAI case を保持します。 |
| `docs/reference/real-adopter-reference-validation.md` | implemented-different-by-design | immutable public Release adopter/upgrade harness と isolation/lifecycle/cleanup evidence。 |
| `docs/reference/reference-impact-gate.{md,zh-CN.md,ja.md}` | reference-only | source static scanner/schema/Make surface は提供せず、operation-time policy は declared facts の狭い boundary に留めます。 |

この batch では Standard profile の overclaim も修正しました。static reference-impact scanner が存在するとは記載せず、
source の adversarial language page にある named-case count の差異は manifest を machine truth として可視化します。
これは semantic parity と明示的な boundary documentation であり、source command/JSON-wire compatibility ではありません。

## WI-378 reference documentation batch 17

WI-378 は pinned source commit にある次の deferred 10 path を一つずつ比較しました。9 つの責務は Rust-native の三言語文書と既存 Runtime/test で表現し、生成された plan trace 1 つは `reference-only` としました。source の Python、Make、provider configuration、historical remediation decision はコピーしません。

| Pinned reference path | Classification | Rust counterpart / boundary |
| --- | --- | --- |
| `docs/reference/remediation-instruction-traceability.json` | reference-only | `docs/reference/instruction-traceability.md` と machine inventory が現在の traceability boundary を説明します。source の generated historical plan directive は target authority ではありません。 |
| `docs/reference/repository-workflow.ja.md` | implemented-different-by-design | 三言語 `docs/reference/repository-workflow.*`、`.ai/README.md`、`AGENTS.md` が明示的 repository context、serial Work Item、reviewed PR、close、cleanup を保持します。 |
| `docs/reference/schemas.md` | implemented-different-by-design | 三言語 `schemas.*`、typed Protocol/repository validator、immutable evidence/decision boundary が record family に対応します。source wire compatibility は主張しません。 |
| `docs/reference/test-architecture.md` | implemented-different-by-design | 三言語 `test-architecture.*`、CI quality route、conformance manifest、release/adopter harness、negative-first test が layered evidence と external limit を説明します。 |
| `docs/reference/test-weakening-guard.ja.md` | implemented-different-by-design | 日本語 Rust-native weakening route、snapshot-derived governance signal、regression。source Python/Make surface は搭載しません。 |
| `docs/reference/test-weakening-guard.md` | implemented-different-by-design | 英語 Rust-native weakening route、保守的な path handling、dynamic profile boundary、recovery condition。 |
| `docs/reference/test-weakening-guard.zh-CN.md` | implemented-different-by-design | 中国語 Rust-native weakening route、fail-closed unknown、比例分析、明示した non-claim。 |
| `docs/reference/troubleshooting.ja.md` | implemented-different-by-design | 日本語 stop-state/recovery、command reference、installed-lifecycle boundary、documentation check が source wizard/Make instruction に対応します。 |
| `docs/reference/troubleshooting.md` | implemented-different-by-design | 英語 stop-state/recovery route と toolchain、adopter、active Work Item、evidence 保全の境界。 |
| `docs/reference/upgrade.ja.md` | implemented-different-by-design | 日本語 Runtime upgrade と repository migration の分離、immutable Release、rollback、history preservation。 |

更新後の ledger は 5,119 record：4,262 `generated-history`、284
`implemented-different-by-design`、1 `implemented-equivalent`、4
`not-applicable`、43 `reference-only`、525 `deferred-next-batch`、
`migrate-gap` は 0 です。Deferred は予定された比較であり parity claim ではありません。

## WI-379 reference documentation batch 18

WI-379 は pinned source commit の次の deferred 10 path を一つずつ比較しました。8 つの
責務は Rust-native 三言語 documentation で表現し、歴史的 audit 2 ファイルは
`reference-only` のまま保持します。Runtime code は追加せず、source の Python、Make、
provider configuration、generated history はコピーしません。

| Pinned reference path | Classification | Rust counterpart / boundary |
| --- | --- | --- |
| `docs/reference/upgrade.md` | implemented-different-by-design | 三言語 `upgrade.*`、`installed-lifecycle.*`、migration/conflict/rollback boundary。source installer command は説明用です。 |
| `docs/reference/verification-evidence-reuse-runtime.md` | implemented-different-by-design | `verification-evidence-reuse-runtime.*`、`verification-route.*`、`verification-semantics.*`、typed identity-bound receipt、protected node 実行、reuse metric。 |
| `docs/reference/verification-evidence-reuse.md` | implemented-different-by-design | `verification-evidence-reuse.*`、`verification-cost.*`、`verification-planner.*`。exact binding/invalidation と advisory call-count boundary。 |
| `docs/reference/verification-fixture-boundary.md` | implemented-different-by-design | `verification-fixture-boundary.*` と repository-native test。local fixture は Runtime/cache state を除外し provider/adopter evidence にはなりません。 |
| `docs/reference/wi01-wi20-bidirectional-traceability-audit.json` | reference-only | historical generated V1 audit bytes。current truth は pinned inventory、Work Item archive、evidence、三言語 traceability page です。 |
| `docs/reference/wi01-wi20-bidirectional-traceability-audit.md` | reference-only | source Python/Make evidence に束縛された歴史 narrative で、コピーせず current authority にしません。 |
| `docs/reference/wiii-v2-integration-audit.md` | implemented-different-by-design | `wiii-v2-integration-audit.*`、Rust `status`/intelligence projection、explicit schema/source identity check、scheduler/provider claim の除外。 |
| `docs/reference/work-item-intelligence-performance-baseline.md` | implemented-different-by-design | `work-item-intelligence-performance-baseline.*`、`diagnose`、advisory cost/performance observation。source benchmark 数値は主張しません。 |
| `docs/reference/work-item-lifecycle-closure.ja.md` | implemented-different-by-design | `work-item-lifecycle-closure.*`、`repository-workflow.*`、Runtime `finalize`/`close` receipt による PR/base/branch/worktree cleanup。 |
| `docs/reference/work-item-lifecycle-closure.md` | implemented-different-by-design | 英語の Rust-native close/recovery route。source `make`/Python orchestration は command 要件ではありません。 |

これは semantic/documentation parity であり、source command、JSON-wire、provider state の
compatibility ではありません。object/adopter boundary は shared Runtime、明示的な
`--repo`、分離された repository fact、Work Item、evidence、knowledge、snapshot のままです。
WI-379 後の ledger は 4,262 `generated-history`、292 `implemented-different-by-design`、
1 `implemented-equivalent`、4 `not-applicable`、45 `reference-only`、515
`deferred-next-batch`、`migrate-gap` は 0 です。

## WI-386 — reference documentation batch 19

WI-386 は pinned source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の deferred
4 文書を一つずつ比較しました。歴史/内部文書 2 件は `reference-only` のまま保持し、
Roadmap と Security Boundary の責務は現在の Rust-native documentation で表現します。
source Python、Make command、provider configuration、historical GO/NO-GO claim、未来の
roadmap milestone はコピーせず、出荷済み能力として主張しません。

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/review-final-evidence.md` | reference-only | source 固有の `make` check と歴史的 review state に束縛された生成 R11 evidence index。current `final-replacement-acceptance.md`、`ci-release-evidence.md`、repository-local Work Item evidence が新しい identity-bound truth を生成し、過去の GO/NO-GO はコピーしません。 |
| `docs/review-remediation-backlog.md` | reference-only | 内部 R0–R11 remediation backlog と Python/Make execution plan。current boundary は `repository-workflow.md`、`governance-integrity-gate.md`、この比較台帳で維持し、source plan は current authority ではありません。 |
| `docs/roadmap.md` | implemented-different-by-design | `docs/philosophy.md`、`docs/architecture.md`、`docs/capabilities.md` が mission、evidence governance、intent、human control、repository intelligence、organization-policy direction を保持します。歴史的 V1–V4 milestone と source wording は shipped capability claim ではありません。 |
| `docs/security-boundaries.md` | implemented-different-by-design | `docs/security/threat-model.md`、`docs/reference/input-trust-dataflow.md`、`docs/reference/operation-time-policy-reevaluation.md`、`docs/security/adversarial-validation.md` が content/authority separation、deterministic fail-closed、高 risk reevaluation、limitations を保持します。source classifier implementation はコピーしません。 |

これは semantic/documentation parity であり、source command、JSON-wire、provider state の
compatibility ではありません。すべての object/adopter project は shared Runtime からこの
Rust-native documentation boundary を継承しますが、repository fact、Work Item、evidence、
knowledge、snapshot は明示的な `--repo` の下で分離されます。WI-386 後の ledger は 4,262
`generated-history`、294 `implemented-different-by-design`、1 `implemented-equivalent`、4
`not-applicable`、47 `reference-only`、511 `deferred-next-batch`、`migrate-gap` は 0 です。

## WI-387 — reference documentation batch 20

WI-387 は pinned source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の security / supply-chain
文書 4 件を一つずつ比較します。責務は Rust-native security、trust-flow、release-evidence、distribution
文書で表現します。本 batch は bounded な repository-governance response と外部 control boundary を保持し、
general prompt-injection detector、signature、SBOM、provenance、provider assurance を Runtime が提供すると
主張しません。

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/security/injection-boundary.ja.md` | implemented-different-by-design | `docs/security/adversarial-validation.ja.md`、`docs/reference/input-trust-dataflow.ja.md`、`docs/reference/operation-time-policy-reevaluation.ja.md` が日本語の injection boundary、operation-time fail-closed review、外部 control 制限を保持します。 |
| `docs/security/injection-boundary.md` | implemented-different-by-design | `docs/security/adversarial-validation.md`、`docs/reference/input-trust-dataflow.md`、`docs/reference/operation-time-policy-reevaluation.md` が bounded repository-governance response を保持します。untrusted text は data のままで、source page を general detector claim としてコピーしません。 |
| `docs/security/injection-boundary.zh-CN.md` | implemented-different-by-design | `docs/security/adversarial-validation.zh-CN.md`、`docs/reference/input-trust-dataflow.zh-CN.md`、`docs/reference/operation-time-policy-reevaluation.zh-CN.md` が中国語の boundary、deterministic fail-closed handling、non-claims を保持します。 |
| `docs/security/supply-chain.md` | implemented-different-by-design | `docs/security/threat-model.md`、`docs/reference/ci-release-evidence.md`、`docs/release/distribution.md`、`docs/getting-started/security-release-verification.md` が delegated supply-chain evidence ownership と exact artifact binding を保持し、external trust root は Runtime 外に残します。 |

WI-387 後の ledger は 4,262 `generated-history`、298 `implemented-different-by-design`、1
`implemented-equivalent`、4 `not-applicable`、47 `reference-only`、507 `deferred-next-batch`、
`migrate-gap` は 0 です。すべての attach 済み object/adopter project は同じ Rust-native security / supply-chain
boundary を継承し、repository fact と evidence は明示的な `--repo` context で分離されます。

## WI-388 — reference documentation batch 21

WI-388 は pinned source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の deferred 6 文書を一つずつ比較します。
責務は Rust-native threat model、adoption、release evidence、installation、troubleshooting route で表現されています。
本 batch は分散した counterpart と evidence boundary を記録し、source command や歴史的な stability claim は copy しません。

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/security/threat-model.md` | implemented-different-by-design | `docs/security/threat-model.md`、`.zh-CN.md`、`.ja.md` が protected asset、trust boundary、fail-closed threat、external control limit を保持します。すべての malicious intention の検出や enterprise security certification は主張しません。 |
| `docs/template-adopter-stability-matrix.md` | implemented-different-by-design | `docs/reference/final-replacement-acceptance.md`、`docs/getting-started/standard-adoption-guide.md`、`docs/reference/ci-release-evidence.md`、`tests/release/adopter_acceptance.sh` が template/adoption/lifecycle/evidence-kind boundary を分担します。template-only run を external stability proof に昇格しません。 |
| `docs/troubleshooting.md` | implemented-different-by-design | 三言語 `docs/reference/troubleshooting.*` が stop state、recovery、evidence preservation、明示的な repository-bound command を提供し、compatibility-only redirect にはしません。 |
| `docs/troubleshooting/installation.ja.md` | implemented-different-by-design | `docs/getting-started/installation.ja.md`、`installation-security.ja.md`、`docs/reference/troubleshooting.ja.md` が uncertainty stop、strict Release verification、explicit attachment を保持し、source wizard command は copy しません。 |
| `docs/troubleshooting/installation.md` | implemented-different-by-design | `docs/getting-started/installation.md`、`installation-security.md`、`docs/reference/troubleshooting.md` が uncertainty stop、strict Release verification、explicit attachment を保持し、moving/older artifact を黙って選びません。 |
| `docs/troubleshooting/installation.zh-CN.md` | implemented-different-by-design | `docs/getting-started/installation.zh-CN.md`、`installation-security.zh-CN.md`、`docs/reference/troubleshooting.zh-CN.md` が中国語 recovery route、strict artifact binding、explicit repository context を保持します。 |

これは semantic/documentation parity であり、source command、JSON-wire、provider state の compatibility ではありません。
すべての attach 済み object/adopter project は shared Runtime から threat、adoption、installation、recovery boundary を継承し、
repository fact と evidence は明示的な `--repo` の下で分離されます。WI-388 後の ledger は 4,262 `generated-history`、
304 `implemented-different-by-design`、1 `implemented-equivalent`、4 `not-applicable`、47 `reference-only`、
501 `deferred-next-batch`、`migrate-gap` は 0 です。

## WI-389 — reference documentation batch 22

WI-389 は pinned source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の deferred 6 文書を一つずつ比較します。Uninstall は installed Runtime lifecycle route、upgrade は Rust-native upgrade reference に対応させます。proposal-before-write、owner confirmation、immutable Release binding、rollback、conflict stop、明示的な active recovery の境界を保ち、source installer command はコピーしません。

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/troubleshooting/uninstall.ja.md` | implemented-different-by-design | `docs/reference/installed-lifecycle.ja.md` が read-only inventory、owner confirmation、proposal と別の execution confirmation、bounded removal、receipt verification、evidence retention、Unknown 時の fail-closed recovery を保持します。 |
| `docs/troubleshooting/uninstall.md` | implemented-different-by-design | `docs/reference/installed-lifecycle.md` が read-only inventory、owner confirmation、proposal と別の execution confirmation、bounded removal、receipt verification、evidence retention、Unknown 時の fail-closed recovery を保持します。 |
| `docs/troubleshooting/uninstall.zh-CN.md` | implemented-different-by-design | `docs/reference/installed-lifecycle.zh-CN.md` が read-only inventory、owner confirmation、proposal と別の execution confirmation、bounded removal、receipt verification、evidence retention、Unknown 時の fail-closed recovery を保持します。 |
| `docs/upgrade.ja.md` | implemented-different-by-design | `docs/reference/upgrade.ja.md` が immutable Release/runtime identity、rollback-safe active configuration、conflict/downgrade stop、explicit migration、別途 review された `--upgrade-with-active` recovery を保持します。 |
| `docs/upgrade.md` | implemented-different-by-design | `docs/reference/upgrade.md` が immutable Release/runtime identity、rollback-safe active configuration、conflict/downgrade stop、explicit migration、別途 review された `--upgrade-with-active` recovery を保持します。 |
| `docs/upgrade.zh-CN.md` | implemented-different-by-design | `docs/reference/upgrade.zh-CN.md` が immutable Release/runtime identity、rollback-safe active configuration、conflict/downgrade stop、explicit migration、別途 review された `--upgrade-with-active` recovery を保持します。 |

これは semantic/documentation parity であり、source command、JSON wire、provider state compatibility ではありません。すべての attached object/adopter repository は shared Runtime から同じ uninstall、upgrade、rollback、recovery boundary を継承し、repository fact と evidence は明示的な `--repo` で隔離されます。WI-389 後の ledger は 4,262 `generated-history`、310 `implemented-different-by-design`、1 `implemented-equivalent`、4 `not-applicable`、47 `reference-only`、495 `deferred-next-batch`、`migrate-gap` はゼロです。

## WI-390 — reference Work Item style guide

WI-390 は pinned `docs/work-item-style-guide.md` を section ごとに比較します。読者向けの guidance は
三言語の Rust-native style guide と Contract/workflow reference への link で表します。本 batch は、結果を
先に書くこと、問題と境界の明示、検証可能な acceptance、人が所有する decision、必要十分な process、
実行可能な verification、documentation-before-schema の原則を保持します。

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/work-item-style-guide.md` | implemented-different-by-design | `docs/reference/work-item-style-guide.md`、`.zh-CN.md`、`.ja.md`。reference index から link し、`contract-fields` と `repository-workflow` を context とします。human-owned intent/problem/constraints/rationale、明示的 scope/non-goals、machine-checkable acceptance、実行可能な verification、proportional profile、object/adopter project 継承を保持し、source metadata、Python/Make command、installer behavior、Runtime implementation はコピーしません。 |

これは semantic/documentation parity であり、source command や JSON wire compatibility ではありません。shared Runtime は adopter project の外部に残り、各 attached repository は自分の `.ai/` と adapter から同じ reader-facing boundary を継承します。Contract、evidence、knowledge、repository identity は明示的な `--repo` で分離されます。WI-390 後の ledger は 4,262 `generated-history`、311 `implemented-different-by-design`、1 `implemented-equivalent`、4 `not-applicable`、47 `reference-only`、494 `deferred-next-batch`、`migrate-gap` は 0 です。

## WI-391 — C# adaptation example

WI-391 は pinned source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の
`examples/csharp/README.md` を section ごとに比較します。source の四つの concern（installation、.NET
quality check と coverage boundary、Contract design、guideline compliance evidence）は、三言語の
Rust-native C# adaptation page と既存の installation、Contract、verification reference で表します。

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `examples/csharp/README.md` | implemented-different-by-design | `docs/reference/csharp-adaptation.md`、`.zh-CN.md`、`.ja.md` が shared Runtime installation、Contract fields、verification route への link とともに対応します。source semantics は保ちますが、`install.sh`、`Makefile.ai.stack`、source guard/Python orchestration、legacy JSON-wire example は意図的に external または non-compatible のままです。 |

これは semantic/documentation parity であり、C# toolchain support または second-technology adopter acceptance の主張ではありません。将来の C# adopter receipt は immutable public Release と自身の repository context を使います。shared Runtime は adopter の外部に一度だけ install し、`.ai/`、Contract、evidence、project policy は明示的な `--repo` で repository-local に分離します。
WI-391 後の ledger は 4,262 `generated-history`、312 `implemented-different-by-design`、1 `implemented-equivalent`、4 `not-applicable`、47 `reference-only`、493 `deferred-next-batch`、`migrate-gap` は 0 です。

## WI-392 — Android fixture adaptation

WI-392 は pinned source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の Android fixture 4 ファイルを一つずつ比較します。Kotlin source/test の semantic は adopter-owned path と command に対応付け、fixture metadata と Gradle topology は Project Profile/Observer fact として bounded に扱います。

| Pinned reference path | Classification | Rust-native counterpart と boundary |
| --- | --- | --- |
| `examples/fixtures/android-app/app/src/main/kotlin/example/MainActivity.kt` | implemented-different-by-design | `docs/reference/android-fixture-adaptation.ja.md` が source path を明示的な Contract scope に対応付け、Kotlin 実行は provider-owned のままにします。 |
| `examples/fixtures/android-app/app/src/test/kotlin/example/MainActivityTest.kt` | implemented-different-by-design | `kotlin.test` assertion を owner-confirmed Gradle verification command に対応付けます。test file だけでは SDK/device/CI readiness を証明しません。 |
| `examples/fixtures/android-app/fixture.json` | implemented-different-by-design | Project Profile/Observer は stack/toolchain/platform/path fact を記録できますが、`installerStack` は Runtime install contract ではなく platform label は evidence ではありません。 |
| `examples/fixtures/android-app/settings.gradle.kts` | implemented-different-by-design | Gradle repository/module topology を bounded context とし、dependency、SDK、credential、network、hosted-CI readiness は evidence まで Unknown です。 |

これは semantic/documentation parity であり、Android toolchain support、build execution、source JSON-wire compatibility ではありません。Install は adopter 外部の immutable shared Runtime 一つと明示的な `attach --repo` を使い、fixture の Gradle file、SDK install、installer behavior はコピーしません。WI-392 後の ledger は 4,262 `generated-history`、316 `implemented-different-by-design`、1 `implemented-equivalent`、4 `not-applicable`、47 `reference-only`、489 `deferred-next-batch`、`migrate-gap` は 0 です。

## WI-393 — Flutter fixture adaptation

WI-393 は pinned source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の Flutter fixture 4 ファイルを一つずつ比較します。Dart source/test の semantic は adopter-owned path と command に対応付け、fixture と package metadata は Project Profile/Observer fact として bounded に扱います。

| Pinned reference path | Classification | Rust-native counterpart と boundary |
| --- | --- | --- |
| `examples/fixtures/flutter-app/fixture.json` | implemented-different-by-design | `docs/reference/flutter-fixture-adaptation.ja.md` が project type、stack、toolchain、platform、safe/test path を bounded な Profile/Contract fact に対応付けます。`installerStack` は Runtime install contract ではありません。 |
| `examples/fixtures/flutter-app/lib/main.dart` | implemented-different-by-design | `greeting()` の source path は adopter の Contract scope です。Dart 実行は owner/provider の責任であり、Runtime は推測しません。 |
| `examples/fixtures/flutter-app/pubspec.yaml` | implemented-different-by-design | package name と Dart SDK range は観測可能な metadata です。SDK、dependency、network、lockfile readiness は evidence まで Unknown です。 |
| `examples/fixtures/flutter-app/test/widget_test.dart` | implemented-different-by-design | `flutter_test` assertion を owner-confirmed provider command に対応付けます。file だけでは SDK、platform runner、plugin、hosted CI readiness を証明しません。 |

これは semantic/documentation parity であり、Flutter toolchain support、build execution、source JSON-wire compatibility ではありません。Install は adopter 外部の immutable shared Runtime 一つと明示的な `attach --repo` を使い、Flutter SDK/package install と reference installer implementation はコピーしません。WI-393 後の ledger は 4,262 `generated-history`、320 `implemented-different-by-design`、1 `implemented-equivalent`、4 `not-applicable`、47 `reference-only`、485 `deferred-next-batch`、`migrate-gap` は 0 です。

## WI-394 — iOS Swift Package fixture adaptation

WI-394 は pinned source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の iOS Swift Package fixture 4 ファイルを一つずつ比較します。Swift Package topology、source、XCTest の semantic は adopter-owned path と command に対応付け、fixture metadata は Project Profile/Observer fact として bounded に扱います。

| Pinned reference path | Classification | Rust-native counterpart と boundary |
| --- | --- | --- |
| `examples/fixtures/ios-swift-package/Package.swift` | implemented-different-by-design | SwiftPM product/target topology は adopter/provider-owned build metadata であり、Runtime は SDK/Xcode readiness を推測しません。 |
| `examples/fixtures/ios-swift-package/Sources/AppCore/AppCore.swift` | implemented-different-by-design | `greeting()` source path は adopter の Contract scope で、Swift 実行は provider-owned です。 |
| `examples/fixtures/ios-swift-package/Tests/AppCoreTests/AppCoreTests.swift` | implemented-different-by-design | XCTest assertion を owner-confirmed `swift test` または Xcode command に対応付けます。file だけでは SDK、simulator、signing、hosted CI readiness を証明しません。 |
| `examples/fixtures/ios-swift-package/fixture.json` | implemented-different-by-design | Project Profile/Observer は package/toolchain/platform/path fact を記録できます。`installerStack` と `macos` は metadata であり shared Runtime install/execution evidence ではありません。 |

これは semantic/documentation parity であり、Apple toolchain support、build execution、source JSON-wire compatibility ではありません。Install は adopter 外部の immutable shared Runtime 一つと明示的な `attach --repo` を使い、SwiftPM/Xcode install、SDK 選択、source installer behavior はコピーしません。WI-394 後の ledger は 4,262 `generated-history`、324 `implemented-different-by-design`、1 `implemented-equivalent`、4 `not-applicable`、47 `reference-only`、481 `deferred-next-batch`、`migrate-gap` は 0 です。
