---
author: AI Cockpit maintainers
title: "WI-112 Agent operating rules parity"
description: "Document the applicable reference-source Agent workflow rules for future Rust Work Items."
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - agent_workflow_boundaries
---

# WI-112: Agent operating rules parity

## Goal

Make future Work Items inherit the useful Agent workflow, Outcome, review,
release, and safety boundaries found in the reference source while preserving
this project's shared Rust Runtime and repository-local state model.

## Scope

This Work Item updates `AGENTS.md`, `.ai/README.md`, the three-language Agent
workflow reference, its reference indexes, and this Work Item record. It
classifies the reference rules as inherited, Rust-specific adaptations, or
template-only exclusions. It does not change Runtime code, Protocol schemas,
global Agent/MCP configuration, packaging, or release assets.

## Acceptance

- The remote/default-branch and immutable published Release boundaries are
  explicit.
- Contract, glossary, scope, Summary, evidence, checks, Outcome, defect
  resolution, parallel compatibility, and post-merge closure rules are
  inherited for future Work Items.
- Human Outcome delivery explicitly retains visible `🔴`, `🟡`, and `🟢`
  markers and fail-closed progression semantics.
- English, Chinese, and Japanese reference pages and Work Item records are
  synchronized and link-valid.
- Reference-specific `make ai-*`, `contractVersion: 2`, and V1 assumptions are
  explicitly excluded from this Rust project.

## Verification

```text
bash tests/docs/documentation_acceptance.sh
git diff --check
```

## Outcome

Status: **Implemented locally; Runtime-bound lifecycle and documentation checks
passed.**
