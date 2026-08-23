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
lastVerifiedBy: WI-224-ci-reference-parity
---

# CI Runtime verification shadow

WI-224 は repository CI route を明示的な policy にします。`quality_route.py` は changed
paths、Contract risk、workflow stage から `light`、`standard`、`strict` を選択します。
unknown path、release-owned path、high risk、merge、release stage は `strict` へ
escalate します。型付き route receipt は Git base/head、changed paths、Contract の
path/digest、manifest byte digest、選択理由、順序付き gate ID を bind します。
`run_repository_gates.py` は repository facts から receipt を再計算し、canonical
manifest に保存された command だけを実行します。任意 command override はありません。

profile は累積です。`light` は docs と governance-policy regression、`standard` は
Cargo fmt/Clippy/package gates、immutable Runtime shadow、source conformance を追加し、
`strict` は release、workflow、performance、adopter、source-archive gates を追加します。
Pull request は path/risk route を使用し、merge push の stage floor は strict です。
release source quality は常に `strict` を明示要求し、route receipt と gate report を
upload します。

`standard` と `strict` では、独立した execution shadow が public immutable
`v0.2.28` Runtime を download し、platform archive/binary digest を検証して、repository
の canonical profile で verify を実行します。receipt は tag、version、archive digest、
binary digest、platform、download source、Runtime result を bind します。source build、
workspace binary、任意の `--command` 代替、unpinned artifact、digest mismatch、malformed
output は拒否されます。

これは repository CI/release layer の policy です。Runtime-global T0–T3 route、
affected-graph completeness、cross-Work-Item physical execution、generic CLI
`verify --command` semantics は主張しません。WI-224 は `crates/**` を authorize しない
ため、これらの Runtime change は明示的に deferred です。shadow は execution identity
check であり、選択された manifest gates や provider/enterprise assurance の代替では
ありません。
