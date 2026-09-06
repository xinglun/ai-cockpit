---
author: AI Cockpit maintainers
title: "WI-620 — Reference installer, release, quality, and lifecycle parity"
description: "File-level semantic comparison of the remaining source-changed reference tests."
audience: [maintainer, reviewer, adopter]
workItemId: WI-620-reference-release-governance-batch
status: in-progress
authority: canonical
lastVerifiedBy: WI-620-reference-release-governance-batch
---

# WI-620 — Reference installer, release, quality, and lifecycle parity

## Intent

Re-read the remaining 22 source-changed reference test paths one file at a
time and record an evidence-backed Rust counterpart or an explicit boundary.
The goal is semantic coverage for the Runtime and attached adopters, not a
copy of source Python, Make, provider configuration, or source JSON wire.

## Boundary and decisions

The pinned local reference is commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`; the target comparison commit is
`cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd`. Twenty-one paths are
`implemented-different-by-design`; `tests/test_install_plan.py` is
`reference-only` because Rust uses immutable Release artifacts plus explicit
`attach --repo`, not the source's interactive stack/provider wizard. No path
is a `migrate-gap`.

The detailed file-level ledger is in
[reference-file-comparison.md](../reference/reference-file-comparison.md#wi-620--reference-installer-release-quality-and-lifecycle-test-parity)
and the machine record is
[reference_file_inventory.json](../../tests/conformance/reference_file_inventory.json).
The 22 paths cover installed-runtime parity, installer behavior, Make/CI
surfaces, PR aggregation, project governance, quality architecture and
measurements, reference impact, release distribution/preflight/state/workflow,
start/archive, supply chain, published-release projection, verification policy,
Work Item intelligence/lifecycle, and workflows.

## Adopter inheritance

Attached object/adopter projects inherit the shared external Runtime, explicit
repository context, isolated Contract/evidence/knowledge, dynamic verification,
fail-closed lifecycle and visible human Outcome boundary. They do not inherit
source Python tests, Make targets, interactive stack installers, provider policy
values, fixture stacks, or source wire formats. This is semantic parity and not
source implementation or wire compatibility.

## Verification

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd
bash tests/conformance/reference_file_inventory_test.sh
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/conformance/reference_inventory_docs_test.py
python3 tests/docs/reference_comparison_metadata_test.py
cargo test --locked --workspace
```

Contract acceptance remains in its original language. Presentation chrome may
be localized, but governance facts and human decisions are not translated or
invented.

See also: [中文](WI-620-reference-release-governance-batch.zh-CN.md) ·
[日本語](WI-620-reference-release-governance-batch.ja.md).
