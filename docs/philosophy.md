---
author: AI Cockpit maintainers
title: "Design Philosophy"
description: "Why AI Cockpit turns repository facts into bounded decisions for human review."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - design_philosophy
keywords: [ai-cockpit, design-philosophy, evidence, human-control]
---

# Design Philosophy

## Purpose

This page answers: **why is AI Cockpit designed as a governance layer instead
of an autonomous agent or workflow engine?**

## Audience

Read it when you are deciding whether the product fits your development process,
or when you need to understand why a check stops instead of guessing.

## Outcome

You will understand the principles behind the runtime and the boundary between
what AI Cockpit can prove locally and what must remain external evidence.

## The North Star

AI Cockpit supports calibrated human-agent trust. It makes the intended change,
the allowed scope, the repository facts, the verification result, and the
remaining human decision visible:

```text
Evidence → Governance Decision → Human Control
```

## Principles

1. **Evidence over self-declaration.** A command, agent message, or local flag
   is not proof by itself. Decisions are derived from typed repository facts and
   recorded evidence.
2. **Explicit boundaries.** A Work Item states intent, scope, exclusions,
   authority, acceptance, and required evidence before implementation begins.
3. **One observed snapshot.** Git state, configuration, and relevant files are
   observed once and reused as immutable inputs. A later change is a new fact,
   not something to silently fold into the old decision.
4. **Fail closed.** Missing, stale, contradictory, or tampered evidence becomes
   `unknown` or `blocked`; it never becomes a convenient pass.
5. **Proportional controls.** Low-risk local inspection should stay lightweight;
   protected gates require stronger identity, evidence, and human authority.
6. **Human control remains real.** AI Cockpit can explain what is safe to do
   next. It cannot approve an unverified change, authenticate an external actor,
   or replace review.
7. **Adapters stay thin.** CLI and MCP translate requests and responses. The
   governance rules live in shared application services and the pure core.

## What this means in practice

When a user asks an agent to “update the documentation,” AI Cockpit does not
interpret that sentence as an unrestricted workflow. It asks for a bounded Work
Item, records the repository baseline, runs the declared checks, and presents a
decision that a person can proceed with, investigate, approve, block, or recover.

## Action or decision

Put request, scope, repository state, verification, and human decisions in the
governed Work Item. Put domain-specific proof—such as a provider signature,
SBOM, vulnerability scan, or production approval—in the tool or service that
can actually produce it. Link the evidence without claiming ownership of it.

## Stop conditions

Stop when a requested effect has no declared boundary, when evidence ownership
is ambiguous, when a snapshot changed during a protected operation, or when a
local record is being used as proof of an external control. Investigate the
missing link; do not guess.

## Next steps

1. [Architecture](architecture.md) — the runtime path and evidence ownership.
2. [Capabilities](capabilities.md) — a reader-first feature overview and details.
3. [Product boundary](architecture/product-boundary.md) — what remains outside.

## Technical depth

The implementation expresses these principles through the Repository Protocol,
typed Work Item lifecycle, immutable repository snapshots, deterministic
governance decisions, bounded verification plans, content-addressed evidence,
and shared CLI/MCP services. These mechanisms support review; they are not a
general semantic-risk detector, identity provider, sandbox, or compliance
certificate.
