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

`inspect → attach → preflight → verify → finish/archive/close`

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

## 製品の境界

この repository は V1 の upgrade、migration、Rust port ではありません。V1 template
は specification source、behavioral oracle、conformance corpus、過去の reference
としてだけ使います。runtime code、Python module、`Makefile.ai`、installer、runtime
schema を target repository にコピーしません。

AI Cockpit は Agent Runtime、Workflow Engine、Security Sandbox、identity provider、
compliance certificate、human review の代替ではありません。external identity、branch
protection、production isolation、provider Release、provenance は外部 evidence または
adopter の責任です。
