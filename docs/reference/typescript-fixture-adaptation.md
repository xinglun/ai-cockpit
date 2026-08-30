---
author: AI Cockpit maintainers
title: "TypeScript web fixture adaptation"
description: "A file-by-file Rust-native mapping of the pinned TypeScript web fixture without copying its application, npm toolchain, or lifecycle script."
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-432-reference-typescript-fixture-boundary
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
capabilityClaims:
  - semantic_reference_mapping
---

# TypeScript web fixture adaptation

This page compares the eleven files in the pinned reference fixture
`examples/fixtures/typescript-web/` one by one. It preserves useful semantics
for a TypeScript/web adopter without copying the fixture application, npm
dependencies, Node commands, or source lifecycle implementation into the Rust
Runtime.

[English](typescript-fixture-adaptation.md) · [简体中文](typescript-fixture-adaptation.zh-CN.md) · [日本語](typescript-fixture-adaptation.ja.md)

## File-by-file mapping

| Pinned source file | Source fact | Rust-native counterpart and boundary |
| --- | --- | --- |
| `.gitignore` | Ignores `node_modules/`, `dist/`, and generated `.fixture-state.json`. | Build-output hygiene remains the adopter's responsibility. The release harness uses its own isolated roots and does not generate a project ignore file or copy this one. |
| `evidence.json` | Describes local npm lifecycle evidence and explicitly marks provider evidence as unavailable. | Runtime verification binds repository, snapshot, Runtime, command, and result identity. Source-local evidence is not promoted to provider, hosted-CI, sandbox, immutable-audit, or enterprise evidence. |
| `fixture.json` | Declares the TypeScript web stack, Node/npm/TypeScript toolchain, platforms, safe path, and test path. | Project Observer/Profile may record confirmed adopter facts. The Runtime never infers capability, platform readiness, or Contract scope from this fixture metadata. |
| `package-lock.json` | Pins the fixture's TypeScript 5.8.3 npm dependency and registry integrity. | Dependency manifests and registries belong to the adopter. The shared Runtime does not install Node packages, carry this lockfile, or treat its integrity as Runtime supply-chain evidence. |
| `package.json` | Defines build, test, lint, format-check, and lifecycle npm scripts. | An adopter declares explicit verification argv in its Contract; Runtime records each result and keeps governance lifecycle (`preflight` through `close`) separate from npm script orchestration. |
| `scripts/format-check.mjs` | Checks a trailing newline and rejects tabs in `src/index.ts`. | This is a fixture-specific format rule. An adopter may declare its own formatter command; a local result is bound as local evidence only. |
| `scripts/lifecycle.mjs` | Runs install/configure/normal phases, blocks ambiguous and critical-domain requests, exercises upgrade/rollback, and performs release checks. | The installed Runtime supplies the repository-bound lifecycle, human review pause, evidence binding, recovery, and visible Outcome. The source Node script is not executed as Runtime authority or copied. |
| `scripts/lint.mjs` | Checks for `evaluateRequest` and rejects `any` in the sample source. | This is application-specific lint logic, not a portable governance control. The adopter owns its lint command and acceptance evidence. |
| `src/index.ts` | Sample request evaluator returns `allow` or `block` with a reason and resume condition. | Application behavior remains adopter-owned. Runtime decisions and stop states are typed governance records; it does not import or infer this sample policy. |
| `test/index.test.mjs` | Node tests assert normal allowance and dangerous-request blocking. | The adopter supplies and runs its own test command. A source fixture assertion is never promoted to Runtime, provider, or enterprise evidence. |
| `tsconfig.json` | Enables strict TypeScript compilation with NodeNext modules and declaration output. | TypeScript compiler configuration is an adopter responsibility. The Runtime accepts explicit command results but does not promise a Node/TypeScript toolchain or copy compiler settings. |

## Installation and adopter boundary

The fixture's stack metadata is not an AI Cockpit installation recipe. Install
one shared Runtime outside the adopter, then attach the repository explicitly:

```bash
repo=/path/to/typescript-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

The adopter owns Node.js, npm, TypeScript, dependency locks, build outputs,
and hosted/provider evidence. Every later Runtime command carries the same
explicit `--repo`; Contract scope, profile, snapshot, evidence, knowledge,
and Agent adapter records remain repository-local.

## What an adopter inherits

An attached TypeScript/web project inherits the shared Runtime's Contract
validation, fail-closed unknown handling, identity-bound evidence, lifecycle,
and visible human Outcome rules. It does not inherit this fixture's Node
dependencies, npm scripts, application code, tests, or a claim that any
command has run. Local npm results are not provider, hosted-CI, release, or
enterprise evidence unless the corresponding external authority supplies it.

This is semantic/documentation parity, not TypeScript toolchain support, source
command compatibility, or JSON-wire compatibility. A second-technology
adopter acceptance remains a separately authorized post-release Work Item.

[Reference index](README.md) · [Reference file comparison](reference-file-comparison.md)
