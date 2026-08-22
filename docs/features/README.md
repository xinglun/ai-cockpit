---
author: AI Cockpit maintainers
title: "Features"
description: "A goal-first index of current AI Cockpit capabilities and boundaries."
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - capability_index
---

# Features

Start with [Capabilities and boundaries](../capabilities.md) for the complete
user-facing index. The main paths are:

- attach and observe a repository;
- create a governance skeleton without inventing human decisions;
- run the Work Item lifecycle with bounded verification and evidence reuse;
- connect an Agent or repository-bound MCP service explicitly;
- inspect Outcome, knowledge, status, diagnosis, and recovery signals.

AI Cockpit is a Repository Governance Layer. It does not become an Agent Runtime,
identity provider, security sandbox, workflow scheduler, or external audit system.
MCP returns repository-bound structured data; the Agent or conversation layer owns
the human-facing projection and must preserve unknowns and decision boundaries.
For a consistent visible handoff, call the repository-bound `work_item_outcome`
tool; it uses the same renderer as the CLI. Release acceptance also records
typed isolation manifests and digests, with only TMPDIR and CARGO_HOME
classified as allowed Runtime-write roots.

[Getting started](../getting-started/README.md) | [Operations](../operations/README.md) |
[Reference](../reference/README.md) | [中文](README.zh-CN.md) | [日本語](README.ja.md)
