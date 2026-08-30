---
author: AI Cockpit maintainers
title: "Mixed-monorepo fixture adaptation"
description: "A file-by-file Rust-native boundary for the pinned mixed Python/Node fixture without copying its application or toolchains."
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-421-reference-mixed-monorepo
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
capabilityClaims:
  - semantic_reference_mapping
---

# Mixed-monorepo fixture adaptation

This page compares the five files in the pinned reference fixture
`examples/fixtures/mixed-monorepo/` one by one. The fixture is an executable
application sample, not Rust Runtime code or portable enterprise evidence.
The target records its useful governance meaning without copying its Python
or Node toolchains.

[English](mixed-monorepo-fixture-adaptation.md) · [简体中文](mixed-monorepo-fixture-adaptation.zh-CN.md) · [日本語](mixed-monorepo-fixture-adaptation.ja.md)

## File-by-file mapping

| Pinned source file | Source fact | Rust-native counterpart and boundary |
| --- | --- | --- |
| `fixture.json` | Declares a mixed Python/Node sample, generic installer metadata, three platforms, and safe/test paths. | Project Observer/Profile may record facts actually observed in an adopter. The Runtime does not infer toolchain capability or safe scope from fixture metadata. |
| `package.json` | Private Node package metadata with no dependencies or scripts. | Fixture application input only. Node installation, dependencies, scripts, and execution remain adopter/provider responsibilities. |
| `pyproject.toml` | Minimal Python project metadata. | Not a portable Contract or Runtime dependency. Python installation, dependencies, and test commands require explicit adopter evidence. |
| `services/api/app.py` | A health function returns `ok`. | Application code, not governance logic. Runtime verification can bind an adopter-declared argv result but does not ship or infer Python behavior. |
| `services/api/tests/test_app.py` | A pytest assertion checks the health function. | Fixture evidence only. An adopter must declare and run its own verification command; source tests are never promoted as target evidence. |

## Installation and adopter boundary

The fixture does not define an AI Cockpit installation recipe. Install one
shared Runtime outside the adopter and attach the repository explicitly:

```bash
repo=/path/to/mixed-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

The adopter owns its Python and Node interpreters, dependency locks, test
commands, and hosted-provider evidence. Every later Runtime command carries
the same explicit `--repo`; Contract, snapshot, evidence, knowledge, and
Agent adapter records remain repository-local.

## What an adopter inherits

An attached mixed repository inherits the shared Runtime's Contract
validation, fail-closed unknown handling, identity-bound evidence, lifecycle,
repository isolation, and visible human Outcome rules. It does not inherit the
fixture's package metadata, source, test runner, installer behavior, or a
claim that either toolchain is available. This is semantic/documentation
parity, not mixed-stack toolchain support, source-command compatibility, or a
second-technology adopter acceptance.

[Reference index](README.md) · [Reference file comparison](reference-file-comparison.md)
