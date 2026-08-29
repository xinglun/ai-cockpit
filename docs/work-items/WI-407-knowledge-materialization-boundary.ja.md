---
author: AI Cockpit maintainers
title: WI-407 — Knowledge projection materialization boundary
description: 派生 Knowledge projection を明示的・決定的・repository-local に保ちます。
workItemId: WI-407-knowledge-materialization-boundary
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-407-knowledge-materialization-boundary
terminalArchive: .ai/work-items/archive/WI-407-knowledge-materialization-boundary.contract.json
terminalVerification: .ai/evidence/WI-407-knowledge-materialization-boundary.verification.json
---

# WI-407 — Knowledge projection materialization boundary

## Intent

Knowledge directory、index、source digest、refresh timing を明示的かつ検証可能にし、第二の governance authority を作りません。

## 範囲

- CLI と MCP の Knowledge query が repository-local derived write boundary を同じ形式で報告します。
- stale または malformed な legacy/v2 projection を決定的に rebuild し、repository 間を隔離します。
- Contract、evidence、archive、decision を authority として保持します。
- 英語・簡体字中国語・日本語の文書で同じ境界を説明します。

## Evidence

- Archive Contract: `.ai/work-items/archive/WI-407-knowledge-materialization-boundary.contract.json`
- Verification: `.ai/evidence/WI-407-knowledge-materialization-boundary.verification.json`
- Pull Request: [#372](https://github.com/xinglun/ai-cockpit/pull/372)

## 境界

Knowledge は repository-local な derived projection です。明示的な query は `.ai/knowledge/` を materialize または rebuild できますが、変更を authorize せず、governance authority も変更しません。Lifecycle command は黙って materialize しません。
