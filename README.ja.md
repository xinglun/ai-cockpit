---
author: AI Cockpit maintainers
title: "AI Cockpit"
description: "AI 支援開発のための、evidence-based な repository governance。"
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - repository_governance_layer
---

# AI Cockpit

[English](README.md) | [中文](README.zh-CN.md)

AI Cockpit は AI 支援開発のための repository governance runtime です。repository
の事実、宣言した範囲、検証結果、人間の選択を、後から確認できる bounded decision
に変換します。

## 解決する問題

AI による変更は範囲を越えたり、テストを弱めたり、検証を省略したり、reviewer に
十分な evidence を残さないことがあります。AI Cockpit は変更の意図、実際の状態、
必要な check、unknown、human decision を明示します。

## 動作の流れ

利用者と tool は CLI または local MCP adapter を使います。repository の状態は
Repository Protocol v1 に保存し、Rust governance core は application code から独立
しています。基本の流れは次のとおりです。

`inspect → attach → start → preflight → checkpoint → verify → finish → archive → close`

`start` は human-owned Contract を記録し、`preflight` は開始できるかを評価します。
`checkpoint` は実装を進める前の serial gate です。`verify` は fresh evidence を記録し、
`finish` は結果を bind、`archive` は immutable な Work Item bundle を保存し、`close` は
明示的な human decision を記録します。

## 30 秒で開始

Runtime は一度だけ install し、作業対象 repository を attach します。

```bash
ai-cockpit attach --repo /path/to/repository
ai-cockpit status --repo /path/to/repository
```

最初の governed Work Item は[機能と境界](docs/capabilities.ja.md)を、
install と検証は[Release と配布](docs/release/distribution.ja.md)を参照してください。

## 検証済みの完了例

実際の、範囲を限定した handoff の完了例は[WI-663 Outcome](.ai/work-items/archive/WI-663-wi659-outcome-trust-replacement.outcome.json)
です。これはこの repository の governance record に関する evidence であり、普遍的な安全性や
product performance の主張ではありません。

- **結果:** Archive record は `state=finish_ready`、`decisionState=green`、
  `verification.status=verified` を記録しています。別の[close decision](.ai/decisions/WI-663-wi659-outcome-trust-replacement.close.json)
  は repository owner の approval を記録します。検証通過と approval は同じ事実ではありません。
- **主な変更:** Input は明示された base と bounded scope に対する Outcome presentation-layer
  repair でした。記録された finding は verification、lifecycle、human decision を分離し、
  historical、stale、missing、superseded evidence の違いも保持しています。
- **Evidence boundary:** [verification evidence](.ai/evidence/WI-663-wi659-outcome-trust-replacement.verification.json)
  は declared check と repository/Work Item binding を支えます。[finalization receipt](.ai/decisions/WI-663-wi659-outcome-trust-replacement.finalize.json)
  は記録された merge と cleanup の事実を支えます。どちらも release、普遍的な安全性、user-visible
  benefit を証明しません。
- **残る不確実性:** `user_visible_benefit_not_declared` は明示的に残ります。Current Runtime
  から historical record を見ると、historical evidence が再検証されていないと表示されることも
  あります。これは freshness の制限であり、current test failure ではありません。
- **人の次の一歩:** Archive の green verification から新しい authorization は推論できません。
  Current decision に evidence を使う場合は、current Runtime で再検証し、人が明示的に decision を行います。

Checkout から read-only handoff lookup を繰り返すには、placeholder を実際の repository path に置き換えます。

```bash
repo=/path/to/ai-cockpit
ai-cockpit work-item outcome --repo "$repo" \
  --id WI-663-wi659-outcome-trust-replacement
```

[最初の Work Item walkthrough](docs/getting-started/first-work-item.ja.md)では、同じ case を input と
scope から evidence、Outcome、human decision、cleanup まで対応付けます。

## Shared Runtime と repository isolation

各 target repository を個別に attach します。

```text
ai-cockpit attach --repo /project-a
ai-cockpit attach --repo /project-b
```

binary は共有しますが、各 repository は独自の `.ai/` Contract、Evidence、Knowledge
を持ちます。repository-bound command には常に `--repo` が必要で、Runtime に global な
current repository や active Work Item はありません。

`attach` は最小の repository scaffold（`cockpit.toml`、`project.json`、`agent-interface.json`、
Work Item directory、evidence、decisions、knowledge）だけを作成し、Agent provider instruction は install しません。
Governance skeleton が必要な場合は明示的に実行します。

```bash
ai-cockpit work-item new --repo /project-a \
  --id payment-refund-guard --mode code
```

解決できた snapshot-derived fact と、人間が入力すべき `intent`、`scope`、`acceptanceCriteria`、`authority` を表示します。
状態は `not_ready` で、scaffold が approved や verified を主張することはありません。`profile propose --repo /project-a` も
read-only の candidate amendment を出力し、formal profile は変更しません。

選択した Agent host に repository を発見させる場合は、repository-local adapter を明示的に使います。

```bash
ai-cockpit agent list --repo /project-a
ai-cockpit agent install --repo /project-a --provider codex
ai-cockpit agent doctor --repo /project-a --json
```

書き込まれるのは選択した repository surface と `.ai/adapters/` の ownership 付き section だけで、
global Agent/MCP 設定は変更しません。Discovery、adapter install、connection、verification、compliance は別の state です。

## 3 つの decision state

- `green`: 必要な evidence が bounded な次の操作を支える。
- `yellow`: evidence が不足、stale、矛盾、または human confirmation が必要。
- `red`: control が失敗、または authority がなく、操作を停止する。

## ここから開始

- [ドキュメントマップ](docs/README.ja.md) — adopter、contributor、reviewer、MCP、maintainer の入口。
- [機能と境界](docs/capabilities.ja.md) — 現在の command surface と外部責任。
- [Release と配布](docs/release/distribution.ja.md) — install、検証、rollback、MCP 設定。

source checkout では、contributor は `cargo run -p cockpit-cli -- --help` で command
surface を確認できます。Public Release と Homebrew availability は別の release
evidence であり、この checkout だけでは利用可能とは言えません。

## 外部に残る責任

External identity、branch protection、production isolation、provider Release、provenance は
外部 evidence または adopter の責任です。AI Cockpit は bounded な repository governance を
提供しますが、human review、組織の security system、compliance framework の代替ではありません。
