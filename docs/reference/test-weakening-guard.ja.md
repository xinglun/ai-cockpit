---
author: AI Cockpit maintainers
title: Test weakening signals
description: Rust Runtime で snapshot から検証強度の低下を検出する境界。
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: translation
canonical: docs/reference/test-weakening-guard.md
lastVerifiedBy: WI-378-reference-documentation-batch-17
capabilityClaims:
  - test_weakening_detection
---

# Test weakening signals

[English](test-weakening-guard.md) · [简体中文](test-weakening-guard.zh-CN.md) · [日本語](test-weakening-guard.ja.md)

Rust Runtime は preflight と Contract quality gate で、宣言された base と現在の repository snapshot から test/coverage weakening signal を derive します。Agent の prose は evidence ではなく、signal が空でも完全な semantic coverage の証明にはなりません。

## Signal boundary

Detector は repository-relative な tracked change、たとえば test の削除、skip/disable marker の追加、negative/security regression の削除、required check の non-blocking 化、coverage requirement の低下、明示的な成功 bypass を観察します。不正な revision、traversal、regular でない file、repository 外へ出る symlink、読めない/binary input は保守的に unknown または blocked とし、green にはしません。

`test_weakening` は blocking governance signal です。Coverage weakening は、適用される Contract/policy が blocking と定義しない限り review/unknown です。Dynamic quality route は変更 surface に応じた分析量を選び、strict/release route は full check を要求できます。

continue 以外の結果には stable finding と recovery condition が含まれます。検証強度を戻すか、独立レビュー可能な requirement-change evidence を示し、同じ base に対して再実行します。環境変数、local receipt、人の prose による critical signal の bypass はできません。Provider 側の required check と dynamic/generated test の意味は external または明示的な limitation です。

これは reference Test Weakening Guard の Rust-native semantic counterpart です。source Python module、Make target、source JSON wire format は搭載しません。
