---
author: AI Cockpit maintainers
title: "WI-619 — Reference adoption, enterprise, lifecycle, and trust test parity"
description: "File-level semantic comparison of the next maintained reference test batch."
audience: [maintainer, reviewer, adopter]
workItemId: WI-619-reference-script-semantic-batch
status: implemented
authority: canonical
lastVerifiedBy: WI-619-reference-script-semantic-batch
terminalArchive: .ai/work-items/archive/WI-619-reference-script-semantic-batch.contract.json
terminalVerification: .ai/evidence/WI-619-reference-script-semantic-batch.verification.json
terminalFinalization: .ai/decisions/WI-619-reference-script-semantic-batch.finalize.json
terminalDecision: .ai/decisions/WI-619-reference-script-semantic-batch.close.json
---

# WI-619 — Reference adoption, enterprise, lifecycle, and trust test parity

## Intent

Compare the next 21 maintained reference test paths one file at a time and
record an evidence-backed Rust counterpart or an explicit source/provider
boundary. This batch covers adoption, locked environments, enterprise evidence,
dependency projections, lifecycle readiness, governance guards, hosted
verification, and input trust.

## Boundary

The pinned local reference is commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`. This is semantic parity, not source
Python, Make, provider configuration, fixture matrix, or source JSON-wire
copying. The shared Rust Runtime and repository-native tests own portable
governance; provider and adopter environments retain their own toolchains and
external assurance.

## Decisions

All 21 paths are recorded in the [machine inventory](../../tests/conformance/reference_file_inventory.json)
as `implemented-different-by-design`. The detailed per-file table is in the
[reference comparison ledger](../reference/reference-file-comparison.md#wi-619--reference-adoption-enterprise-lifecycle-and-trust-test-parity).
No `migrate-gap` was found. The attached object/adopter route inherits the
shared Runtime, explicit repository context, isolated Contract/evidence/
knowledge, dynamic verification, fail-closed lifecycle, and human Outcome
boundary; it does not inherit source fixture stacks or provider-specific
commands.

## Verification

Run with an explicit repository context:

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/conformance/reference_inventory_docs_test.py
cargo test --locked --workspace
```

Contract acceptance remains in its original language. Only fixed presentation
chrome is localized; no unsupported benefit or compliance claim is introduced.

See also: [中文](WI-619-reference-script-semantic-batch.zh-CN.md) ·
[日本語](WI-619-reference-script-semantic-batch.ja.md).
