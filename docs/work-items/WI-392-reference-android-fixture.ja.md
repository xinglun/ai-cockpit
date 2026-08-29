---
workItemId: WI-392-reference-android-fixture
title: "Reference Android fixture adaptation"
author: AI Cockpit maintainers
description: "固定 Android fixture を file ごとに semantic mapping し、shared Runtime の install boundary を明示します。"
type: documentation
audience:
  - adopter
  - contributor
  - maintainer
authority: canonical
status: implemented
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
lastVerifiedBy: WI-392-reference-android-fixture
terminalArchive: .ai/work-items/archive/WI-392-reference-android-fixture.contract.json
terminalVerification: .ai/evidence/WI-392-reference-android-fixture.verification.json
terminalFinalization: .ai/decisions/WI-392-reference-android-fixture.finalize.53b26b80706cab70f1fb4c8c3772cbf92475c25fa11d5141c906ccafa9566fea.json
terminalDecision: .ai/decisions/WI-392-reference-android-fixture.close.json
---

# WI-392 — Reference Android fixture adaptation

## Intent

固定した Android fixture 4 ファイルを一つずつ比較し、Android の install/build 実装を hard-copy せず Rust-native/adopter mapping を記録します。

## Scope

- `examples/fixtures/android-app/app/src/main/kotlin/example/MainActivity.kt`
- `examples/fixtures/android-app/app/src/test/kotlin/example/MainActivityTest.kt`
- `examples/fixtures/android-app/fixture.json`
- `examples/fixtures/android-app/settings.gradle.kts`
- 三言語の Android adaptation、reference comparison、parity、index、Work Item record

## Acceptance

1. 各 source file に個別の semantic mapping または bounded non-applicability を記録します。
2. Android/Gradle check は adopter/provider-owned であり、SDK、device、signing、network、CI の不足 fact は Unknown のままと説明します。
3. Install は adopter 外部の immutable shared Runtime 一つと明示的な `attach --repo` とし、source installer/build/wire artifact は copy しません。
4. Inventory、parity、link、三言語 record が `e5acb677` を bind します。
5. Installed Runtime で documentation と conformance check を verify します。

## Evidence boundary

これは semantic/documentation parity の証拠だけであり、Android toolchain support や post-release Android adopter acceptance の証明ではありません。
