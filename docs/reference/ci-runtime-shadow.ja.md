---
author: AI Cockpit maintainers
title: CI Runtime verification shadow
description: 型付き repository quality route と immutable public Runtime execution shadow。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-804-release-route-ordering
---

# CI Runtime verification shadow

WI-224 は repository CI route を明示的な policy にします。`quality_route.py` は changed
paths、Contract risk、workflow stage から `light`、`standard`、`strict` を選択します。
unknown path、release-owned path、high risk、merge、release stage は `strict` へ
escalate します。型付き route receipt は Git base/head、changed paths、Contract の
path/digest、manifest byte digest、選択理由、順序付き gate ID を bind します。
`run_repository_gates.py` は repository facts から receipt を再計算し、canonical
manifest に保存された command だけを実行します。任意 command override はありません。

Runtime shadow は Contract に bind されます。`standard` または `strict` の pull request
は、initial route が active Contract を一つ解決した場合だけ shadow を実行して upload
します。外部 resource のない archived PR は通常の repository gate を実行しますが、
現在の Contract がないため immutable Runtime が Work Item verification evidence を生成
できず、この execution-only shadow は skip します。歴史的に external resource に bind
された PR は別ルートです。pull-request stage で read-only Rust Contract gate が正確な
archived Contract、archive manifest、identity、適用範囲を検証し、通常の no-Contract
route へ fallback してはいけません。identity または manifest の不一致は fail closed
です。これらの明示的な route は選択された repository gate を弱めず、missing evidence
を pass にもしません。

profile は累積です。`light` は docs と governance-policy regression、`standard` は
Cargo fmt/Clippy/package gates、immutable Runtime shadow、source conformance を追加し、
`strict` は release、workflow、performance、adopter、source-archive gates を追加します。
Pull request は path/risk route を使用し、merge push の stage floor は strict です。
release source quality は常に `strict` を明示要求し、route receipt と gate report を
upload します。

CI は境界付きの route plan を 2 回使用します。initial receipt は Runtime shadow が
必要かを決め、`light` は shadow を skip します。`standard` または `strict` は shadow
実行後、同じ immutable Git base/head と `.ai/evidence/reuse/**` を含む Runtime の
repository-local write から final receipt を再計算します。gate runner が consume する
のは final receipt だけで、両 receipt は診断用に保持されます。final profile が
non-light の場合は workspace package coverage が必須で、regular receipt file が存在する
場合だけ upload します。正当な `light` route はその file を要求も upload もしません。

`standard` と `strict` では、独立した execution shadow が public immutable
`v0.2.28` Runtime を download し、platform archive/binary digest を検証して、固定した
`git version` execution smoke を実行します。receipt は tag、version、archive digest、
binary digest、platform、download source、smoke command、Runtime result を bind します。
repository の canonical profile は引き続き必須ですが、この immutable Runtime identity
probe で重複実行せず、選択された quality-gate owner が一度だけ実行します。source build、
workspace binary、command substitution、unpinned artifact、digest mismatch、malformed
output は拒否されます。

これは repository CI/release layer の policy です。Runtime-global T0–T3 route、
affected-graph completeness、cross-Work-Item physical execution、generic CLI
`verify --command` semantics は主張しません。WI-224 は `crates/**` を authorize しない
ため、これらの Runtime change は明示的に deferred です。shadow は execution identity
check であり、選択された manifest gates や provider/enterprise assurance の代替では
ありません。
