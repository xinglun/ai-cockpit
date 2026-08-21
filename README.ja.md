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

`inspect → attach → preflight → verify → finish/archive/close`

## 30 秒で開始

Runtime は一度だけ install し、作業対象 repository を attach します。

```bash
ai-cockpit attach --repo /path/to/repository
ai-cockpit status --repo /path/to/repository
```

最初の governed Work Item は[機能と境界](docs/capabilities.ja.md)を、
install と検証は[Release と配布](docs/release/distribution.ja.md)を参照してください。

Runtime は 1 つだけ install し、各 target repository を個別に attach します。

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

## 製品の境界

この repository は V1 の upgrade、migration、Rust port ではありません。V1 template
は specification source、behavioral oracle、conformance corpus、過去の reference
としてだけ使います。runtime code、Python module、`Makefile.ai`、installer、runtime
schema を target repository にコピーしません。

AI Cockpit は Agent Runtime、Workflow Engine、Security Sandbox、identity provider、
compliance certificate、human review の代替ではありません。external identity、branch
protection、production isolation、provider Release、provenance は外部 evidence または
adopter の責任です。
