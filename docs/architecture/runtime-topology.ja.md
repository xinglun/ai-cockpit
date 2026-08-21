---
author: AI Cockpit maintainers
title: "Runtime topology"
description: "Runtime component と repository-owned storage path。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - runtime_topology
---

# Runtime topology

```text
Human / Agent / CI
        │
    CLI / MCP adapter
        │
        ▼
    cockpit-core
        │
 ┌──────┼────────┬─────────┐
 ▼      ▼        ▼         ▼
Git  Repository Evidence Verification Knowledge
        │
        ▼
 対象 repository の .ai/
```

`cockpit-core` は pure かつ deterministic です。Adapter は外部 request を core
input に変換し、decision を CLI/MCP response に変換します。Repository access は
明示的な port から提供し、core は filesystem traversal や Git 呼び出しを行いません。

対象 repository に保存するのは facts、decisions、evidence、generated knowledge
projection だけです。Rust source、Python runtime、helper script、runtime schema
copy は保存しません。

## 1 つの Runtime、複数の Repository Context

```mermaid
flowchart TB
    Runtime["machine に 1 つの ai-cockpit binary"]
    Runtime --> A["RepositoryContext A<br/>/project-a/.ai/"]
    Runtime --> B["RepositoryContext B<br/>/project-b/.ai/"]
    Runtime --> C["RepositoryContext C<br/>/project-c/.ai/"]
```

Core は request-scoped です。`--repo`（または同等の MCP repository binding）で 1 回の
operation の context を選び、global な current repository、Work Item、profile は持ちません。
互換性のある repository は Runtime upgrade を共有できますが、各 context の Contract、
evidence、knowledge は分離されます。

## Agent Discovery / Adapter 層

```mermaid
flowchart LR
    Core[共有 Rust Core / binary]
    Attach[Layer 2: attach + Repository Protocol]
    Discover[Layer 3: 明示的 Agent Discovery / Adapter]
    Surface[AGENTS.md / CLAUDE.md / GEMINI.md / .cursor]
    Ownership[.ai/adapters/<provider>.json]
    Core --> Attach --> Discover
    Discover --> Surface
    Discover --> Ownership
```

`.ai/agent-interface.json` は repository-bound discovery fact だけを保持し、prompt や governance decision ではありません。
`attach` は provider rule を注入せず、`agent install --repo ... --provider ...` だけが ownership 付き managed section を追加できます。
`doctor`、`repair`、`detach` は操作前に ownership を検証します。Provider skill は利用性を高める任意の層であり、Runtime authority にはなりません。
