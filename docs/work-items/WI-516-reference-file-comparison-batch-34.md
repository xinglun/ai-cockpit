---
author: AI Cockpit maintainers
title: "WI-516 — release, adoption, calibration, and evidence comparison batch 34"
description: "A bounded one-by-one comparison of the next maintained reference-source surfaces without copying Python, packaging, or provider bytes."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-516-reference-file-comparison-batch-34
sourceCommit: fde3380f81fea5fd2e288f7a8849f737dc074060
lastVerifiedBy: WI-516-reference-file-comparison-batch-34
terminalArchive: .ai/work-items/archive/WI-516-reference-file-comparison-batch-34.contract.json
terminalVerification: .ai/evidence/WI-516-reference-file-comparison-batch-34.verification.json
terminalFinalization: .ai/decisions/WI-516-reference-file-comparison-batch-34.finalize.json
terminalDecision: .ai/decisions/WI-516-reference-file-comparison-batch-34.close.json
---

[简体中文](WI-516-reference-file-comparison-batch-34.zh-CN.md) · [日本語](WI-516-reference-file-comparison-batch-34.ja.md)

## Goal

Read the pinned local reference paths one by one and record an evidence-backed
Rust counterpart or an explicit non-claim. This batch covers release
projections, Python development metadata, adopter evidence, archive behavior,
baseline/cost observation, calibration, capability truth, and canonical
evidence. It does not copy source Python, Shell, Make, packaging, provider
state, interactive wizard, or JSON-wire formats.

## File-by-file decisions

