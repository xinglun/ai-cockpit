---
author: AI Cockpit maintainers
title: "WI-617 — Reference Outcome, Event, and Human Handoff Parity"
description: "File-level semantic comparison of maintained reference Outcome, event, and human-handoff tests."
audience: [maintainer, reviewer, adopter]
workItemId: WI-617-reference-outcome-parity
status: implemented
authority: canonical
lastVerifiedBy: WI-617-reference-outcome-parity
terminalArchive: .ai/work-items/archive/WI-617-reference-outcome-parity.contract.json
terminalVerification: .ai/evidence/WI-617-reference-outcome-parity.verification.json
terminalFinalization: .ai/decisions/WI-617-reference-outcome-parity.finalize.json
terminalDecision: .ai/decisions/WI-617-reference-outcome-parity.close.json
---

# WI-617 — Reference Outcome, Event, and Human Handoff Parity

## Intent

Compare the maintained reference Outcome, event-log, multilingual, unsupported-
claim, schema, generator, renderer, and validator tests one file at a time.
Record the Rust-native counterpart and the explicit boundary for each path so a
future release cannot silently leave a portable omission.

## Boundary

The pinned source is the local reference checkout at
`fde3380f81fea5fd2e288f7a8849f737dc074060`. This is a semantic comparison,
not source-file, Python, Make, provider-output, or JSON-wire copying. The Rust
Runtime owns repository-bound evidence and human Outcome truth; adapters may
provide provider-specific presentation without becoming a second authority.

## File-level decisions

The eleven source paths and their counterparts are recorded in the
[machine-readable inventory](../../tests/conformance/reference_file_inventory.json)
and summarized in the [reference comparison ledger](../reference/reference-file-comparison.md#wi-617--reference-outcome-event-and-human-handoff-parity).
Ten paths are `implemented-different-by-design`. The provider/adapter PR summary
projection is `not-applicable`; the canonical Runtime surfaces are typed
OutcomeV2, task Outcome reports/events, and the visible human handoff.

The attached object/adopter route inherits the shared Runtime, explicit
repository context, isolated Contract/evidence/knowledge, fail-closed evidence,
and human Outcome boundary. It does not inherit source Python tests, Make
targets, provider PR formatting, or source wire formats.

## Verification

Run from this repository with the explicit repository context:

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit cf361147ede1f37fb8929eba26a78abffbd943b2
bash tests/conformance/reference_file_inventory_test.sh
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

The Runtime verification evidence and terminal Outcome are authoritative for
completion. Contract acceptance text remains in its original language; only
fixed presentation chrome is localized.

See also: [中文](WI-617-reference-outcome-parity.zh-CN.md) ·
[日本語](WI-617-reference-outcome-parity.ja.md).
