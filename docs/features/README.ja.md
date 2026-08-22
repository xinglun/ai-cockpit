---
author: AI Cockpit maintainers
title: "Features"
description: "AI Cockpit の現在の capability と責任境界を goal-first で読むための index。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - capability_index
---

# Features

完全な利用者向け index は[Capabilities and boundaries](../capabilities.ja.md)です。主な path は次のとおりです。

- repository を attach して observe する;
- human decision を発明せず governance skeleton を作る;
- bounded verification と evidence reuse で Work Item lifecycle を実行する;
- Agent または repository-bound MCP service を明示的に接続する;
- Outcome、knowledge、status、diagnosis、recovery signal を読む。

AI Cockpit は Repository Governance Layer です。Agent Runtime、identity provider、security sandbox、
workflow scheduler、外部 audit system ではありません。MCP の `work_item_outcome` は Runtime が生成・検証した
人間向け projection も返します。Agent または conversation layer は handoff を選択・表示・伝達しますが、
projection をガバナンス権限に変えてはならず、unknown と decision boundary を保持してください。Release acceptance は
typed isolation manifest と digest も記録し、Runtime write root として
許可されるのは TMPDIR と CARGO_HOME だけです。

[Getting started](../getting-started/README.ja.md) | [Operations](../operations/README.ja.md) |
[Reference](../reference/README.ja.md) | [English](README.md) | [中文](README.zh-CN.md)
