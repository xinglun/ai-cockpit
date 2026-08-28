---
author: AI Cockpit maintainers
title: 実装 Knowledge
description: 完了した Work Item のための決定的で evidence-bound な Knowledge record。
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: translation
canonical: docs/reference/implementation-knowledge.md
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
capabilityClaims:
  - evidence_bound_knowledge
---

# 実装 Knowledge

[English](implementation-knowledge.md) · [简体中文](implementation-knowledge.zh-CN.md) · [日本語](implementation-knowledge.ja.md)

実装 Knowledge は、検証済みで archive された Work Item の read-only projection です。Agent memory、第二の事実源、design authority ではありません。Authority は Contract、verification evidence、archive、最終 Outcome に残ります。

## Query

```text
ai-cockpit knowledge query --repo /path/to/repository \
  --topic <topic> --component <component> \
  --state verified --work-item-id <id>
```

指定した filter は AND で評価され、repository に bind された安定した record が返ります。`--v2` は truth state、confidence、evidence reference、unknown、snapshot digest を含む `KnowledgeV2Record` を選択します。Query は `.ai/` に書き込まず、新しい変更を authorize しません。

Work Item の finalize 後、Runtime は検証済み archive から record と index を更新します。source-path dependency map があるため影響を受けた record だけを更新できます。index が欠落、破損、stale、不完全なら rebuild と再検証を行うか、partial/unknown を明示します。

## Reference source との差分

Reference は date、merged commit、`latestKnownRecord`、supersession filter も説明します。現在の Rust projection が公開するのは上記の repository-bound filter だけです。追加の次元は推測せず、この release の CLI/MCP contract には含めません。追加時は別の Contract、schema、test、三言語文書が必要です。

Knowledge は semantic search、vector retrieval、fuzzy recommendation、RAG ではありません。空の結果は未対応の証明ではなく、date、supersession、benefit は evidence に明記された場合だけ表示します。

## Shared Runtime と adopter

Runtime は共有できますが、各 query は明示的な `--repo` を要求します。index、record、evidence、adapter state は repository の `.ai/` に隔離されます。Adopter が継承するのは read-only の evidence boundary であり、reference の生成 record や Python/Make command ではありません。
