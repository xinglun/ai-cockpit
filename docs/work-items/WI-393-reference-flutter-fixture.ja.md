---
workItemId: WI-393-reference-flutter-fixture
title: "Reference Flutter fixture 適応"
author: AI Cockpit maintainers
description: "固定 Flutter fixture の意味をファイル単位で対応付け、shared Runtime の install boundary を示す。"
type: documentation
audience:
  - adopter
  - contributor
  - maintainer
authority: canonical
status: in_progress
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
lastVerifiedBy: WI-393-reference-flutter-fixture
---

# WI-393 — Reference Flutter fixture 適応

## Intent

固定 Flutter fixture 4 ファイルを一つずつ比較し、Flutter の install/SDK 実装を
コピーせず Rust-native/adopter mapping を記録します。

## Scope

- `examples/fixtures/flutter-app/fixture.json`
- `examples/fixtures/flutter-app/lib/main.dart`
- `examples/fixtures/flutter-app/pubspec.yaml`
- `examples/fixtures/flutter-app/test/widget_test.dart`
- 三言語の Flutter adaptation、reference comparison、parity、index、Work Item record

## Acceptance

1. 各 source file に個別の semantic mapping または bounded non-applicability がある。
2. Flutter/Dart check は adopter/provider-owned であり、SDK、dependency、network、
   platform、plugin、CI の不足は Unknown のままであることを説明する。
3. Install は adopter 外部の immutable shared Runtime と明示的な `attach --repo` とし、
   source installer/build/wire artifact をコピーしない。
4. Inventory、parity、link、三言語 record が `e5acb677` に bind する。
5. Installed Runtime が documentation と conformance check を検証する。

## Evidence boundary

この WI は semantic/documentation parity のみを証明し、Flutter toolchain support や
post-release Flutter adopter acceptance は証明しません。
