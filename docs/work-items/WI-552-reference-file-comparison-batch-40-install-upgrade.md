---
author: AI Cockpit maintainers
title: "WI-552 — installation and upgrade reference comparison batch 40"
description: "Seventeen pinned installer and upgrade paths compared one by one, with Runtime capability discovery tightened."
audience: [maintainer, reviewer]
status: current
authority: canonical
workItemId: WI-552-reference-file-comparison-batch-40-install-upgrade
lastVerifiedBy: WI-552-reference-file-comparison-batch-40-install-upgrade
---

# WI-552 — installation and upgrade reference comparison batch 40

## Goal

Compare the pinned reference installer/upgrade paths one by one and preserve
their portable governance responsibilities in the shared Rust Runtime without
copying Python implementation, source JSON wire formats, provider registries,
or repository-local installer state.

## Scope and result

The batch covers the seventeen paths recorded in
`tests/conformance/reference_file_inventory.json`, including install facts,
planning/status/wizard, repository detection/evidence/ownership/transactions,
version parsing, upgrade application/conflict/proposal, and the Python launcher.
All paths are explicitly classified as `implemented-different-by-design` or
`reference-only`; no `migrate-gap` was introduced.

The Runtime now keeps one protocol-owned capability registry for
`.ai/agent-interface.json`. `attach` advertises the complete command surface
for discovery, while readiness, authorization, evidence, and lifecycle gates
remain request-scoped and repository-bound. Agents must inspect the manifest
and then query CLI/MCP schemas; a capability entry is not permission.

## Non-claims

Runtime installation is external and shared across repositories. `attach`
creates only minimum repository scaffolding. Source installer catalogs,
Python launchers, provider policy, global Agent/MCP configuration, and source
wire JSON are not inherited by attached object projects.

## Verification

- Rust attach regression checks the complete protocol capability registry and
  idempotent manifest bytes.
- Inventory and shell conformance checks cover all seventeen source paths and
  reject deferred/migrate-gap records in this batch.
- Tri-language capability/configuration/reference/parity docs explain
  capability discovery and exact `--help`/MCP `tools/list` lookup.
