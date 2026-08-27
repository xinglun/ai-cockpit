---
author: AI Cockpit maintainers
title: "Checks Catalog"
description: "Repository quality and governance checks with explicit evidence boundaries."
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-331-checks-release-evidence
keywords: [ai-cockpit, checks, governance, verification]
---

# Checks Catalog

The target catalog describes the checks that are actually available in this
Rust repository. It preserves the reference source's distinction between a
local quality check, a Work Item governance gate, hosted provider evidence, and
enterprise assurance. It is not a copy of the reference Make targets or
Python executors.

## Check layers

| Layer | Target entry point | What it proves | What it does not prove |
| --- | --- | --- | --- |
| Runtime Contract gate | `ai-cockpit gate --repo <path> --manifest tests/ci/repository_gate_manifest.json --stage <stage>` | The current Contract, repository snapshot, route, and selected gate manifest are internally consistent. | It does not execute hosted CI or grant enterprise assurance. |
| Local workspace quality | `cargo fmt --all -- --check`; `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` | Rust formatting and lint results for the checked-out workspace. | A local pass is not a reviewed PR or release result. |
| Package verification | `tests/ci/run_workspace_package_tests.sh --report <path>` | Deterministic package test coverage and its receipt. | It does not prove provider-side branch protection or publication. |
| Conformance and documentation | `tests/conformance/reference_file_inventory_test.sh`; `tests/docs/documentation_acceptance.sh` | Reference-ledger, reader-route, and documentation invariants. | Documentation text is not a substitute for executable evidence. |
| Release and adopter | `tests/release/*` through the strict manifest route | Artifact identity, checksums, SBOM/provenance bindings, and isolated adopter lifecycle where the named harness runs. | A staged or local result is not public Release evidence unless the provider receipt says so. |

The canonical set and profile floors are versioned in
`tests/ci/repository_gate_manifest.json`. The route is cumulative:
`light` covers documentation and low-cost policy checks, `standard` adds the
Rust workspace and conformance checks, and `strict` adds release, workflow,
performance, and adopter checks. Changed paths, Contract risk, and lifecycle
stage select the minimum profile. Unknown or release-owned input escalates to
`strict`; a caller cannot lower a selected profile by passing a faster command.

`VerificationTier` (the strength of an executed check) and
`EvidenceAssurance` (who can vouch for the result) are independent. A strict
local check is not automatically provider- or enterprise-verified.

## Evidence ownership

Runtime receipts bind the repository, Work Item, Contract, snapshot, selected
route, and Runtime identity. Hosted CI owns provider run/job conclusions and
external branch or merge observations. The public Release owns published
archive, checksum, SBOM, provenance, and attestation facts. Enterprise systems
own identity, retention, WORM/SIEM, and organizational approval. AI Cockpit can
require, bind, validate, display, and archive delegated evidence; it does not
forge any of those external claims.

All checks remain subordinate to the active Contract, preflight review,
required scenario evidence, human decisions, and reviewed PR lifecycle. A
green local check is useful evidence, not an authorization to skip a required
gate or to claim production readiness.

## Failure and recovery

Missing, malformed, stale, foreign, or contradictory receipts fail closed.
Keep the failed command, its source revision, output receipt, and any provider
run identity for diagnosis. Rerun the named check after repairing the bounded
cause; do not replace a failed result with an unpinned command or a source-built
Runtime. The repository's object-engineering adopter supplies its own stack
commands, while every AI Cockpit invocation remains explicitly bound with
`--repo <path>`.
