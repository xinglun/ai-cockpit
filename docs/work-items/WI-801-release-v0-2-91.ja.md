---
author: AI Cockpit maintainers
title: "WI-801 — governed v0.2.91 release"
description: "WI-799 の root-cause repair 後に ai-cockpit v0.2.91 を準備し公開する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization-root-cause-repair-and-release
workItemId: WI-801-release-v0-2-91
lastVerifiedBy: WI-801-release-v0-2-91
---

[English](WI-801-release-v0-2-91.md) · [简体中文](WI-801-release-v0-2-91.zh-CN.md)

# WI-801 — governed v0.2.91 release

## Intent と境界

WI-801 は Cargo workspace と lockfile を v0.2.91 に揃え、三言語の governance
projection を登録し、review 済み main revision を immutable Release として公開します。
release workflow は candidate artifact とダウンロード済みの公開 artifact を使って、
install と N-1 upgrade の acceptance を行わなければなりません。

この Work Item は Runtime lifecycle の一境界だけを変更します。verification 後の
governance refresh が検証を実行した Runtime identity に bind され続けるようにします。
その他の Runtime behavior、release workflow semantics、historical records、既存の
Release/tag、無関係な source feature は変更しません。

## Acceptance

- `Cargo.toml` と `Cargo.lock` が v0.2.91 を一貫して識別し、無関係な dependency 変更がない。
- WI-801 の三言語ページと parity row が verification 前に登録され、意味的に整合する。
- hosted checks が pass した後、review 済み main に tag を付けて v0.2.91 を公開する。
- candidate の fresh install/N-1 upgrade と、公開 artifact の fresh install/N-1 upgrade、
  version consistency、release close が immutable artifact と isolation cleanup を証明する。

## Verification

- `cargo test --locked --workspace`
- `bash tests/release/version_consistency.sh --repo <repo>`
- `bash tests/release/version_consistency_test.sh`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `cargo test -p cockpit-repository --test lifecycle_order runtime_bound_verification_keeps_governance_bound_to_current_runtime -- --exact`
- hosted release preflight、source quality、candidate acceptance、公開 artifact acceptance、
  N-1 upgrade acceptance、release close。

## Evidence policy

Release tag、Release asset、workflow receipt、公開 adopter receipt は immutable な外部事実です。
Runtime-owned の Contract、Summary、Outcome、verification、archive、finalization、close
record はインストール済み Runtime が生成します。moving branch、source checkout、workspace
binary を公開 artifact の代わりにはしません。
