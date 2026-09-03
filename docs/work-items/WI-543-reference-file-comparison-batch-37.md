---
author: AI Cockpit maintainers
title: "WI-543 — reference file comparison batch 37"
description: "Safe conformance-ledger checking and seven source checker comparisons."
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
workItemId: WI-543-reference-ledger-check-safety
---

# WI-543 — Reference file comparison batch 37

## Objective

Compare the next seven maintained reference checker modules at pinned source
commit `fde3380f81fea5fd2e288f7a8849f737dc074060`, while making the inventory
checker safe in read-only `--check` mode. The reference remains a behavior and
specification corpus; its Python, Make, YAML, provider, and source JSON wire
implementations are not copied into the Rust Runtime.

## File-level result

| Reference path | Classification | Rust boundary |
| --- | --- | --- |
| `scripts/ai_check_task_outcome.py` | `implemented-different-by-design` | Typed OutcomeV2/TaskOutcomeReport, append-only events, localized human handoff, and archive binding provide the portable boundary; source report wire shape and lexical policy are not copied. |
| `scripts/ai_check_test_weakening.py` | `implemented-different-by-design` | Snapshot-based Rust governance signals and fail-closed unknowns cover the portable weakening boundary; source thresholds and maintenance report format remain source/provider policy. |
| `scripts/ai_classify_operation_impact.py` | `implemented-different-by-design` | Operation-time policy and scope evaluation provide explicit impact facts without inferring intent or importing the source report format. |
| `scripts/ai_close_work_item.py` | `implemented-different-by-design` | Typed lifecycle/finalization/ready-on-base gates enforce closure; provider PR operations and source runner orchestration remain external. |
| `scripts/ai_common.py` | `implemented-different-by-design` | JSON/Git/scope/redaction concerns are distributed across typed Core, Protocol, repository, and conformance services rather than a copied helper module. |
| `scripts/ai_critical_domain_guards.py` | `implemented-different-by-design` | Typed operation, authority, prompt-injection, and evidence-forgery controls preserve fail-closed governance without promoting lexical classification to authority. |
| `scripts/ai_dependabot_intake.py` | `not-applicable` | Dependabot event identity and bot-branch intake are provider-specific; generic delegated evidence and source binding remain available. |

## Ledger safety

`reference_file_inventory.py --check` is strictly read-only. It rejects
generation, rebaseline, and apply options before loading or writing a manifest,
so an accidental combined invocation cannot replace append-only retired
history with a fresh generated projection. The regression wrapper checks both
the rejection and byte identity of the manifest.

Historical and retired records are validated as immutable records; only the
current pinned path set is eligible for a new batch decision. This keeps source
renames, removals, and rebaseline deltas from reopening completed comparisons.

## Adopter inheritance

Every attached object project inherits the same shared Runtime, explicit
`--repo` context, isolated Contract/evidence/knowledge, fail-closed lifecycle,
and human Outcome handoff. It does not inherit source checker modules,
Dependabot/provider events, source policy values, or source JSON wire formats.

## Verification

- `python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --locked --workspace --all-targets --all-features -- --test-threads=1`
