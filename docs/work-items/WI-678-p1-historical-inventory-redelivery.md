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
authority. The remote default later advanced to `8bf7a301`; the final
experiment was rebased and remeasured on that base before deciding whether to
accept the candidate.

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

The latest-base 40-warm-sample round rejects the candidate: status p50/p95 are
`3592.196/4695.920 ms` for baseline versus `3911.300/8195.299 ms` for
candidate, and candidate doctor p95 is `105.029 ms` against the `100 ms`
budget. The normalized governance outputs and exit codes remain equal. This
is a measured rejection, not a production performance claim; the candidate
PR is not merge-authorized.

## Evidence and status

- predecessor audit: PR #671, `https://github.com/xinglun/ai-cockpit/pull/671`
- initial base: `origin/main` at `c1f1f1d9f17ec2242a59f287a4368bd6bda5993e`
- final measured base: `origin/main` at `8bf7a301`
- Runtime evidence: `.ai/evidence/WI-678-p1-historical-inventory-redelivery.verification.json`
- latest-base decision: `.ai/evidence/external/WI-678-p1-historical-inventory-redelivery.latest-base.rejection.json`
- latest-base comparability: `.ai/evidence/external/WI-678-p1-historical-inventory-redelivery.latest-base.comparability.json`
- terminal records, when reached, are generated under `.ai/work-items/archive`
  and `.ai/decisions`

The Work Item remains `in_progress` until the rejected experiment, hosted
checks, and the terminal Runtime Outcome are all bound. The WI-678 parity
registration and governance-integrity check pass; repository-wide
documentation acceptance and status consistency remain blocked by the
concurrently maintained WI-674 document retaining `status: in_progress`.
That out-of-scope failure is preserved in
`.ai/evidence/external/WI-678-p1-historical-inventory-redelivery.documentation-regression.json`.
The closed-Work-Item promotion check is bound by the same evidence.
No production change is accepted from this candidate.
