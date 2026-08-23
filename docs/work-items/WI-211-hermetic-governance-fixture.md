---
author: AI Cockpit maintainers
title: "WI-211 — hermetic governance fixture event context"
description: "Make governance regression fixtures deterministic when release workflows export GitHub event variables."
audience:
  - maintainer
  - reviewer
workItemId: WI-211-hermetic-governance-fixture
status: current
authority: canonical
lastVerifiedBy: WI-211-hermetic-governance-fixture
---

# WI-211 — hermetic governance fixture event context

The release workflow exports GitHub event variables for the entire source
quality job. The governance regression test previously allowed those values to
leak into ordinary fixtures, so a local run could pass while the release-tag
run failed. This Work Item makes every fixture's event context explicit.

## Acceptance

1. `tests/ci/governance_integrity_gate_test.sh` passes under ordinary and
   release-tag environment variables.
2. Ordinary fixtures explicitly clear release context; real `release-tag-*`
   fixtures still receive strict tag context.
3. The same deterministic findings and exit status are preserved in both
   environments.
4. The immutable v0.2.26 publication history is not moved, rewritten, or used
   as a source fallback.

## Out of scope

This Work Item does not change Runtime governance semantics, public release
assets, reference-source parity, or user-global Agent/MCP configuration.

## Verification

Run the regression once with no GitHub event variables and once with
`GITHUB_EVENT_NAME=push`, `GITHUB_REF=refs/tags/<tag>`, and matching
`GITHUB_SHA`. Then run the repository gate manifest and workspace tests.

## Evidence boundary

The environment-isolation fix is source-test evidence. It does not convert the
failed v0.2.26 publication into a successful Release; that history remains
immutable and any new release must use a new tag.
