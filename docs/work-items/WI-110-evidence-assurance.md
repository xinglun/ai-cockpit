---
author: AI Cockpit maintainers
title: "WI-110 — Evidence assurance and historical projection"
description: "Strict verification evidence, current Runtime binding, and honest legacy projection."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-110-evidence-assurance
---

# WI-110 — Evidence assurance and historical projection

## Intent and goal

Make the verification trust boundary explicit: a persisted v2 evidence
envelope and captured receipt must be typed, identity-bound, and fail closed.
The current CLI lifecycle must accept only evidence produced by its installed
Runtime. Historical pre-v2 bytes remain immutable and are shown as historical
input rather than a fabricated current failure.

## Scope

- strict `VerificationEvidenceV2` envelope and nested `VerificationReceipt`
  validation;
- Work Item, repository, snapshot, and Runtime identity binding;
- Runtime-bound CLI/MCP verify, finish, archive, close, and Outcome paths;
- regression coverage for unknown fields, missing nested identity, malformed,
  foreign-runtime, and legacy evidence;
- English, Simplified Chinese, and Japanese documentation.

## Invariants

Unknown envelope or captured-receipt fields, missing nested identity, invalid
digests, and foreign Runtime identity cannot produce a green Outcome or pass a
Runtime-bound lifecycle operation. `digest_only` retention intentionally has no
captured receipt. A pre-v2 record (no `evidenceSchemaVersion`) is read-only
historical input and projects as yellow `legacy_evidence_historical`; it is not
rewritten, promoted to green, or reported as a current red failure. A v2 record
with missing identity remains red.

The compatibility Rust APIs without an explicit `RuntimeContext` remain for
embedders that own Runtime identity. The installed CLI and repository-bound
MCP always use the Runtime-bound APIs.

## Verification

Focused evidence and lifecycle tests cover strict envelope/nested receipt
tampering, foreign Runtime rejection, CLI foreign-runtime rejection, and
immutable legacy projection. Workspace format, Clippy, and full tests are
required before merge.

## Boundary

This Work Item does not implement provider attestation, external immutable
audit storage, or historical byte migration. Those remain separate enterprise
assurance work.
