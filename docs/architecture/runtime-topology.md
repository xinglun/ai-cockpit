---
author: AI Cockpit maintainers
title: "Runtime Topology"
description: "The runtime components and repository-owned storage path."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - runtime_topology
---

# Runtime Topology

```text
Human / Agent / CI
        │
   CLI / MCP adapters
        │
        ▼
   cockpit-core
        │
 ┌──────┼────────┬─────────┐
 ▼      ▼        ▼         ▼
Git  Repository Evidence Verification Knowledge
        │
        ▼
 Target repository .ai/
```

`cockpit-core` is pure and deterministic. Adapters translate external requests
into core inputs and translate decisions into CLI/MCP responses. Repository
access is supplied through explicit ports; core never traverses a filesystem or
invokes Git.

The target repository stores facts, decisions, evidence, and generated knowledge
projections. It does not store Rust source, Python runtime, helper scripts, or
runtime schema copies.
