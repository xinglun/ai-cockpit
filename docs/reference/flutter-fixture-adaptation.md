---
author: AI Cockpit maintainers
title: "Flutter fixture adaptation"
description: "A file-by-file Rust-native mapping of the pinned Flutter fixture without copying its installer or SDK implementation."
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

# Flutter fixture adaptation

This page compares the four files in the pinned reference fixture
`examples/fixtures/flutter-app/` one by one. It preserves useful semantics for
a Flutter adopter, but it is not a promise of Flutter SDK support and it does
not copy the reference installer, Make/Python orchestration, guard files, or
legacy JSON wire shape.

## File-by-file mapping

| Pinned source file | Source fact | Rust-native counterpart and boundary |
| --- | --- | --- |
| `fixture.json` | Declares a Flutter application, Flutter/Dart toolchain, Linux/macOS/Windows platforms, and safe/test paths. | Project Profile/Observer may record these as facts or candidate facts. `installerStack` describes the adopter, not shared Runtime installation; platform labels are not execution evidence. Paths become Contract scope and verification inputs only after human confirmation. |
| `lib/main.dart` | A small `greeting()` function returns the stable value `hello`. | Treat the path as adopter-owned source. A Work Item records intent, scope, and an owner-approved verification command; the Runtime does not execute or infer Dart semantics. |
| `pubspec.yaml` | Names the fixture and declares a Dart SDK range without declaring package dependencies. | This is package metadata that an Observer may report. SDK availability, dependency resolution, network, and lockfile state remain Unknown until provider evidence exists. The Runtime does not install Flutter or rewrite `pubspec.yaml`. |
| `test/widget_test.dart` | Uses `flutter_test` to assert the greeting. | This is an adopter/provider test capability. An owner may confirm `flutter test`; `verify --repo` records its result and identity. The file alone does not prove Flutter SDK, platform runner, plugin, or hosted CI readiness. |

## Installation is intentionally different

The fixture's `installerStack` and Dart metadata are not an installation recipe
for AI Cockpit. The target model is one immutable shared Runtime installed
outside each adopter, followed by an explicit repository attachment:

```bash
repo=/path/to/flutter-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

The attachment owns the repository's `.ai/`, Contract, evidence, knowledge,
and adapter state. It does not copy the Flutter fixture, install Flutter/Dart,
download packages, or bind a project to global Runtime state. Every later
command must carry the same explicit `--repo`; a different adopter receives a
separate repository identity and evidence chain.

For an adopter route, confirm the exact Flutter command with the owner and
provider before verification. A local `flutter test` result is not provider,
release, or enterprise evidence by itself.

## What is and is not inherited

An attached Flutter project inherits the shared Runtime's Contract validation,
fail-closed unknown handling, evidence identity, lifecycle, and human Outcome
rules. It does not inherit the fixture's SDK, package cache, platform runners,
installer variables, Dart source, or a claim that Flutter checks have run. The
project keeps its own scope, profile, snapshot, and evidence under its own
repository context.

This is semantic/documentation parity, not source command, build-tool, or
JSON-wire compatibility. A real Flutter adopter acceptance remains a separate
post-release test using an immutable public Runtime artifact.

[Reference index](README.md) | [中文](flutter-fixture-adaptation.zh-CN.md) | [日本語](flutter-fixture-adaptation.ja.md)
