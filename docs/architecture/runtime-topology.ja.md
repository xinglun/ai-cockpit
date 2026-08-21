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
