---
author: AI Cockpit maintainers
title: "WI-828 — governed v0.2.92 release"
description: "最新の review 済み main から ai-cockpit v0.2.92 を準備し公開する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-828-release-v0-2-92
lastVerifiedBy: WI-828-release-v0-2-92
---

[English](WI-828-release-v0-2-92.md) · [简体中文](WI-828-release-v0-2-92.zh-CN.md)

# WI-828 — governed v0.2.92 release

## Intent と境界

WI-828 は workspace と current release projection を v0.2.92 に揃え、review 済み main revision を
immutable Release として公開します。candidate と公開 acceptance は isolated root の identity-bound
artifact を消費しなければなりません。

Historical Work Item migration、v0.2.91 の変更、無関係な performance work、user-global configuration
はこの Work Item の範囲外です。

## Acceptance

- Cargo metadata、lockfile、current release documentation、reference comparison metadata が v0.2.92
  を一貫して識別する。
- hosted source verification、build、candidate install、N-1 upgrade が pass し、identity-bound receipt を保持する。
- 公開 v0.2.92 artifact が fresh install、N-1 upgrade、version consistency、cleanup を pass する。
- failure recovery は有効な phase を再利用し、rebuild/re-publish を繰り返さない。finalization、archive、close、
  documentation promotion が確認される。

## Verification

- `bash tests/release/version_consistency.sh --repo <repo>`
- `python3 tests/docs/reference_comparison_metadata_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh <repo>`
- `cargo fmt --all --check`
- `cargo test --locked --workspace --all-targets`
- hosted release preflight、candidate acceptance、公開 artifact acceptance、N-1 upgrade、close-only recovery。

## Evidence policy

Release tag、provider asset、manifest、公開 acceptance receipt は immutable な外部事実です。Runtime の
Contract、verification、archive、finalization、close record はインストール済み Runtime が生成し、local
build や moving branch で置き換えません。
