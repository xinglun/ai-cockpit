---
author: AI Cockpit maintainers
title: "WI-516 — release・adoption・calibration・evidence 比較 batch 34"
description: "Python、package、provider の bytes をコピーせず、maintained reference の次の境界を一つずつ比較する。"
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

[English](WI-516-reference-file-comparison-batch-34.md) · [简体中文](WI-516-reference-file-comparison-batch-34.zh-CN.md)

## Goal

固定した local reference path を一つずつ読み、evidence に基づく Rust
counterpart または明示的な non-claim を記録する。release projection、Python
development metadata、adopter evidence、archive、baseline/cost observation、
calibration、capability truth、canonical evidence を対象とする。source の
Python、Shell、Make、package、provider state、interactive wizard、JSON wire
format はコピーしない。

## File-by-file decisions

| 固定 source path と digest | 分類 | Rust counterpart / non-claim |
| --- | --- | --- |
| `next-release.json` — `sha256:b5189750265e8b09350c153b47a9ffbff629042fe035a7dfe143b5e15c8949c2` | implemented-different-by-design | `crates/cockpit-release`、release workflow、version check、distribution docs が immutable artifact、checksum、SBOM/provenance、adopter acceptance を bind する。source candidate fields は Runtime wire ではない。 |
| `pyproject.toml` — `sha256:4d5ad0892ea3ee4bafc744c59a64dda3111d24ca6238873cf1107d537693c9c2` | implemented-different-by-design | Cargo metadata、lockfile、dynamic CI gate manifest が Python Ruff/mypy/coverage/pytest 設定を置き換える。Python tool 設定は source/provider fact のまま。 |
| `release-state.json` — `sha256:c747a4a6cb48190e55765eb76675f271af389c8db92b03efc720395844132f4c` | implemented-different-by-design | Rust release manifest/evidence が immutable tag、artifact、supply-chain、post-release state を保持する。source projection bookkeeping はコピーしない。 |
| `release.json` — `sha256:1e8ce44257efb4b8267bc30e6866a2ac085afad49bc621011599c1f2900615f8` | implemented-different-by-design | target release manifest と `SHA256SUMS` が target artifact を bind する。source URL、schema、historical release claim は移植しない。 |
| `requirements-dev.in` — `sha256:296d516b6548e2fa541e6eec23223a160bda0ea887d2ffccec8f50cfe550449c` | implemented-different-by-design | `Cargo.toml`、`Cargo.lock`、CI が Rust tooling を宣言する。adopter は自分の language toolchain を管理する。 |
| `requirements-dev.lock` — `sha256:b07fca668d49671422fb8213908d475b3698dd375ca3cfb03346d5ad51483537` | implemented-different-by-design | Cargo lock と Rust archive/supply-chain test が target の再現性境界を提供する。Python package hash は Runtime evidence ではない。 |
| `scripts/ai_adoption_evidence.py` — `sha256:87c883e556132cb759c792c4c106d112e2a0917222063f8a797658666d52e161` | implemented-different-by-design | public Release adopter acceptance が downloaded artifact、repository identity、isolation manifest、lifecycle evidence を bind する。source Work Item id と JSON wire はコピーしない。 |
| `scripts/ai_adoption_reality_report.py` — 現在の pinned checkout では retired | reference-only（historical） | inventory の retired ledger としてのみ確認し、current source file とは扱わない。source の historical Python report/evidence を Rust が継承するとは主張しない。 |
| `scripts/ai_archive_work_item.py` — `sha256:ceef1b14e6760a38b6873eeb971f6b20165fa831016e83393bdc52d8d7ec9324` | implemented-different-by-design | Rust archive/manifest/recovery/close service と archive-integrity test が immutable history と exact cleanup を保持し、Python path-rewrite helper は複製しない。 |
| `scripts/ai_baseline_evidence.py` — `sha256:ba47fbec6d2a9dbb66d43230dac5b25dbedbd9861726401a413828a69a4974a0` | implemented-different-by-design | Rust performance baseline、snapshot-bound verification、cost observation が identity と再現性を保持する。source Python coverage field は project-owned。 |
| `scripts/ai_calibrate.py` — `sha256:99a126a836b518c49d76349c286fc491fe1556652c36b1d22c676daf4b4af965` | implemented-different-by-design | typed project governance、`profile propose/confirm`、calibration docs が owner review、unknown、snapshot binding を保持する。source ten-stage Python session はコピーしない。 |
| `scripts/ai_calibration_corrective.py` — `sha256:6839e84e5309d32ad06b3e851a89eab5ddf1134bea2bf84f5c6692a65bf71635` | implemented-different-by-design | Rust profile/amendment validation と project-governance test が repository-bound corrective boundary を提供する。source session path は取り込まない。 |
| `scripts/ai_calibration_inventory.py` — `sha256:d0fff777e86e1746b393952c1f5ce96fb8cbe5b2570ca778d8b9fc56e6a50d164` | implemented-different-by-design | typed capability truth、profile fact、evidence assurance、external exclusion が source inventory aggregation を置き換える。source status key は universal protocol ではない。 |
| `scripts/ai_calibration_profiles.py` — `sha256:8c6be65cca8ee0340a113dcfb4120b395b8421d26dfcd4275d6fcdb21e21f8e7` | implemented-different-by-design | Rust proportional project policy と明示的な profile confirm が lite/standard/strict の意図を保持する。source YAML/selection bytes はコピーしない。 |
| `scripts/ai_calibration_wizard.py` — `sha256:63aa3f26f0cdd98c00ad88ffb1ec16e890f29dd18cbe16a360017ec00178d005` | implemented-different-by-design | CLI と reader-first calibration guide が reviewable な propose/confirm presentation を提供する。第二の provider interactive wizard は出荷しない。 |
| `scripts/ai_canonical_evidence.py` — `sha256:421c6ab34cc80ce1ac6f4b19cd4304a0491a9c38322c0aef8131ea13465dae28` | implemented-different-by-design | typed evidence、audit-export、digest、receipt、archive schema が deterministic identity/status を保持する。source canonical JSON/Markdown wire はコピーしない。 |
| `scripts/ai_capability_freshness.py` — `sha256:e6471b84dcab07396a4a24f3454b41ff55632e762ad6b3cfd41d41c26103a397` | implemented-different-by-design | capability projection は current repository snapshot と Runtime identity に bind する。toolchain/provider freshness は明示的な repository evidence とする。 |
| `scripts/ai_capability_truth.py` — `sha256:5cda977775e5b4fa6531886f963f1c8a4a976344ed974e34bcf39b58b1a3500e` | implemented-different-by-design | typed `CapabilityTruth`/`AdopterCapabilityTruth` が CLI/test から confidence、evidence refs、unknown、exclusion を示す。source matrix row と Python validator はコピーしない。 |

## Object/adopter inheritance boundary

adopter は shared Runtime の repository-bound attach、profile、Contract、
evidence、knowledge、capability、release acceptance、human Outcome boundary を
継承する。Python dependency、source release projection、calibration session、
provider credential、source JSON wire format は継承しない。各 adopter は自分の
project fact と明示的な verification evidence を用意する。

## Acceptance criteria

- batch 内の current path はすべて source digest、分類、counterpart または
  non-claim、inventory evidence を持つ。retired adoption-report path は
  historical/non-current として明示する。
- inventory は current 17 件を
  `WI-516-reference-file-comparison-batch-34` に所属させ、deferred または
  `migrate-gap` を残さない。
- source bytes、Python package behavior、provider state、object repository を
  変更しない。
- English、Simplified Chinese、日本語の comparison/parity 文書が同じ
  semantic/non-wire と adopter inheritance boundary を記載する。
- Contract の conformance、documentation、parity、workspace verification が
  成功する。

## Verification

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

reference checkout は `fde3380f81fea5fd2e288f7a8849f737dc074060` に固定し、network
source や source implementation をこの repository に追加しない。
