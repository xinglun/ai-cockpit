---
author: AI Cockpit maintainers
title: “Python fixture の適応境界”
description: “固定した Python fixture をファイル単位で Rust-native に対応づけ、アプリ・パッケージ・テスト実装はコピーしません。”
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-414-reference-python-fixture-boundary
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
capabilityClaims:
  - semantic_reference_mapping
---

# Python fixture の適応境界

このページは、固定した reference fixture `examples/fixtures/python/` の4ファイルを
一つずつ比較します。Python adopter に有用な意味だけを記録し、fixture、パッケージ
メタデータ、テスト runner を Rust Runtime にコピーしません。

[English](python-fixture-adaptation.md) · [简体中文](python-fixture-adaptation.zh-CN.md) · [日本語](python-fixture-adaptation.ja.md)

## ファイル単位の対応

| 固定 source file | source の事実 | Rust-native の対応と境界 |
| --- | --- | --- |
| `fixture.json` | Python service、`python3` toolchain、Linux/macOS platform、安全/テスト path を宣言します。 | Project Observer/Profile は repository-local の事実または候補事実として記録できます。Shared Runtime はこのファイルから Python capability、platform readiness、safe scope を推論せず、正確な Contract は owner が確認します。 |
| `pyproject.toml` | package metadata（`requires-python >=3.11`）と pytest の `tests` path を宣言します。 | Python packaging と pytest は adopter/provider の責任です。owner が `python -m pytest` のような明示的 command を提供し、Runtime は argv と結果を記録しますが、Python を install したり manifest をコピーしたりしません。 |
| `src/service.py` | 最小の application function が health value `ok` を返します。 | これは fixture の application code であり governance logic ではありません。Rust verification は adopter が宣言した command を実行して evidence に bind できますが、target は Python の意味を source から同梱・推論しません。 |
| `tests/test_service.py` | pytest test が health function の結果を assert します。 | これは sample assertion であり portable Runtime test Contract や enterprise evidence ではありません。adopter は自分の test command を宣言・実行し、source fixture test を target evidence に昇格しません。 |

## Installation と adopter 境界

Reference fixture の stack metadata は AI Cockpit の installation recipe ではありません。
adopter の外側に shared Runtime を一つ install し、repository を明示的に attach します。

```bash
repo=/path/to/python-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

Python interpreter、virtual environment、dependency lock、pytest configuration、
CI/provider evidence は adopter が所有します。その後の Runtime command には同じ
`--repo` を必ず付け、Contract scope、profile、snapshot、evidence、knowledge、Agent
adapter を repository ごとに隔離します。

## Adopter が継承するもの

attach 済み Python project は、shared Runtime の Contract validation、unknown の
fail-closed、identity-bound evidence、lifecycle、human Outcome を継承します。
reference fixture の `pyproject.toml`、Python source、pytest installation、または
test 実行済みという claim は継承しません。外部 authority が別途提供しない限り、
local test result は provider、hosted CI、Release、enterprise evidence ではありません。

これは semantic/documentation parity であり、Python toolchain support、source command
compatibility、JSON-wire compatibility ではありません。実際の Python adopter acceptance
は別途認可された post-release test です。

[Reference index](README.ja.md) · [Reference file comparison](reference-file-comparison.ja.md)
