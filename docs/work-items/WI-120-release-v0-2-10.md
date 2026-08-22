---
author: AI Cockpit maintainers
workItemId: WI-120-release-v0-2-10
title: Publish v0.2.10 and perform immutable adopter acceptance
description: Publish and verify the v0.2.10 release and its adopter acceptance baseline.
audience:
  - adopter
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-120-release-v0-2-10
---

# WI-120 — Publish v0.2.10 and perform immutable adopter acceptance

## Intent

Publish the first public Runtime containing the Contract preflight human-review
gate, then prove that the downloaded Release binary can govern a fresh adopter
and upgrade the immediately previous adopter without source fallback.

## Scope

- bump the workspace and current release documentation to `v0.2.10`;
- publish the immutable public artifacts and record their Runtime identity;
- run fresh-adopter and v0.2.9 → v0.2.10 N-1 acceptance in isolated roots;
- install the published binary and verify the current repository with explicit
  repository context.

## Boundaries

This Work Item does not add Runtime features, rewrite historical evidence, or
modify global Agent/MCP configuration. Post-release acceptance can report a
failure but must not rewrite the published Release truth.

## Acceptance

- version, documentation, and release-policy checks identify only `v0.2.10` as
  the current baseline while retaining historical references explicitly;
- CI and release checks pass;
- fresh-adopter output preserves `first-adopter-smoke = not_ready` and records
  the downloaded binary digest, repository identity, evidence reuse, lifecycle,
  isolation, and cleanup receipts;
- N-1 acceptance proves the v0.2.9 → v0.2.10 compatibility path;
- the installed public binary reports `0.2.10` and passes current-repository
  inspect, status, doctor, Agent doctor, and Outcome checks.

## Evidence and decision boundary

Release publication is not proof of adopter acceptance. The public archive,
manifest, checksum, acceptance receipts, and Runtime identity must remain
separately verifiable. A human decision is required for any yellow or red
condition; acceptance criteria remain Contract source text and are not silently
translated into governance facts.
