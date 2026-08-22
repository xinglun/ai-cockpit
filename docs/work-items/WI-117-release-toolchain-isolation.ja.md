---
author: AI Cockpit maintainers
title: "WI-117 Release adopter toolchain isolation"
description: "N-1 acceptance を既存の Rust toolchain に bind し、暗黙の download を拒否する。"
audience:
  - maintainer
  - release-engineer
status: implemented
authority: canonical
lastVerifiedBy: release-adopter-toolchain-regression
capabilityClaims:
  - bounded_release_acceptance
  - isolated_toolchain_identity
---

# WI-117：Release adopter toolchain isolation

## Goal

隔離した HOME、TMPDIR、CARGO_HOME を使う post-release adopter と N-1
acceptance を deterministic にする。

## Scope

N-1 harness は host の Rustup home と active toolchain を解決し、隔離した
fixture command に明示的に渡す。どちらかの identity が取得できない場合は、
暗黙の network toolchain download を拒否する。Runtime Protocol の意味論と
global Rust installation は対象外である。

## Acceptance

- 環境変数がない場合、`RUSTUP_HOME` は `rustup show home` に fallback する。
- active toolchain から `RUSTUP_TOOLCHAIN` を解決し、隔離した Cargo/Runtime 呼び出しへ渡す。
- toolchain identity がない場合、無制限の fixture を作る前に fail closed する。
- cleanup evidence は acceptance truth と分離し、検証済み run root だけを削除する。
- 英中日三言語の release documentation がこの境界を説明する。

## Verification

```text
bash tests/release/adopter_upgrade_acceptance_test.sh
bash tests/docs/documentation_acceptance.sh
git diff --check
```

## Outcome

Status: **Implemented; toolchain identity と bounded cleanup を明示した。**
