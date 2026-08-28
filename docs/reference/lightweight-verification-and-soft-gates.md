---
author: AI Cockpit maintainers
title: "Lightweight verification and soft gates"
description: "Proportional, evidence-bound verification without weakening mandatory governance controls."
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# Lightweight verification and soft gates

[简体中文](lightweight-verification-and-soft-gates.zh-CN.md) · [日本語](lightweight-verification-and-soft-gates.ja.md)

AI Cockpit selects a verification route from repository facts, Work Item
Contract, stage, and applicable policy. `light`, `standard`, and `strict` are
verification intensity choices; they are not assurance levels and never grant
authority.

## Rules

- A route may add checks, but a cache hit may be reused only when every
  content, diff, environment, Runtime, policy, repository, Work Item, and
  stage binding still matches.
- Dependency planning is deterministic. A cycle, malformed node, or unknown
  dependency is not silently complete; it remains `partial` or `unknown` and
  escalates the affected checks.
- Escalation is monotonic: light → standard → strict can add required work,
  but cost, reuse, or a provider hint cannot lower a required route.
- Soft, skipped, or advisory observations are visible in evidence and cannot
  turn missing, stale, contradictory, or protected evidence green.

The canonical gate is evaluated with explicit repository context:

```sh
ai-cockpit gate --repo /path/to/repository --contract .ai/work-items/active/WI.contract.json
```

The gate receipt describes routing and is not an execution token. Hosted CI,
release, provider, and enterprise assurance remain separate delegated
boundaries. See [Governance profiles](governance-profiles.md) and
[Verification semantics](verification-semantics.md).

## Object-project inheritance

Every adopter uses the same shared Runtime and explicit `--repo` binding. The
selected route and evidence are repository-local; there is no global current
project or global Work Item. A lighter route is a proportional verification
choice, not permission to omit a Contract, human review, scope, or evidence
integrity control.
