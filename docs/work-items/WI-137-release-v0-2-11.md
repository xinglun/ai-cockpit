---
author: AI Cockpit maintainers
workItemId: WI-137-release-v0-2-11
title: Publish v0.2.11 and perform immutable adopter acceptance
description: Publish the merged Runtime fixes and validate the public binary in isolated adopter and current-repository flows.
audience:
  - adopter
  - maintainer
status: release-preparation
authority: canonical
lastVerifiedBy: WI-137-release-v0-2-11
---

# WI-137 — Publish v0.2.11 and perform immutable adopter acceptance

## Intent

Publish the first immutable Runtime containing WI-135 repository-bound
retention/evidence validation and WI-136 Task Outcome reporting, then prove the
downloaded public binary can govern a fresh adopter and this repository.

## Scope and boundaries

- Bump the workspace and current release documentation to `v0.2.11`.
- Run source quality, release policy, fresh-adopter, and N-1 acceptance checks.
- Install only the published v0.2.11 artifact and verify the current repository.
- Keep the release acceptance artifact separate from repository history.

This Work Item does not add Runtime behavior, rewrite historical evidence,
modify global Agent/MCP configuration, mutate an external Homebrew tap, or use
a source/workspace binary as release acceptance evidence.

## Acceptance

1. Cargo metadata, archive names, manifests, and the three language routes agree
   on v0.2.11; historical N-1 references remain explicit.
2. Public fresh-adopter acceptance downloads and verifies only the immutable
   v0.2.11 Release, preserves `first-adopter-smoke = not_ready`, records
   repository/runtime identity, evidence reuse, lifecycle, isolation, and
   cleanup receipts.
3. N-1 acceptance proves v0.2.10 → v0.2.11 compatibility without rewriting
   old bytes or Release truth.
4. The installed public binary passes inspect, status, doctor, Agent doctor,
   and the human Outcome handoff on this repository; the new WI-136 report is
   readable by that installed binary.

## Verification and evidence

Required evidence is the full workspace verification receipt, public fresh-
adopter acceptance receipt, N-1 upgrade receipt, runtime identity (version,
archive digest, binary digest, target, download source), and the final Outcome.
