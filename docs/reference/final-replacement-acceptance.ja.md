---
author: AI Cockpit maintainers
title: "最終置換 acceptance"
description: "Rust Runtime が reference runtime を置き換えることを証明する再現可能な境界。"
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-82
capabilityClaims:
  - final_replacement_acceptance
---

# 最終置換 acceptance

`tests/conformance/final_replacement_acceptance.sh --repo <repository>` を実行すると、
インストール済み Runtime の version と binary digest、repository identity、固定した
reference commit、各 gate の結果、`acceptance.json`、`SHA256SUMS` を含む監査可能な
acceptance directory が生成されます。

gate は、固定 reference conformance、adversarial negative corpus、負例を拒否する
performance regression、release workflow policy、人間向け Outcome、実行可能な
reference oracle、V1 Runtime 実装をコピーしていないことを示す tracked-path 検査です。

この script は fail-closed で、`cargo build`、`cargo run`、workspace binary、ローカル
`target/` fallback を使用しません。緑の receipt はこの acceptance boundary の通過だけを
示し、merge や publish を承認するものではありません。
