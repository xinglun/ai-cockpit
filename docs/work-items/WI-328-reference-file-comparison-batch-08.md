---
author: AI Cockpit maintainers
title: "WI-328 — reference file comparison batch 08"
workItemId: WI-328-reference-file-comparison-batch-08
description: "Compare nine pinned calibration and capability reference paths one file at a time and record explicit Rust-native boundaries."
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-328-reference-file-comparison-batch-08
terminalArchive: .ai/work-items/archive/WI-328-reference-file-comparison-batch-08.contract.json
terminalVerification: .ai/evidence/WI-328-reference-file-comparison-batch-08.verification.json
terminalFinalization: .ai/decisions/WI-328-reference-file-comparison-batch-08.finalize.json
terminalDecision: .ai/decisions/WI-328-reference-file-comparison-batch-08.close.json
---

# WI-328 — reference file comparison batch 08

## Intent and boundary

Compare the nine paths below from pinned reference commit
e5acb677da6621004d96f0ef353c58fe8d3acfbf one file at a time. Preserve
adopter-facing calibration and capability-truth responsibilities without
copying source Python, Make, wizard, or matrix bytes.

The shared Rust Runtime remains external and every repository request is bound
with an explicit --repo. This is a documentation and conformance-ledger
batch; it does not add Runtime commands or claim source wire compatibility.

## File-level comparison

| Pinned reference path | Classification | Rust/adopter counterpart and boundary |
| --- | --- | --- |
| docs/reference/calibration-session-model.md | implemented-different-by-design | Repository-bound profile proposal, human confirmation, and explicit calibration facts preserve the source fact/evidence boundary. The target does not introduce a generic persisted Session or treat a proposal as active policy. |
| docs/reference/calibration-session-model.zh-CN.md | implemented-different-by-design | The same repository-bound proposal/confirmation boundary is documented for Chinese readers; unknowns and human authority remain visible. |
| docs/reference/calibration-session.ja.md | implemented-different-by-design | The source ten-stage interactive Session is represented only by the target explicit profile proposal and confirmation route; source Make/Python and enterprise/security claims are not copied. |
| docs/reference/calibration-session.md | implemented-different-by-design | The source persisted ten-stage wizard is source-specific orchestration. Target calibration stays read-only-first and repository-bound, with human confirmation required for policy changes. |
| docs/reference/canonical-terminology.md | implemented-different-by-design | .ai/glossary.md, configuration, and Outcome reference pages provide canonical terms. Governance light is not a hidden alias for a source Calibration lite, and release is an operation rather than a profile. |
| docs/reference/capability-claim-authoring.md | reference-only | The source lexical claim checker and matrix-binding front matter are not a target Runtime gate. The target registry reports observed repository facts and explicit exclusions; a future capability-claim/evidence boundary is tracked as candidate WI-329. |
| docs/reference/capability-evidence-freshness.md | reference-only | Rust validates Work Item verification freshness and identity-bound receipts, but does not ship a separate Capability Truth row expiry or portable-environment matrix. Candidate WI-329 owns any such extension. |
| docs/reference/capability-truth-matrix.json | reference-only | The source 30-row public matrix is not copied. capability_truth_registry is a request-scoped observed-capability projection, not public claim authorization or adopter/provider proof. |
| docs/reference/capability-truth-matrix.md | reference-only | Current capability and adoption pages state the observed-fact, adopter-installation, provider-evidence, and enterprise-assurance boundaries. No source matrix or claim checker is advertised until a bounded successor is approved. |

The four reference-only entries are an explicit product boundary, not an
unrecorded omission. Candidate WI-329 is intentionally not started in this
batch; it would need a human-owned scope for a Rust-native claim/evidence
matrix, freshness policy, strict multilingual binding checks, and their
adopter-facing documentation. Source Python/Make checkers are not copied.

## Cursor adopter feedback reconciliation

The Cursor report is external adopter evidence, not a new source wire contract.
Current Runtime evidence already covers:

- stable lifecycle JSON on stdout plus the replayable work-item outcome;
- close-before-next entry checks and explicit readyOnBase;
- fail-closed start checks for a dirty or unsynchronized base;
- verification invalidation after relevant changes; and
- repository-local Agent adapter installation with no automatic chat posting.

Cursor cannot be forced by the Runtime to expand an IDE chat panel. The
adapter/host must surface or replay the durable human handoff. Detailed
mismatch remediation, controls scaffolding, and a close-gap convenience
command are useful follow-up product work, but are not silently claimed as
current parity. This repository also deliberately has no Makefile.ai
requirement; explicit --repo CLI/MCP commands are the adopter interface.

## Non-goals

This Work Item does not add Runtime behavior, copy source Python/Make/YAML,
introduce a generic calibration wizard, add a public claim matrix, require
Makefile integration, modify global Agent/MCP configuration, or change the
pinned source/target commits.

## Acceptance and evidence

1. All nine pinned paths are read and have exactly one inventory record with a
   non-empty, evidence-backed reason.
2. The inventory records five implemented-different-by-design and four
   reference-only entries for WI-328, with no deferred or hidden
   classification.
3. English, Simplified Chinese, and Japanese comparison/parity pages and this
   Work Item agree on the source pin, classifications, Cursor boundaries, and
   candidate WI-329 follow-up.
4. The target does not claim a generic ten-stage Session, source
   Python/Make execution, a public capability-claim matrix, provider identity,
   or enterprise assurance without target evidence.
5. Installed Runtime inspect/status/doctor/agent doctor, focused
   documentation/conformance checks, lifecycle closure, hosted CI, and exact
   cleanup provide terminal evidence. Historical evidence is not rewritten.

[简体中文](WI-328-reference-file-comparison-batch-08.zh-CN.md) · [日本語](WI-328-reference-file-comparison-batch-08.ja.md)
