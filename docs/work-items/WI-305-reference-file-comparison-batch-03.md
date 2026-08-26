---
author: AI Cockpit maintainers
title: "WI-305 — reference architecture installation and verification batch 03"
workItemId: WI-305-reference-file-comparison-batch-03
description: "Compare four pinned reference architecture files and record the Rust/adopter boundary without copying the source installer or Wizard."
audience:
  - maintainer
  - reviewer
status: in progress
lastVerifiedBy: WI-305-reference-file-comparison-batch-03
authority: canonical
---

# WI-305 — reference architecture installation and verification batch 03

## Intent and goal

Compare the next four deferred reference architecture documents one file at a
time. Establish whether the Rust Runtime and an adopter repository inherit the
reference responsibilities for installation detection, interactive wizard
boundaries, lightweight verification/soft gates, and Wizard IO/localization.
Record a counterpart or an explicit reference-only/external boundary; do not
copy the source Python, Make, Installer, or Wizard implementation.

## Scope and boundary

In scope:

- `docs/architecture/installation-detection-boundary.md`
- `docs/architecture/interactive-installation-wizard.md`
- `docs/architecture/lightweight-verification-and-soft-gates.md`
- `docs/architecture/wizard-io-and-localization.md`
- `tests/conformance/reference_file_inventory.py`
- `tests/conformance/reference_file_inventory.json`
- `tests/conformance/reference_file_inventory_test.sh`
- the three-language reference comparison pages;
- the three-language installation route updates;
- these three-language Work Item projections.

Out of scope:

- copying `scripts/**`, source Python, Make targets, `install_ai_cockpit.py`,
  locales, or the interactive Wizard;
- adding an interactive Installer Wizard or new Runtime commands;
- changing Rust Runtime semantics, release versions, Homebrew, or adopter
  acceptance;
- global Agent/MCP configuration, a second technology adopter, or immutable
  historical evidence.

## Pinned source and observed boundary

The source is `spirex-ds-dev/ai-cockpit-template` at
`e5acb677da6621004d96f0ef353c58fe8d3acfbf`. The ledger's Rust comparison
baseline remains `a533d49dfa848d95742833f8cd1b5f7e1bb897d5`; the Work Item
itself starts from the latest remote `main`.

The installed Runtime used for this Work Item is `ai-cockpit 0.2.33`, binary
SHA256 `sha256:eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4`.
Every Runtime command uses an explicit `--repo`.

The source detector and Wizard are repository-local Python presentation and
transaction adapters. The Rust target instead installs one immutable shared
Runtime and onboards a repository with explicit `inspect`, `attach`, profile
proposal/confirmation, and `doctor` operations. The source Wizard is therefore
reference-only in this target; no missing Runtime feature is hidden behind a
parity claim.

The file-level reading also covered the source evidence named by each page:
`scripts/ai_installer_detection.py`, `scripts/ai_install_wizard.py`,
`scripts/ai_install_plan.py`, `scripts/ai_installer_evidence.py`,
`scripts/ai_wizard_io.py`, `scripts/ai_wizard_localization.py`,
`scripts/install_ai_cockpit.py`, the calibration-wizard adapter, and the
corresponding installer, Wizard IO/localization, quality, and calibration test
modules. These source paths remain corpus-only; the target evidence is the
Rust code/tests and reader-facing routes listed below.

## File-level comparison decision

| Reference file | Result | Target evidence / boundary |
| --- | --- | --- |
| `installation-detection-boundary.md` | implemented-different-by-design | Read-only facts and explicit write boundaries are split across `inspect`, `status`, `doctor`, `attach`, `profile propose`, calibration docs, and their tests. Release installation is a separate immutable-artifact boundary. |
| `interactive-installation-wizard.md` | reference-only | The ten-stage dry-run/confirmation UI wraps the source Installer and is not shipped by the Rust Runtime. The target's explicit command route and provider-owned conversation UI prevent a prompt from becoming approval. |
| `lightweight-verification-and-soft-gates.md` | implemented-different-by-design | Typed stages, policy-driven tiers, hard/soft/informational decisions, skipped/unknown reasons, dynamic light/standard/strict routing, request-scoped context, and advisory cost/reuse are covered by Rust verification, CI, and cost docs/tests. |
| `wizard-io-and-localization.md` | implemented-different-by-design | CLI/MCP presentation localizes Runtime-generated text in en/zh-CN/ja and preserves Contract values verbatim. Wizard-specific TTY controls are not a Runtime feature; adapters own conversation controls. |

## Acceptance criteria

1. Read all four pinned files and record their concrete responsibilities,
   boundaries, and source test/module references.
2. Give each file an evidence-backed counterpart or explicit reference-only or
   external boundary. Do not call the absent interactive Wizard equivalent.
3. Make the three-language installation docs state the shared external Runtime,
   explicit `--repo`, attach/calibration route, and the intentional no-Wizard
   boundary.
4. Preserve the source soft-gate safety boundary in the Rust documentation:
   stage-aware fail-closed decisions, explicit skipped/unknown facts, dynamic
   light/standard/strict selection, and advisory cost telemetry. Record that
   source `hard`/`soft`/`informational` labels are not a copied target wire enum.
5. Move exactly four ledger records to the WI-305 batch, with non-empty reasons
   and counterparts, and leave no `migrate-gap` or deferred WI-305 record.
6. Run the inventory regression, documentation checks, governance gate, and
   `cargo test --locked --workspace` with the installed Runtime lifecycle.
7. Complete reviewed PR merge, post-merge finalization, exact branch/worktree
   cleanup, and a visible human Outcome. The object/adopter boundary must
   remain shared Runtime plus isolated repository state.

## Explicit non-claims

This Work Item does not claim source JSON/wire compatibility, general-language
translation, a Rust interactive installer, provider identity, hosted CI proof,
or production readiness. Localization changes only presentation chrome;
Contract intent, acceptance criteria, commands, paths, and machine evidence
remain authored values.
