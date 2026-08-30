---
author: AI Cockpit maintainers
title: "Python fixture adaptation"
description: "A file-by-file Rust-native mapping of the pinned Python fixture without copying its application, packaging, or test implementation."
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-414-reference-python-fixture-boundary
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
capabilityClaims:
  - semantic_reference_mapping
---

# Python fixture adaptation

This page compares the four files in the pinned reference fixture
`examples/fixtures/python/` one by one. It records useful Python-adopter
semantics without copying the fixture, its packaging metadata, or its test
runner into the Rust Runtime.

[English](python-fixture-adaptation.md) · [简体中文](python-fixture-adaptation.zh-CN.md) · [日本語](python-fixture-adaptation.ja.md)

## File-by-file mapping

| Pinned source file | Source fact | Rust-native counterpart and boundary |
| --- | --- | --- |
| `fixture.json` | Declares a Python service, `python3` toolchain, Linux/macOS platforms, and safe/test paths. | Project Observer/Profile may record these as repository-local facts or candidate facts. The shared Runtime does not infer Python capability, platform readiness, or safe scope from this file; an owner confirms the exact Contract. |
| `pyproject.toml` | Declares package metadata (`requires-python >=3.11`) and pytest's `tests` path. | Python packaging and pytest remain adopter/provider responsibilities. The owner supplies an explicit command such as `python -m pytest`; Runtime verification records its argv and result but does not install Python or copy this manifest. |
| `src/service.py` | A minimal application function returns the health value `ok`. | This is fixture application code, not governance logic. Rust verification can execute an adopter-declared command and bind its evidence, but the target does not ship or infer Python semantics from this source. |
| `tests/test_service.py` | A pytest test asserts the health function result. | This is a sample assertion, not a portable Runtime test contract or enterprise evidence. An adopter must declare and run its own test command; source fixture tests are never promoted as target evidence. |

## Installation and adopter boundary

The reference fixture's stack metadata is not an AI Cockpit installation
recipe. Install one shared Runtime outside the adopter, then attach the
repository explicitly:

```bash
repo=/path/to/python-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

The adopter owns its Python interpreter, virtual environment, dependency
lock, pytest configuration, and CI/provider evidence. Every later Runtime
command carries the same explicit `--repo`; its Contract scope, profile,
snapshot, evidence, knowledge, and Agent adapter remain repository-local.

## What an adopter inherits

An attached Python project inherits the shared Runtime's Contract validation,
fail-closed unknown handling, identity-bound evidence, lifecycle, and visible
human Outcome rules. It does not inherit the reference fixture's
`pyproject.toml`, Python source, pytest installation, or a claim that tests
have run. A local test result is not provider, hosted-CI, release, or
enterprise evidence unless the corresponding external authority supplies it.

This is semantic/documentation parity, not Python toolchain support, source
command compatibility, or JSON-wire compatibility. A Python adopter
acceptance remains a separately authorized post-release test.

[Reference index](README.md) · [Reference file comparison](reference-file-comparison.md)
