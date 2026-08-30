---
author: AI Cockpit maintainers
title: "WI-421 — mixed-monorepo fixture boundary"
description: "Compare the pinned mixed Python/Node fixture one file at a time without copying application or toolchain assets."
workItemId: WI-421-reference-mixed-monorepo
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-421-reference-mixed-monorepo
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
---

# WI-421 — mixed-monorepo fixture boundary

[简体中文](WI-421-reference-mixed-monorepo.zh-CN.md) · [日本語](WI-421-reference-mixed-monorepo.ja.md)

## Intent and boundary

Read each pinned file under `examples/fixtures/mixed-monorepo/` and record
whether its responsibility is portable to an attached Rust repository. Keep
the mixed Python/Node application, package metadata, source commands, and
installer behavior out of the Runtime. Preserve only the explicit facts,
scope, provider-owned execution, and evidence-binding boundaries an adopter
can inherit.

## Scope

The five files are `fixture.json`, `package.json`, `pyproject.toml`,
`services/api/app.py`, and `services/api/tests/test_app.py`. The inventory,
three-language comparison/parity routes, reference index, and adaptation page
are updated together.

## Acceptance

- All five pinned paths are read and classified exactly once as `reference-only`
  with a non-empty reason and Rust/adopter counterpart.
- No fixture source, Python/Node dependency, installer, provider-global
  configuration, or source JSON wire shape is copied.
- English, Simplified Chinese, and Japanese routes agree on the source pin,
  file list, inheritance boundary, and non-claims.
- Inventory, documentation, and object/adopter inheritance checks pass.

## Verification boundary

This is semantic/documentation parity, not Python/Node toolchain support,
source-command compatibility, or a second-technology adopter acceptance.
Verification uses the installed shared Runtime with an explicit `--repo`; the
adopter's own interpreters, dependencies, commands, and provider evidence stay
outside this Work Item.
