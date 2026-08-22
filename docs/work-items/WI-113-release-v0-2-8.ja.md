---
author: AI Cockpit maintainers
title: "WI-113 v0.2.8 public Release と self-adopter acceptance"
description: "merge 済み Runtime を公開し、immutable artifact を install して本 repository 自身の governance を検証する。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: release-adopter-acceptance
capabilityClaims:
  - public_release
  - self_adopter_acceptance
---

# WI-113: v0.2.8 public Release と self-adopter acceptance

## 目的

self-governed な main から v0.2.8 を公開し、immutable な public binary を install して、
source や workspace binary に fallback せず本 repository を governance できることを証明します。

## 範囲

workspace version metadata と current release/version documentation を更新し、source と
supply-chain gate を実行して v0.2.8 tag を push します。download した artifact を install し、
post-release adopter と N-1 acceptance evidence を記録します。Historical Work Item record と
external Homebrew tap state は書き換えません。

## 受入れ

- すべての workspace package と `Cargo.lock` が 0.2.8 を示します。
- current の English、中文、日本語 release、operations、versioning、parity page が v0.2.8 を示し、
  v0.2.7 は明示的な N-1 input または historical record としてだけ残します。
- hosted release、artifact、manifest、checksum、provenance、Node24 policy gate が通過します。
- public archive と binary SHA-256 を Runtime identity evidence に記録し、adopter acceptance に
  source/workspace binary を使用しません。
- install 済み v0.2.8 Runtime が `changedPaths=[]`、`COMPATIBLE`、`doctor=ok`、
  `agent doctor=VERIFIED`、`runtimeCodeInRepository=false` を示します。
- public adopter、N-1 upgrade、isolation cleanup、evidence reuse、`first-adopter-smoke=not_ready`
  の assertion が通過します。
- release closure 前に self-governed Work Item lifecycle と English/中文/日本語の可視 Outcome handoff
  を記録します。

## 検証

```text
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-targets --all-features -- --test-threads=1
bash tests/docs/documentation_acceptance.sh
bash tests/release/version_consistency.sh --repo .
bash tests/release/adopter_acceptance.sh --repository xinglun/ai-cockpit --tag v0.2.8
bash tests/release/adopter_upgrade_acceptance.sh --repository xinglun/ai-cockpit --from-tag v0.2.7 --to-tag v0.2.8
```

Post-release harness が public artifact identity と isolation の authority です。失敗時は
`releasePublished: true` を記録し、Release truth を書き戻しません。

## Outcome

Status: **implementation と release preparation は完了。public publication と download artifact
acceptance が release-bound step として残っています。**
