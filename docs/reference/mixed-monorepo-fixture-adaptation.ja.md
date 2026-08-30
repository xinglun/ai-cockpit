---
author: AI Cockpit maintainers
title: "Mixed-monorepo fixture 適応"
description: "業務コードや toolchain をコピーせず、固定 mixed Python/Node fixture の file-by-file Rust-native boundary を定義します。"
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-420-reference-mixed-monorepo
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
capabilityClaims:
  - semantic_reference_mapping
---

# Mixed-monorepo fixture 適応

このページは pinned reference の `examples/fixtures/mixed-monorepo/` にある 5 file を
一つずつ比較します。この fixture は executable application sample であり、Rust Runtime
code や portable な enterprise evidence ではありません。有用な governance meaning は残しますが、
Python/Node toolchain はコピーしません。

[English](mixed-monorepo-fixture-adaptation.md) · [简体中文](mixed-monorepo-fixture-adaptation.zh-CN.md) · [日本語](mixed-monorepo-fixture-adaptation.ja.md)

## File-by-file mapping

| Pinned source file | Source fact | Rust-native counterpart と boundary |
| --- | --- | --- |
| `fixture.json` | mixed Python/Node sample、generic installer metadata、3 platform、safe/test path を宣言します。 | Project Observer/Profile は adopter で実際に観測した fact を記録できます。Runtime は fixture metadata から toolchain capability や safe scope を推測しません。 |
| `package.json` | dependency/script のない private Node package metadata です。 | fixture application input に限ります。Node install、dependency、script、execution は adopter/provider の責任です。 |
| `pyproject.toml` | 最小の Python project metadata です。 | portable Contract/Runtime dependency ではありません。Python install、dependency、test command は明示的 adopter evidence を必要とします。 |
| `services/api/app.py` | `ok` を返す health function です。 | application code であり governance logic ではありません。Runtime は adopter 宣言の argv 結果を bind できますが、Python behavior は持ち込みません。 |
| `services/api/tests/test_app.py` | health function を pytest assertion で確認します。 | fixture evidence に限られます。adopter は自分の verification command を宣言・実行し、source test を target evidence に昇格させません。 |

## Installation と adopter boundary

この fixture は AI Cockpit の install recipe ではありません。adopter の外側に shared Runtime を 1 つ
install し、repository を明示的に attach します。

```bash
repo=/path/to/mixed-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

Python/Node interpreter、dependency lock、test command、hosted-provider evidence は adopter が所有します。
後続の Runtime command も同じ `--repo` を必ず付け、Contract、snapshot、evidence、knowledge、Agent adapter
record は repository-local に保ちます。

## Adopter が継承するもの

attach された mixed repository は shared Runtime の Contract validation、fail-closed unknown handling、
identity-bound evidence、lifecycle、repository isolation、human Outcome rule を継承します。ただし fixture の
package metadata、source、test runner、installer behavior、toolchain availability claim は継承しません。
これは semantic/documentation parity であり、mixed-stack toolchain support、source-command compatibility、
second-technology adopter acceptance ではありません。

[Reference index](README.ja.md) · [Reference file comparison](reference-file-comparison.ja.md)
