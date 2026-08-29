---
author: AI Cockpit maintainers
title: "Android fixture adaptation"
description: "A file-by-file Rust-native mapping of the pinned Android fixture without copying its installer or build implementation."
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
capabilityClaims:
  - semantic_reference_mapping
lastVerifiedBy: documentation-acceptance
---

# Android fixture adaptation

This page compares the four files in the pinned reference fixture
`examples/fixtures/android-app/` one by one. It preserves useful semantics for
an Android adopter, but it is not a promise of Android tooling support and it
does not copy the reference installer, Make/Python orchestration, guard files,
or legacy JSON wire shape.

## File-by-file mapping

| Pinned source file | Source fact | Rust-native counterpart and boundary |
| --- | --- | --- |
| `app/src/main/kotlin/example/MainActivity.kt` | A small Kotlin `greeting()` function returns a stable value. | Treat the path as repository-owned source. Put it in a Work Item `scope`/`outOfScope` decision and verify the owner-approved command; the Runtime does not execute or infer Kotlin semantics. |
| `app/src/test/kotlin/example/MainActivityTest.kt` | `kotlin.test` asserts the greeting. | This is an adopter/provider test capability. A project owner may confirm a command such as `./gradlew test`; `verify --repo` records its result and identity. The file alone does not prove an installed SDK, emulator, signing setup, or hosted CI. |
| `fixture.json` | Declares `projectType`, `stack`, `installerStack`, toolchain, platforms, and safe/test paths. | Project Profile/Observer may record these as facts or candidate facts. `installerStack` describes the adopter, not the shared Runtime installation; platform names are not execution evidence. Safe/test paths become explicit Contract scope and verification inputs only after human confirmation. |
| `settings.gradle.kts` | Configures Gradle repositories, root name, and `:app` inclusion. | It is build-topology evidence only. Gradle/Android dependency download, SDK/device readiness, credentials, network, and CI remain Unknown until an owner-approved provider check supplies evidence. |

## Installation is intentionally different

The reference fixture is an example project and its stack metadata is not an
installation recipe for AI Cockpit. The target model is one immutable shared
Runtime installed outside each adopter, followed by an explicit repository
attachment:

```bash
repo=/path/to/android-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

The attachment owns the repository's `.ai/`, Contract, evidence, knowledge,
and adapter state. It does not copy the Android fixture, install Gradle or the
Android SDK, or bind a project to a global Runtime state. Every later command
must carry the same explicit `--repo`; a different adopter receives a separate
repository identity and evidence chain.

For an adopter route, use the [Android profile start](../getting-started/examples/android.md)
guide: propose a read-only candidate, obtain owner confirmation for the exact
Gradle command, then verify it. A local result is not provider, release, or
enterprise evidence.

## What is and is not inherited

An attached Android project inherits the shared Runtime's Contract validation,
fail-closed unknown handling, evidence identity, lifecycle, and human Outcome
rules. It does not inherit the reference fixture's installer variables,
Gradle files, Kotlin source, or a claim that Android checks have run. The
project keeps its own scope, profile, snapshot, and evidence under its own
repository context.

This is semantic/documentation parity, not source command, build-tool, or
JSON-wire compatibility. A real Android adopter acceptance remains a separate
post-release test using an immutable public Runtime artifact.

[Reference index](README.md) | [中文](android-fixture-adaptation.zh-CN.md) | [日本語](android-fixture-adaptation.ja.md)
