---
author: AI Cockpit maintainers
title: "WI-414 — Python fixture boundary"
workItemId: WI-414-reference-python-fixture-boundary
description: "Compare the pinned Python fixture files one by one and record an explicit reference-only boundary without copying source fixtures."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-414-reference-python-fixture-boundary
terminalArchive: .ai/work-items/archive/WI-414-reference-python-fixture-boundary.contract.json
terminalVerification: .ai/evidence/WI-414-reference-python-fixture-boundary.verification.json
terminalFinalization: .ai/decisions/WI-414-reference-python-fixture-boundary.finalize.json
terminalDecision: .ai/decisions/WI-414-reference-python-fixture-boundary.close.json
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
---

# WI-414 — Python fixture boundary

## Intent and boundary

Read each of the four files under `examples/fixtures/python/` at reference
commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`. These files form an
executable Python/pytest sample in the reference repository. They are not Rust
Runtime code, Python toolchain support, portable governance policy, or
enterprise evidence.

| Pinned reference path | Classification | Bounded target decision |
| --- | --- | --- |
| `fixture.json` | `reference-only` | Sample stack, platform, and path metadata; target facts remain repository-local and are not inferred from this file. |
| `pyproject.toml` | `reference-only` | Sample packaging and pytest configuration; Python installation and test commands remain adopter/provider responsibilities. |
| `src/service.py` | `reference-only` | Application sample returning `ok`; it is not governance logic and is not copied. |
| `tests/test_service.py` | `reference-only` | Fixture-only pytest assertion; it is not Runtime or enterprise evidence, and an adopter must declare its own verification command. |

No Python source, dependency manifest, installer, or test runner is copied into
the Rust repository. The shared installed Runtime still supplies the same
Contract, evidence, lifecycle, and human Outcome controls to a Python adopter,
but this is semantic/documentation parity rather than Python toolchain or
source-command compatibility. A second-stack adopter acceptance is a separate
authorized activity and is not claimed here.

## Acceptance

- All four pinned paths are read and appear exactly once in the machine ledger.
- Each path has a non-empty reason and target boundary, and all four are
  `reference-only`; no `deferred-next-batch` or `migrate-gap` remains in this
  batch.
- English, Simplified Chinese, and Japanese comparison/parity routes agree on
  the source pin, file list, and non-copy boundary.
- Inventory regression and documentation gates pass without changing Runtime
  governance semantics, Python tooling, or global Agent/MCP configuration.

## Verification and non-claims

This is semantic/reference-boundary parity, not Python toolchain support, source
command compatibility, JSON-wire compatibility, or a second-stack adopter
acceptance. The machine ledger remains the source of per-file truth.

[简体中文](WI-414-reference-python-fixture-boundary.zh-CN.md) · [日本語](WI-414-reference-python-fixture-boundary.ja.md)
