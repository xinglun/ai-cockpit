---
author: AI Cockpit maintainers
title: Instruction traceability
description: Evidence-backed links from comparison instructions to Work Items and verification.
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
capabilityClaims:
  - comparison_traceability
---

# Instruction traceability

[English](instruction-traceability.md) · [简体中文](instruction-traceability.zh-CN.md) · [日本語](instruction-traceability.ja.md)

File-by-file comparison is governed by the machine-readable inventory at
[`tests/conformance/reference_file_inventory.json`](../../tests/conformance/reference_file_inventory.json).
It binds each pinned source path to exactly one classification, a bounded
counterpart decision, and a reason. The comparison and parity pages explain
the semantic decision for people; the inventory is the anti-omission check.

## Forward and reverse checks

For every comparison batch, the forward path is:

```text
pinned instruction/source path
  → Work Item Contract
  → target counterpart or explicit boundary
  → acceptance criteria and verification evidence
  → reviewed PR, merge, and close receipt
```

The reverse path checks that every listed Work Item has a source set, evidence,
and a delivered counterpart—or a recorded no-change/reference-only reason.
Archived Work Items remain the historical source of delivery truth; they are
not silently replaced by an untracked note. Hosted-performance observations,
when present, use explicit `pass`, `not_run`, or `fail` states with a reason.

The inventory script is structural: it proves coverage and stable identity, not
that a natural-language claim is true. A new semantic responsibility gets its
own bounded Contract and evidence. It is never hidden in a later unrelated
Work Item.

## Non-copy and adopter boundary

This Rust project does not import the reference remediation JSON, Make command,
or Python checker as Runtime authority. The same inventory and explicit
repository-bound lifecycle can be inherited by an adopter, while its own
source paths, Work Items, evidence, and provider receipts remain separate.