| Pinned source path and digest | Classification | Rust counterpart / non-claim |
| --- | --- | --- |
| `next-release.json` — `sha256:b5189750265e8b09350c153b47a9ffbff629042fe035a7dfe143b5e15c8949c2` | implemented-different-by-design | `crates/cockpit-release`, release workflow, version checks, and distribution docs bind immutable candidate/public artifacts, checksums, SBOM/provenance, and adopter acceptance. Source candidate fields are not Runtime wire data. |
| `pyproject.toml` — `sha256:4d5ad0892ea3ee4bafc744c59a64dda3111d24ca6238873cf1107d537693c9c2` | implemented-different-by-design | Cargo metadata, lockfile, and the dynamic CI gate manifest replace Python Ruff/mypy/coverage/pytest configuration. Python tool settings remain source/provider facts. |
| `release-state.json` — `sha256:c747a4a6cb48190e55765eb76675f271af389c8db92b03efc720395844132f4c` | implemented-different-by-design | Rust release manifests and release evidence preserve immutable tag, artifact, supply-chain, and post-release state; source projection bookkeeping is not copied. |
| `release.json` — `sha256:1e8ce44257efb4b8267bc30e6866a2ac085afad49bc621011599c1f2900615f8` | implemented-different-by-design | Target release manifests and `SHA256SUMS` bind this repository's assets. Source URLs, schema, and historical release claims are not portable. |
| `requirements-dev.in` — `sha256:296d516b6548e2fa541e6eec23223a160bda0ea887d2ffccec8f50cfe550449c` | implemented-different-by-design | `Cargo.toml`, `Cargo.lock`, and CI declare Rust tooling. Adopters own their language-specific toolchains. |
| `requirements-dev.lock` — `sha256:b07fca668d49671422fb8213908d475b3698dd375ca3cfb03346d5ad51483537` | implemented-different-by-design | Cargo lock and Rust archive/supply-chain tests provide target reproducibility; Python package hashes are not Runtime evidence. |
| `scripts/ai_adoption_evidence.py` — `sha256:87c883e556132cb759c792c4c106d112e2a0917222063f8a797658666d52e161` | implemented-different-by-design | Public-release adopter acceptance binds downloaded artifact, repository identity, isolation manifests, and lifecycle evidence through Rust release tooling. Source Work Item IDs and wire shape are not copied. |
| `scripts/ai_adoption_reality_report.py` — retired from the current pinned checkout | reference-only (historical) | The retired path is checked against the inventory retired ledger and is not treated as a current source file. Rust does not claim the source report's historical Python implementation or evidence. |
| `scripts/ai_archive_work_item.py` — `sha256:ceef1b14e6760a38b6873eeb971f6b20165fa831016e83393bdc52d8d7ec9324` | implemented-different-by-design | Rust archive/manifest/recovery/close services and archive-integrity tests preserve immutable history and exact cleanup without duplicating Python path-rewrite helpers. |
| `scripts/ai_baseline_evidence.py` — `sha256:ba47fbec6d2a9dbb66d43230dac5b25dbedbd9861726401a413828a69a4974a0` | implemented-different-by-design | Rust performance baselines, snapshot-bound verification, and cost observations preserve identity and reproducibility; source Python coverage fields remain project-owned. |
| `scripts/ai_calibrate.py` — `sha256:99a126a836b518c49d76349c286fc491fe1556652c36b1d22c676daf4b4af965` | implemented-different-by-design | Typed project governance plus `profile propose/confirm` and calibration docs preserve explicit owner review, unknowns, and repository snapshot binding. The source ten-stage Python session is not copied. |
| `scripts/ai_calibration_corrective.py` — `sha256:6839e84e5309d32ad06b3e851a89eab5ddf1134bea2bf84f5c6692a65bf71635` | implemented-different-by-design | Rust profile/amendment validation and project-governance tests provide the repository-bound corrective boundary; source session paths are not imported. |
| `scripts/ai_calibration_inventory.py` — `sha256:d0fff777e86e1746b393952c1f5ce96fb8cbe5b2570ca778d8b9fc56e6a50d164` | implemented-different-by-design | Typed capability truth, profile facts, evidence assurance, and explicit external exclusions replace source inventory aggregation; source status keys are not a universal protocol. |
| `scripts/ai_calibration_profiles.py` — `sha256:8c6be65cca8ee0340a113dcfb4120b395b8421d26dfcd4275d6fcdb21e21f8e7` | implemented-different-by-design | Rust proportional project policies and explicit profile confirmation preserve lite/standard/strict intent without copying source YAML or selection bytes. |
| `scripts/ai_calibration_wizard.py` — `sha256:63aa3f26f0cdd98c00ad88ffb1ec16e890f29dd18cbe16a360017ec00178d005` | implemented-different-by-design | The CLI and reader-first calibration guide provide a reviewable propose/confirm presentation. A second interactive provider wizard is deliberately not shipped. |
| `scripts/ai_canonical_evidence.py` — `sha256:421c6ab34cc80ce1ac6f4b19cd4304a0491a9c38322c0aef8131ea13465dae28` | implemented-different-by-design | Typed evidence, audit-export, digest, receipt, and archive schemas preserve deterministic identity/status semantics; source canonical JSON and Markdown wire formats are not copied. |
| `scripts/ai_capability_freshness.py` — `sha256:e6471b84dcab07396a4a24f3454b41ff55632e762ad6b3cfd41d41c26103a397` | implemented-different-by-design | Capability projections bind to the current repository snapshot and Runtime identity. Toolchain/provider freshness remains explicit repository evidence, not an inferred Runtime claim. |
| `scripts/ai_capability_truth.py` — `sha256:5cda977775e5b4fa6531886f963f1c8a4a976344ed974e34bcf39b58b1a3500e` | implemented-different-by-design | Typed `CapabilityTruth`/`AdopterCapabilityTruth` expose confidence, evidence refs, unknowns, and exclusions through CLI and repository tests; source matrix rows and Python validators are not copied. |

## Object/adopter inheritance boundary

An adopter inherits the shared Runtime's repository-bound attach, profile,
Contract, evidence, knowledge, capability, release-acceptance, and human
Outcome boundaries. It does not inherit Python dependencies, source release
projections, source calibration sessions, provider credentials, or source JSON
wire formats. Every adopter must supply its own project facts and explicit
verification evidence.

## Acceptance criteria

- Every current path in this batch has a source digest, classification,
  counterpart or explicit non-claim, and inventory evidence. The retired
  adoption-report path is explicitly checked as historical/non-current.
- The inventory owns all 17 current decisions under
  `WI-516-reference-file-comparison-batch-34`; none remains deferred or a
  `migrate-gap`.
- No source bytes, Python packaging behavior, provider state, or object
  repository is changed.
- English, Simplified Chinese, and Japanese comparison/parity documentation
  states the same semantic/non-wire and adopter-inheritance boundaries.
- Conformance, documentation, parity, and workspace verification pass from
  the Contract-declared commands.

## Verification

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

The reference checkout is local and pinned to
`fde3380f81fea5fd2e288f7a8849f737dc074060`; no network source or source
implementation is added to this repository.
