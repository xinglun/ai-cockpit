---
author: AI Cockpit maintainers
title: "Android fixture adaptation"
description: "固定した Android fixture を file ごとに Rust-native へ対応付け、installer と build 実装はコピーしません。"
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
capabilityClaims:
  - semantic_reference_mapping
lastVerifiedBy: documentation-acceptance
---

# Android fixture adaptation

このページは pinned reference fixture `examples/fixtures/android-app/` の 4 ファイルを一つずつ比較します。
Android adopter に有用な semantic は保持しますが、Android tooling support を約束するものではなく、reference
installer、Make/Python orchestration、guard file、legacy JSON wire shape はコピーしません。

## File-by-file mapping

| Pinned source file | Source fact | Rust-native counterpart と boundary |
| --- | --- | --- |
| `app/src/main/kotlin/example/MainActivity.kt` | 小さな Kotlin `greeting()` が stable value を返す。 | Repository-owned source として扱い、Work Item の `scope`/`outOfScope` で決定し owner-approved command を verify します。Runtime は Kotlin semantic を実行・推論しません。 |
| `app/src/test/kotlin/example/MainActivityTest.kt` | `kotlin.test` が greeting を assert する。 | Adopter/provider の test capability です。Owner が `./gradlew test` など exact command を confirm し、`verify --repo` が結果と identity を記録します。file だけでは SDK、emulator、signing、hosted CI の準備を証明しません。 |
| `fixture.json` | `projectType`、`stack`、`installerStack`、toolchain、platform、safe/test path を宣言する。 | Project Profile/Observer は fact または candidate fact として記録できます。`installerStack` は adopter を表し、shared Runtime の install contract ではありません。platform 名は execution evidence ではありません。Safe/test path は human confirmation 後に Contract scope と verification input になります。 |
| `settings.gradle.kts` | Gradle repository、root name、`:app` inclusion を設定する。 | Build topology の evidence に限ります。Gradle/Android dependency download、SDK/device readiness、credential、network、CI は owner-approved provider check が evidence を出すまで Unknown です。 |

## Install は意図的に異なる

Reference fixture は example project であり、stack metadata は AI Cockpit の install recipe ではありません。Target
model は adopter の外側に immutable shared Runtime を一つだけ install し、repository を明示的に attach します。

```bash
repo=/path/to/android-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

Attach は repository 固有の `.ai/`、Contract、evidence、knowledge、adapter state を所有します。Android fixture の
copy、Gradle/Android SDK の install、global Runtime state への binding は行いません。以後の command は同じ明示的な
`--repo` を必ず付けます。別 adopter は別の repository identity と evidence chain を持ちます。

Adopter route は [Android profile start](../getting-started/examples/android.ja.md) を参照してください。read-only
candidate を propose し、owner が exact Gradle command を確認してから verify します。Local result は provider、release、
enterprise evidence ではありません。

## Inheritance

Attach 済み Android project は shared Runtime の Contract validation、Unknown 時の fail-closed、evidence identity、
lifecycle、人間向け Outcome rule を継承します。Reference fixture の installer variable、Gradle file、Kotlin source、
「Android check が実行済み」という claim は継承しません。各 project は自身の repository context 内で scope、profile、
snapshot、evidence を独立して保持します。

これは semantic/documentation parity であり、source command、build tool、JSON wire compatibility ではありません。Real
Android adopter acceptance は immutable public Runtime artifact を使う別の post-release test です。

[Reference index](README.ja.md) | [English](android-fixture-adaptation.md) | [中文](android-fixture-adaptation.zh-CN.md)
