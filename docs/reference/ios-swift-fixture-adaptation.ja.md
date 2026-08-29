---
author: AI Cockpit maintainers
title: "iOS Swift Package fixture 適応"
description: "固定 iOS Swift Package fixture の意味を Rust-native に対応付け、installer/Xcode 実装をコピーしないことを示す。"
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

# iOS Swift Package fixture 適応

このページは pinned reference fixture `examples/fixtures/ios-swift-package/` の 4 ファイルを
一つずつ比較します。Swift Package の意味は保持しますが、Apple platform/Xcode support を約束せず、
reference installer、Make/Python orchestration、guard file、legacy JSON wire shape はコピーしません。

## ファイルごとの対応

| Pinned source file | Source fact | Rust-native counterpart と境界 |
| --- | --- | --- |
| `Package.swift` | Swift tools 5.9、`AppCore` library product、`AppCoreTests` test target と `AppCore` dependency を宣言する。 | package topology は adopter/provider-owned build metadata です。Work Item が path と owner-confirmed command を記録し、Runtime は SwiftPM/Xcode を install せず Apple SDK readiness も推測しません。 |
| `Sources/AppCore/AppCore.swift` | public `greeting()` が `hello` を返す。 | path は adopter の source scope です。Contract validation と evidence binding は継承しますが、Swift 実行は provider の責任です。 |
| `Tests/AppCoreTests/AppCoreTests.swift` | XCTest が `AppCore` を import し greeting を検証する。 | adopter/provider の test capability です。owner が `swift test` または Xcode scheme を確認し、`verify --repo` が command と結果を記録します。file だけでは macOS/iOS SDK、simulator、signing、hosted CI readiness を証明しません。 |
| `fixture.json` | iOS Swift package、Swift Package stack、Swift installer metadata、macOS platform、safe/test path を宣言する。 | Project Profile/Observer は fact または candidate fact として記録できます。`installerStack` は adopter の説明であり shared Runtime install contract ではありません。`macos` は platform label であって execution evidence ではありません。 |

## Installation は意図的に異なる

fixture の Swift metadata は AI Cockpit の install recipe ではありません。adopter の外側に immutable
shared Runtime を一つだけ install し、明示的に repository を attach します。

```bash
repo=/path/to/swift-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

attach は repository 固有の `.ai/`、Contract、evidence、knowledge、adapter state を管理します。
Swift fixture のコピー、SwiftPM/Xcode の install、Apple SDK の選択、global Runtime state への binding
は行いません。以後の command には同じ `--repo` を明示し、adopter ごとに repository identity と
evidence chain を分離します。

実際の adopter route では owner/provider が `swift test` または Xcode command を確認してから verify
します。local result だけでは provider/release/enterprise evidence にはなりません。

## 継承するもの、しないもの

attach された Swift project は shared Runtime の Contract validation、fail-closed Unknown 処理、
evidence identity、lifecycle、human Outcome rule を継承します。一方で fixture の Swift toolchain、
Xcode state、Apple SDK、simulator、signing credential、installer variable や test 実行済みという主張は
継承しません。scope、profile、snapshot、evidence は明示的な repository context 内に分離されます。

これは semantic/documentation parity であり、source command、build tool、JSON-wire 互換ではありません。
実際の iOS/Swift adopter acceptance は immutable public Runtime artifact を使う別の post-release test です。

[Reference index](README.ja.md) | [English](ios-swift-fixture-adaptation.md) | [中文](ios-swift-fixture-adaptation.zh-CN.md)
