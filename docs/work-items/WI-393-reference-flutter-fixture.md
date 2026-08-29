---
workItemId: WI-393-reference-flutter-fixture
title: "Reference Flutter fixture adaptation"
author: AI Cockpit maintainers
description: "File-by-file semantic mapping of the pinned Flutter fixture with an explicit shared Runtime installation boundary."
type: documentation
audience:
  - adopter
  - contributor
  - maintainer
authority: canonical
status: in_progress
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
lastVerifiedBy: WI-393-reference-flutter-fixture
---

# WI-393 — Reference Flutter fixture adaptation

## Intent

Compare the four pinned Flutter fixture files one by one and record the
Rust-native/adopter mapping without hard-copying Flutter installation or SDK
implementation.

## Scope

- `examples/fixtures/flutter-app/fixture.json`
- `examples/fixtures/flutter-app/lib/main.dart`
- `examples/fixtures/flutter-app/pubspec.yaml`
- `examples/fixtures/flutter-app/test/widget_test.dart`
- the tri-language Flutter adaptation, reference comparison, parity, index, and
  Work Item records

## Acceptance

1. Each source file has an individual semantic mapping or explicit bounded
   non-applicability.
2. The guide explains that Flutter/Dart checks are adopter/provider-owned and
   that missing SDK, dependency, network, platform, plugin, and CI facts remain
   Unknown.
3. Installation is documented as one shared immutable Runtime outside the
   adopter plus explicit `attach --repo`; source installer/build/wire artifacts
   are not copied.
4. Inventory, parity, links, and all three language records bind `e5acb677`.
5. The installed Runtime verifies the documentation and conformance checks.

## Evidence boundary

This Work Item proves semantic/documentation parity only. It does not prove
Flutter toolchain support or a post-release Flutter adopter acceptance.
