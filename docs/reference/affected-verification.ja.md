---
author: AI Cockpit maintainers
title: Affected verification と dependency confidence
description: 依存関係の知識が complete、partial、unknown の場合の保守的な Verification 計画を説明します。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-142-affected-verification
---

# Affected verification と dependency confidence

Verification graph は `DependencyConfidence` を Policy truth とは別に記録します。

- `complete` は変更ノードと既知の下流依存を計算します。
- `partial` は決定できる affected 集合だけを残し、そのノードを強い候補 tier に
  escalate して `dependency_graph_partial` を表示します。
- `unknown` は graph の全ノードを保守的に含め、`dependency_graph_unknown` を表示します。

未知または安全でないノード参照は fail-closed です。partial を complete として扱い
ませんが、既知の affected 境界が十分な場合に全ノードを最高 tier で再実行することも
ありません。この projection は実行コストだけを下げ、Policy tier、protected gate、
authority、evidence requirement を弱めることはできません。
