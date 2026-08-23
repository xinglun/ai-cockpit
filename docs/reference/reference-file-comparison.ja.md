---
author: AI Cockpit maintainers
title: "Reference file comparison"
description: "固定した baseline で reference source を file 単位に比較する方法。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - reference_parity
---

# Reference file comparison

このページは Rust project と公開 reference source を file ごとに比較する方法を説明します。
Reference は specification と behavior corpus であり、Rust Runtime にコピーする directory ではありません。

## 固定 baseline

- Reference: [spirex-ds-dev/ai-cockpit-template](https://github.com/spirex-ds-dev/ai-cockpit-template)、commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`。
- Rust baseline: [xinglun/ai-cockpit](https://github.com/xinglun/ai-cockpit) の `origin/main`、commit `46e426625a8cae450f1190d0bdbafd6d8e648a90`。
- 比較に使う Runtime: `ai-cockpit 0.2.27`、binary SHA256 `ea9a4a090307cb650eeda008942a5bc72fd0d1276b131405868f31d1eabcc048`。

Machine-readable ledger は
[`reference_file_inventory.json`](../../tests/conformance/reference_file_inventory.json) です。
Regression check は tracked reference path のすべてに一つだけ classification があることを確認し、
first batch の未分類 file を拒否します。

## Classification

- **implemented-equivalent** — 同じ reader/governance responsibility が同じ boundary で存在する。
- **implemented-different-by-design** — responsibility はあるが、Rust Protocol、shared external Runtime、
  または explicit Agent adapter が別の path/abstraction で担当する。
- **migrate-gap** — accepted counterpart がなく、bounded remediation が必要。
- **not-applicable** — 現在の Runtime product boundary の外。
- **reference-only** — 説明または conformance material としてのみ保持する。
- **generated-history** — immutable history または generated projection。コピーも静かな書き換えもしない。
- **deferred-next-batch** — 登録済みだが semantic comparison は後続 batch。parity や omission を意味しない。

## First batch: governance entrypoints

First batch は root Agent rules、`.ai` entrypoint と terminology、reader-facing README/architecture route、
reference governance configuration entrypoint を対象にします。Rust project は重要な boundary を維持しますが、
reference の Python Runtime、Makefile target、YAML guard tree、provider-global rules、generated history はコピーしません。

| Reference surface | Rust result | Boundary |
| --- | --- | --- |
| `AGENTS.md`、`CLAUDE.md`、`GEMINI.md`、Cursor rule | 意図した別実装 | Attached adapter と explicit provider install を使います。Shared Runtime は外部にあり、比較による provider-global config 注入はありません。 |
| `.ai/README.md`、glossary、cockpit workflow/adoption guide | 意図した別実装 | `.ai/README.md`、`.ai/glossary.md`、`docs/reference/agent-workflow.*`、getting-started route が Rust request-scoped Runtime workflow を担います。 |
| Reference guard、policy、quality、trust schema | 意図した別実装 | Typed Rust Protocol/Runtime service、repository test、CI manifest、reference docs が対応します。source YAML/JSON はコピーしません。 |
| Root と documentation README route | 意図した別実装 | 三言語 route は相互リンクし、shared Runtime と repository context isolation を説明します。 |
| `SECURITY.md` | 等価（Rust boundary を追加） | Security policy entrypoint を維持し、Runtime deployment/patch boundary を追加します。 |
| `CONTRIBUTING.md` | この batch で補完 | Explicit `--repo` lifecycle、fail-closed evidence、visible Outcome、reviewed PR、merge 後の exact cleanup を説明します。 |
| Reference の generated Work Item、decision、evidence、audit、release history | Generated history | これらの bytes は reference history として保持し、Rust repository にはコピーしません。 |

従って first batch で見つかった唯一の concrete entrypoint gap（`CONTRIBUTING.md`）は補完しました。
Second governance system は作らず、残りは ledger に明示して後続の semantic batch に送ります。

## Batch order

後続 batch は次の順序で比較し、必要な差分だけを実装します。

1. Contract field、intent、scenario/acceptance dimension、parallel slot、preflight review。
2. CI quality routing、dynamic verification tier、evidence assurance。
3. Runtime lifecycle、Outcome/MCP projection、recovery、knowledge、repository isolation。
4. Conformance、adversarial case、performance、release、adopter acceptance。

各 batch は独立した Contract と evidence を持ちます。review と publish 後、次の batch は published Runtime で
再度 acceptance を実施し、working-tree code を release behavior と取り違えないようにします。
