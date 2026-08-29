---
author: AI Cockpit maintainers
title: "Flutter fixture 適応"
description: "固定 Flutter fixture の意味を Rust-native に対応付け、installer/SDK 実装をコピーしないことを示す。"
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

# Flutter fixture 適応

このページは pinned reference fixture `examples/fixtures/flutter-app/` の 4 ファイルを
一つずつ比較します。Flutter adopter に有用な意味は保持しますが、Flutter SDK の
サポートを約束するものではなく、reference installer、Make/Python orchestration、guard
file、legacy JSON wire shape はコピーしません。

## ファイルごとの対応

| Pinned source file | Source fact | Rust-native counterpart と境界 |
| --- | --- | --- |
| `fixture.json` | Flutter application、Flutter/Dart toolchain、Linux/macOS/Windows、safe/test path を宣言する。 | Project Profile/Observer は fact または candidate fact として記録できます。`installerStack` は adopter 側の説明であり共有 Runtime の install contract ではありません。platform 名は実行 evidence ではなく、path は人の確認後に Contract scope/verification input になります。 |
| `lib/main.dart` | 小さな `greeting()` が安定して `hello` を返す。 | path は adopter 所有の source として扱います。Work Item は intent、scope、owner-confirmed command を記録し、Runtime は Dart の意味を実行・推測しません。 |
| `pubspec.yaml` | fixture 名と Dart SDK range を宣言し、package dependency は宣言しない。 | package metadata は Observer が報告できます。SDK、依存解決、network、lockfile の状態は provider evidence まで Unknown です。Runtime は Flutter を install せず `pubspec.yaml` も書き換えません。 |
| `test/widget_test.dart` | `flutter_test` で greeting を検証する。 | adopter/provider の test capability です。owner が `flutter test` を確認し、`verify --repo` が結果と identity を記録します。file だけでは SDK、platform runner、plugin、hosted CI の readiness を証明しません。 |

## Installation は意図的に異なる

fixture の `installerStack` と Dart metadata は AI Cockpit の install recipe ではありません。
各 adopter の外側に immutable な共有 Runtime を一つだけ install し、明示的に repository
を attach します。

```bash
repo=/path/to/flutter-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

attach は repository 固有の `.ai/`、Contract、evidence、knowledge、adapter state を
管理します。Flutter fixture のコピー、Flutter/Dart の install、package download、
global Runtime state への binding は行いません。以後の command には同じ `--repo` を
明示し、adopter ごとに repository identity と evidence chain を分離します。

実際の adopter route では、owner と provider が正確な Flutter command を確認してから
verify します。local `flutter test` だけでは provider/release/enterprise evidence には
なりません。

## 継承するもの、しないもの

attach された Flutter project は shared Runtime の Contract validation、fail-closed の
Unknown 処理、evidence identity、lifecycle、human Outcome rule を継承します。一方で
fixture の SDK、package cache、platform runner、installer variable、Dart source や、
Flutter check 実行済みという主張は継承しません。scope、profile、snapshot、evidence は
明示的な repository context 内に分離されます。

これは semantic/documentation parity であり、source command、build tool、JSON-wire の
互換性ではありません。実際の Flutter adopter acceptance は immutable public Runtime
artifact を使う別の post-release test です。

[Reference index](README.ja.md) | [English](flutter-fixture-adaptation.md) | [中文](flutter-fixture-adaptation.zh-CN.md)
