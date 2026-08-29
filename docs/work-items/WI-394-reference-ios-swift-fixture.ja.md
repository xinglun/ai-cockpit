---
workItemId: WI-394-reference-ios-swift-fixture
title: "Reference iOS Swift Package fixture 適応"
author: AI Cockpit maintainers
description: "固定 iOS Swift Package fixture の意味をファイル単位で対応付け、shared Runtime の install boundary を示す。"
type: documentation
audience:
  - adopter
  - contributor
  - maintainer
authority: canonical
status: implemented
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
lastVerifiedBy: WI-394-reference-ios-swift-fixture
terminalArchive: .ai/work-items/archive/WI-394-reference-ios-swift-fixture.contract.json
terminalVerification: .ai/evidence/WI-394-reference-ios-swift-fixture.verification.json
terminalFinalization: .ai/decisions/WI-394-reference-ios-swift-fixture.finalize.json
terminalDecision: .ai/decisions/WI-394-reference-ios-swift-fixture.close.json
---

# WI-394 — Reference iOS Swift Package fixture 適応

## Intent

固定 iOS Swift Package fixture 4 ファイルを一つずつ比較し、Swift/Xcode の install/build
実装をコピーせず Rust-native/adopter mapping を記録します。

## Scope

- `examples/fixtures/ios-swift-package/Package.swift`
- `examples/fixtures/ios-swift-package/Sources/AppCore/AppCore.swift`
- `examples/fixtures/ios-swift-package/Tests/AppCoreTests/AppCoreTests.swift`
- `examples/fixtures/ios-swift-package/fixture.json`
- 三言語の iOS Swift Package adaptation、reference comparison、parity、index、Work Item record

## Acceptance

1. 各 source file に個別の semantic mapping または bounded non-applicability がある。
2. Swift/Xcode check は adopter/provider-owned であり、SDK、simulator、signing、network、
   CI の不足は Unknown のままであることを説明する。
3. Install は adopter 外部の immutable shared Runtime と明示的な `attach --repo` とし、
   source installer/build/wire artifact をコピーしない。
4. Inventory、parity、link、三言語 record が `e5acb677` に bind する。
5. Installed Runtime が documentation と conformance check を検証する。

## Evidence boundary

この WI は semantic/documentation parity のみを証明し、Apple/Swift toolchain support や
post-release iOS adopter acceptance は証明しません。
