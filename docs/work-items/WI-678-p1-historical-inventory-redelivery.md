---
author: AI Cockpit maintainers
title: "WI-678 — P1 historical finalization inventory redelivery"
description: "Re-verify bounded historical finalization candidate reuse from the latest default branch."
audience: [maintainer, reviewer, adopter]
workItemId: WI-678-p1-historical-inventory-redelivery
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-678-p1-historical-inventory-redelivery
---

[简体中文](WI-678-p1-historical-inventory-redelivery.zh-CN.md) · [日本語](WI-678-p1-historical-inventory-redelivery.ja.md)

# WI-678 — P1 historical finalization inventory redelivery

## Intent and hypothesis

P0 measurement identified historical finalization inventory as the material
status-path bottleneck. The hypothesis is that reading the decisions directory
once and reusing the immutable transition-candidate list within that single
observation lowers repeated I/O without changing governance semantics.

This Work Item is a fresh redelivery from `origin/main` at
`c1f1f1d9f17ec2242a59f287a4368bd6bda5993e` after draft PR #671 failed the
hosted quality gate. PR #671 remains audit context, not review or merge
authority.

## Boundary

In scope: the historical inventory/resolver path, its bounded counter test,
three-language documentation, and three reference-parity rows. Candidate data
is reused only inside one immutable observation. There is no process-global
cache, cross-request snapshot, authorization change, evidence rebinding, or
unrelated refactor.

## Authorization record

On 2026-09-08, `human:user-request` authorized this Agent to continue through
provider or lifecycle interruptions, including the user-requested pause/resume
boundary. The Contract records the exact scope and preserved evidence; this is
not a fabricated GitHub review or approval.

## Acceptance and verification

- Capture fresh baseline and candidate raw samples with the same toolchain,
  scenario, environment, and corrected order-preserving benchmark semantics.
- Retain warm-up count, sample count, percentile method, environment, and
  unavailable metrics; p95 requires at least 20 warm samples and p99 is unknown
  below 100 samples.
- Prove bounded decisions-directory scan reuse and preserve finalization,
  malformed-record, filename-digest, fork, missing-directory, isolation,
  error, evidence-binding, output-order, and exit-code behavior.
- Run the Runtime lifecycle and repository documentation/parity checks. A
  performance benefit is accepted only if the fresh target-path evidence
  covers the candidate and correctness remains equivalent; otherwise record a
  rejection without a benefit claim.

## Evidence and status

- predecessor audit: PR #671, `https://github.com/xinglun/ai-cockpit/pull/671`
- base: `origin/main` at `c1f1f1d9f17ec2242a59f287a4368bd6bda5993e`
- Runtime evidence: `.ai/evidence/WI-678-p1-historical-inventory-redelivery.verification.json`
- terminal records, when reached, are generated under `.ai/work-items/archive`
  and `.ai/decisions`

The Work Item remains `in_progress` until fresh measurements, correctness
checks, hosted review, and the terminal Runtime Outcome are all bound.
