---
author: AI Cockpit maintainers
title: "Derived artifacts and authority boundary"
description: "How Rust Runtime projections remain observable without becoming governance authority."
audience: [user, agent, maintainer]
status: current
authority: canonical
lastVerifiedBy: WI-548-reference-file-comparison-batch-38
---

# Derived artifacts and authority boundary

AI Cockpit distinguishes repository facts from views derived from those facts.
Contracts, repository snapshots, verification receipts, decisions, and archive
manifests are authoritative only when their typed identity and digest bindings
validate. Status, summaries, Outcome handoffs, and knowledge indexes are
derived projections for people and Agents; a projection cannot authorize a
change or replace its source record.

The reference template has a Python registry that validates generated facts and
artifact inputs. The Rust Runtime keeps the portable rule—explicit inputs,
source references, deterministic derivation, and fail-closed identity checks—
but does not copy that registry or its JSON wire shape. Repository-local
Knowledge is likewise a read/derived view and never a substitute for Contract,
Evidence, or human Decision.

For an audit, read the source record first, then compare the projection:

1. `ai-cockpit inspect --repo <repo>` establishes the snapshot and changed paths.
2. `ai-cockpit status --repo <repo>` shows lifecycle facts and readiness.
3. `ai-cockpit work-item outcome --repo <repo> --id <id>` renders the human
   handoff; it must not be treated as a new decision.

If a projection and its source disagree, the Runtime reports the binding or
freshness problem and stops. Agents must not edit generated status, Outcome,
knowledge, evidence, or archive files by hand; amend the owning Contract or
run the owning Runtime operation with explicit human authority.

Every attached object repository inherits this same boundary through the
shared binary and explicit `--repo` context. It does not inherit the source
Python registry or any source-specific generated-file policy.
