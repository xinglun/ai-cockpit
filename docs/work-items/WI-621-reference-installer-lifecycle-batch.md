---
author: AI Cockpit maintainers
title: "WI-621 — Reference installer and lifecycle safety parity"
description: "Compare the next eighteen maintained reference test paths without copying source implementation."
audience: [maintainer, reviewer, adopter]
workItemId: WI-621-reference-installer-lifecycle-batch
status: implemented
authority: canonical
lastVerifiedBy: WI-621-reference-installer-lifecycle-batch
terminalArchive: .ai/work-items/archive/WI-621-reference-installer-lifecycle-batch.contract.json
terminalVerification: .ai/evidence/WI-621-reference-installer-lifecycle-batch.verification.json
terminalFinalization: .ai/decisions/WI-621-reference-installer-lifecycle-batch.finalize.json
terminalDecision: .ai/decisions/WI-621-reference-installer-lifecycle-batch.close.json
---

# WI-621 — Reference installer and lifecycle safety parity

## Intent

Read the next eighteen current reference tests one at a time and record an
evidence-backed Rust counterpart or a precise source-only boundary. The target
is semantic parity for the shared Runtime and attached adopters, not a copy of
Python, shell, wizard, fixture, or source JSON implementation.

## Boundary and decisions

The pinned local reference is commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`; the Rust comparison baseline is
`8adac3379d8cb3e7a3dc59c70d6fb0b26176b990`, using the published `ai-cockpit`
`0.2.83` binary (`sha256:9f44d14278a614636ca47ee660656ce3b5eb5a0b969059b46d3132947310d130`).
The selected paths were current but deferred in the existing append-only
ledger. Seventeen responsibilities are implemented differently by the Rust
Runtime, repository-native tests, release/adopter harnesses, or documentation;
the interactive source wizard remains `reference-only`. No `migrate-gap` was
found, and retired reference history remains immutable.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `tests/test_install_entrypoint.py` | implemented-different-by-design | Explicit release installation, `attach --repo`, inspect/doctor, and non-interactive fail-closed tests. |
| `tests/test_install_facts.py` | implemented-different-by-design | Typed release manifest/archive/SBOM identity and canonical fact tests. |
| `tests/test_install_script.py` | implemented-different-by-design | Published archive, SHA-256, source-archive policy, and distribution checks. |
| `tests/test_install_sh.py` | implemented-different-by-design | Installation documentation, release CLI tests, and workflow policy; no source quick-install script. |
| `tests/test_install_status.py` | implemented-different-by-design | Release manifest identity plus `doctor` and installed-lifecycle projections. |
| `tests/test_install_wizard.py` | reference-only | Source interactive provider/stack wizard; Rust intentionally uses explicit artifact installation and repository binding. |
| `tests/test_installed_lifecycle_e2e.py` | implemented-different-by-design | Public/N-1 adopter acceptance and typed lifecycle evidence. |
| `tests/test_installer_boundaries.sh` | implemented-different-by-design | Agent/repository ownership and explicit context isolation tests. |
| `tests/test_installer_conflict_matrix.py` | implemented-different-by-design | Attach/Agent ownership, safe paths, symlink, traversal, and trust-boundary checks. |
| `tests/test_installer_detection.py` | implemented-different-by-design | Explicit compatibility, migration proposal, repository identity, and active-Work-Item projections. |
| `tests/test_installer_domains.py` | implemented-different-by-design | Read-only inspect/doctor/planning versus explicit attach/migrate write boundaries. |
| `tests/test_installer_evidence.py` | implemented-different-by-design | Typed release handoff, manifest, and delegated evidence/assurance records. |
| `tests/test_installer_repository.py` | implemented-different-by-design | Git/Observer snapshots and repository facts with request-scoped identity. |
| `tests/test_installer_transaction.py` | implemented-different-by-design | Immutable archive validation, atomic repository writes/locks, migration receipts, and Agent isolation. |
| `tests/test_issue_log.py` | implemented-different-by-design | Immutable Work Item evidence, structured decisions, unknowns, and recovery lineage replace a source issue-log database. |
| `tests/test_lifecycle_facts.py` | implemented-different-by-design | Deterministic request-scoped status/observe/doctor projections. |
| `tests/test_lifecycle_safety_gate.py` | implemented-different-by-design | Typed governance controls, preflight review, and operation-time fail-closed policy. |
| `tests/test_negative_scenarios.py` | implemented-different-by-design | Rust adversarial and input-trust suites preserve unsafe refusal and safe-alternative boundaries. |

The source paths are not copied into the Runtime or an adopter. Source
interactive prompts, provider APIs, Python modules, Make targets, fixture
toolchains, and source wire formats remain outside the target boundary.

## Adopter inheritance

An attached object/adopter repository inherits the one shared external Runtime,
explicit `--repo` context, isolated Protocol/Contract/evidence/knowledge,
immutable release identity, fail-closed lifecycle, and human Outcome boundary.
It does not inherit the source wizard, stack presets, issue-log storage, or
source installer files.

## Verification

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit 8adac3379d8cb3e7a3dc59c70d6fb0b26176b990
bash tests/conformance/reference_file_inventory_test.sh
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/conformance/reference_inventory_docs_test.py
python3 tests/docs/reference_comparison_metadata_test.py
cargo test --locked --workspace
```

Contract acceptance remains in its original language. Presentation chrome may
be localized, but source governance facts and human decisions are not
translated or invented.

See also: [中文](WI-621-reference-installer-lifecycle-batch.zh-CN.md) ·
[日本語](WI-621-reference-installer-lifecycle-batch.ja.md).
