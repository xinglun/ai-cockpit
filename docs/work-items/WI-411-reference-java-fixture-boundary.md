---
author: AI Cockpit maintainers
title: "WI-411 — Java multi-module fixture boundary"
workItemId: WI-411-reference-java-fixture-boundary
description: "Compare the pinned Java fixture files one by one and record an explicit reference-only boundary without copying source fixtures."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-411-reference-java-fixture-boundary
---

# WI-411 — Java multi-module fixture boundary

## Intent and boundary

Read each of the nine files under `examples/fixtures/java-multimodule/` at
reference commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`. These files form an
executable Java/Maven sample in the reference repository. They are not Rust
Runtime code, portable governance policy, or enterprise evidence.

| Pinned reference path | Classification | Bounded target decision |
| --- | --- | --- |
| `.gitignore` | `reference-only` | Fixture-local build hygiene; target release harness owns isolated temporary roots. |
| `app/src/main/java/fixture/app/Main.java` | `reference-only` | Java application sample; generic argv execution does not claim Java-specific Runtime support. |
| `app/src/test/java/fixture/app/MainTest.java` | `reference-only` | Fixture assertion; adopter verification records declared commands rather than copying this test. |
| `core/src/main/java/fixture/core/Decision.java` | `reference-only` | Domain sample policy; target repository policy remains explicit and typed. |
| `core/src/test/java/fixture/core/DecisionTest.java` | `reference-only` | Sample-only test, not Runtime or enterprise evidence. |
| `evidence.json` | `reference-only` | Source-local evidence, including unavailable capabilities; not promoted to target release evidence. |
| `fixture.json` | `reference-only` | Source stack/module metadata; target does not infer adopter capability from it. |
| `pom.xml` | `reference-only` | Maven build input; Java/Maven execution remains adopter or delegated-provider responsibility. |
| `scripts/lifecycle.sh` | `reference-only` | Source fixture orchestration; target lifecycle is provided by the installed Rust Runtime. |

The target therefore adds no Java source, Maven manifest, or source shell
orchestrator. A second-technology adopter acceptance remains a separate,
explicitly authorized Work Item; this batch does not claim that capability.

## Acceptance

- All nine pinned paths are read and appear exactly once in the machine ledger.
- Each path has a non-empty reason and target boundary, and all nine are
  `reference-only`; no `deferred-next-batch` or `migrate-gap` remains in this
  batch.
- The English, Simplified Chinese, and Japanese comparison/parity routes agree
  on the source pin, nine paths, and non-copy boundary.
- The inventory regression and documentation gates pass without changing
  Runtime governance semantics or global Agent/MCP configuration.

## Verification and non-claims

This is semantic/reference-boundary parity, not Java toolchain support, source
command compatibility, JSON-wire compatibility, or a second-stack adopter
acceptance. The full machine ledger remains the source of per-file truth.

[简体中文](WI-411-reference-java-fixture-boundary.zh-CN.md) · [日本語](WI-411-reference-java-fixture-boundary.ja.md)
