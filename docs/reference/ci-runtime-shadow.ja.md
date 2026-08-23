---
author: AI Cockpit maintainers
title: CI Runtime verification shadow
description: 既存 Cargo gate を残したまま immutable な公開 Runtime を使う Phase 1 CI convergence。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-145-ci-runtime-shadow
---

# CI Runtime verification shadow

WI-145 は CI convergence の Phase 1 を定義します。quality job は公開済みで
previous stable である immutable な `v0.2.15` Linux Runtime を download し、archive と binary digest を
検証してから checkout に対して `ai-cockpit verify` を実行します。receipt には tag、
version、archive digest、binary digest、platform、download source、Runtime verify
結果を記録します。

既存の Cargo `fmt`、`clippy`、package test は同じ job に残し、独立した shadow
comparison とします。Runtime shadow が成功してもこれらを置き換えたり弱めたりは
せず、この段階で Runtime と Cargo の結果同値や provider/enterprise assurance を
主張しません。

convergence の境界は段階的です。

1. **Phase 1（現在）：** immutable Runtime verify と既存 Cargo checks。
2. **Phase 2（将来）：** Runtime/Cargo の比較可能な結果を継続収集し、安定した
   convergence を証明する。
3. **Phase 3（将来）：** Phase 2 の Evidence と review 済み移行判断の後だけ、重複
   YAML policy を削除する。

shadow lane は source build、workspace binary、未固定 release artifact、
archive/binary digest 不一致、malformed Runtime output を fail closed します。
現在の installation baseline は新しい Release（現在は `v0.2.23`）へ進められますが、公開前の shadow pin は変更しません。
Release が公開され immutable な archive/binary identity が記録された後にだけ pin を進め、tag workflow が未公開 artifact に依存しないようにします。
