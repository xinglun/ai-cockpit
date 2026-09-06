---
author: AI Cockpit maintainers
title: "WI-621 — reference installer / lifecycle safety parity"
description: "次の 18 maintained reference test path を一つずつ比較し、source implementation は copy しない。"
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

# WI-621 — reference installer / lifecycle safety parity

## Intent

次の 18 個の current reference test を一つずつ読み、evidence に裏付けられた Rust counterpart または明確な source-only boundary を記録します。目的は shared Runtime と attached adopter の semantic parity であり、Python、shell、wizard、fixture、source JSON implementation の copy ではありません。

## Boundary and decisions

Pinned local reference は commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`、Rust comparison baseline は
`8adac3379d8cb3e7a3dc59c70d6fb0b26176b990`、reviewed Runtime は公開済み
`ai-cockpit 0.2.83`（binary SHA256 `sha256:9f44d14278a614636ca47ee660656ce3b5eb5a0b969059b46d3132947310d130`）です。選択した path は既存 append-only ledger で current だが deferred でした。17 件は Rust Runtime、repository-native test、release/adopter harness、または documentation が意図した別設計で担い、interactive source wizard は `reference-only` のままです。`migrate-gap` はなく、retired history は immutable です。

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `tests/test_install_entrypoint.py` | implemented-different-by-design | Explicit release install、`attach --repo`、inspect/doctor、non-interactive fail-closed test。 |
| `tests/test_install_facts.py` | implemented-different-by-design | Typed release manifest/archive/SBOM identity と canonical fact test。 |
| `tests/test_install_script.py` | implemented-different-by-design | Published archive、SHA-256、source-archive policy、distribution check。 |
| `tests/test_install_sh.py` | implemented-different-by-design | Installation docs、release CLI test、workflow policy；source quick-install script は copy しない。 |
| `tests/test_install_status.py` | implemented-different-by-design | Release manifest identity と `doctor`/installed-lifecycle projection。 |
| `tests/test_install_wizard.py` | reference-only | Source interactive provider/stack wizard。Rust は explicit artifact install と repository binding を使う。 |
| `tests/test_installed_lifecycle_e2e.py` | implemented-different-by-design | Public/N-1 adopter acceptance と typed lifecycle evidence。 |
| `tests/test_installer_boundaries.sh` | implemented-different-by-design | Agent/repository ownership と explicit context isolation test。 |
| `tests/test_installer_conflict_matrix.py` | implemented-different-by-design | Attach/Agent ownership、safe path、symlink、traversal、trust boundary check。 |
| `tests/test_installer_detection.py` | implemented-different-by-design | Explicit compatibility、migration proposal、repository identity、active Work Item projection。 |
| `tests/test_installer_domains.py` | implemented-different-by-design | Read-only inspect/doctor/plan と explicit attach/migrate write boundary。 |
| `tests/test_installer_evidence.py` | implemented-different-by-design | Typed release handoff、manifest、delegated evidence/assurance record。 |
| `tests/test_installer_repository.py` | implemented-different-by-design | Git/Observer snapshot と request-scoped identity。 |
| `tests/test_installer_transaction.py` | implemented-different-by-design | Immutable archive validation、atomic repository write/lock、migration receipt、Agent isolation。 |
| `tests/test_issue_log.py` | implemented-different-by-design | Immutable Work Item evidence、structured decision、unknown、recovery lineage が source issue-log DB に代わる。 |
| `tests/test_lifecycle_facts.py` | implemented-different-by-design | Deterministic request-scoped status/observe/doctor projection。 |
| `tests/test_lifecycle_safety_gate.py` | implemented-different-by-design | Typed governance control、preflight review、operation-time fail-closed policy。 |
| `tests/test_negative_scenarios.py` | implemented-different-by-design | Rust adversarial/input-trust suite が unsafe refusal と safe alternative boundary を維持。 |

Source path は Runtime や adopter に copy しません。Source interactive prompt、provider API、Python module、Make target、fixture toolchain、source wire format は target boundary の外です。

## Adopter inheritance

Attached object/adopter は一つの shared external Runtime、explicit `--repo` context、isolated Protocol/Contract/evidence/knowledge、immutable release identity、fail-closed lifecycle、人間向け Outcome boundary を継承します。Source wizard、stack preset、issue-log storage、installer file は継承しません。

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

Contract acceptance は original language のまま保持します。Presentation chrome は localize できますが、governance fact と human decision は翻訳・創作しません。

See also: [English](WI-621-reference-installer-lifecycle-batch.md) · [中文](WI-621-reference-installer-lifecycle-batch.zh-CN.md)。
