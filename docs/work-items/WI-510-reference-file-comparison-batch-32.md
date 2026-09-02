---
author: AI Cockpit maintainers
title: "WI-510 — installer entrypoint and wizard locale boundary"
description: "Compare four maintained reference installer/localization files without copying source installer or wizard implementation."
audience:
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
workItemId: WI-510-reference-file-comparison-batch-32
sourceCommit: fde3380f81fea5fd2e288f7a8849f737dc074060
lastVerifiedBy: WI-510-reference-file-comparison-batch-32
terminalArchive: .ai/work-items/archive/WI-510-reference-file-comparison-batch-32.contract.json
terminalVerification: .ai/evidence/WI-510-reference-file-comparison-batch-32.verification.json
terminalFinalization: .ai/decisions/WI-510-reference-file-comparison-batch-32.finalize.json
terminalDecision: .ai/decisions/WI-510-reference-file-comparison-batch-32.close.json
---

[简体中文](WI-510-reference-file-comparison-batch-32.zh-CN.md) · [日本語](WI-510-reference-file-comparison-batch-32.ja.md)

## Goal

Read the pinned reference `install.sh` and the English, Japanese, and Simplified Chinese wizard locale files one by one. Record an evidence-backed semantic decision and Rust counterpart for each path. This is a comparison and boundary task; it does not copy the source installer, Python wizard, locale bytes, or source JSON wire shape.

## File-by-file decisions

| Pinned path and source digest | Classification | Target boundary |
| --- | --- | --- |
| `install.sh` — `sha256:14f157f828e3ba8d1dd0886708b7eae223fe6d08` | implemented-different-by-design | Rust's immutable public Release, checksum/SBOM/provenance, explicit repository attachment, and isolated adopter acceptance preserve source selection, verification, cleanup, rollback, and isolation. There is no source Shell/Python installer or implicit target write. |
| `locales/wizard/en.json` — `sha256:1b9bfc3535e507c8478b071b641d974cb031e59e` | reference-only | Rust English Runtime labels and human Outcome sections are documented in the installation, command, and Outcome references. Interactive wizard prompts and session controls remain host/Agent adapter UX. |
| `locales/wizard/ja.json` — `sha256:8fab9ba89bd2bac5ccd51e8cb70dfea719435f5c` | reference-only | Rust Japanese Runtime presentation is documented; no second interactive installer is shipped and locale text cannot authorize repository changes. |
| `locales/wizard/zh-CN.json` — `sha256:591e11709864edf2846bfe63aab246b1dafd6473` | reference-only | Rust Chinese Runtime presentation is documented; source wizard bytes are not copied and cannot authorize repository changes. |

## Object/adopter inheritance boundary

Every object or adopter repository installs one shared Runtime externally and binds its own repository context with explicit `--repo`. It inherits the repository-local `attach`, Agent adapter, Contract, evidence, knowledge, and human Outcome boundaries. It does not inherit the source installer's implementation, stack-specific wizard, source locale JSON, or source provider decisions. Contract facts remain in their authoring language; only Runtime-owned presentation is localized.

## Acceptance criteria

- All four pinned paths are classified with source digests, reasons, and counterpart lists.
- Installer semantics are represented by Rust Release/distribution and adopter documentation without source code copying.
- Locale files remain reference-only while supported Runtime presentation and adapter responsibility are explicit.
- Inventory, comparison, parity, and this Work Item documentation stay synchronized in English, Chinese, and Japanese with no `migrate-gap`.
- Conformance, documentation, and workspace verification pass without changing object repositories, global Agent/MCP configuration, or unrelated Runtime behavior.

## Verification

The Contract-declared checks are:

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

The pinned reference checkout is local to the comparison process. No source checkout, locale file, or source installer is added to this repository.

## Terminal evidence

The generated archive, verification, finalization, and close receipts listed in the front matter are the authority for lifecycle status. The comparison page records the same four decisions and current inventory counts; historical evidence is not rewritten.
