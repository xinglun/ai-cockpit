---
workItemId: WI-392-reference-android-fixture
title: "Reference Android fixture adaptation"
author: AI Cockpit maintainers
description: "File-by-file semantic mapping of the pinned Android fixture with an explicit shared Runtime installation boundary."
type: documentation
audience:
  - adopter
  - contributor
  - maintainer
authority: canonical
status: implemented
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
lastVerifiedBy: WI-392-reference-android-fixture
terminalArchive: .ai/work-items/archive/WI-392-reference-android-fixture.contract.json
terminalVerification: .ai/evidence/WI-392-reference-android-fixture.verification.json
terminalFinalization: .ai/decisions/WI-392-reference-android-fixture.finalize.53b26b80706cab70f1fb4c8c3772cbf92475c25fa11d5141c906ccafa9566fea.json
terminalDecision: .ai/decisions/WI-392-reference-android-fixture.close.json
---

# WI-392 — Reference Android fixture adaptation

## Intent

Compare the four pinned Android fixture files one by one and record the
Rust-native/adopter mapping without hard-copying Android installation or build
implementation.

## Scope

- `examples/fixtures/android-app/app/src/main/kotlin/example/MainActivity.kt`
- `examples/fixtures/android-app/app/src/test/kotlin/example/MainActivityTest.kt`
- `examples/fixtures/android-app/fixture.json`
- `examples/fixtures/android-app/settings.gradle.kts`
- the tri-language Android adaptation, reference comparison, parity, index, and
  Work Item records

## Acceptance

1. Each source file has an individual semantic mapping or explicit bounded
   non-applicability.
2. The guide explains that Android/Gradle checks are adopter/provider-owned and
   that missing SDK, device, signing, network, and CI facts remain Unknown.
3. Installation is documented as one shared immutable Runtime outside the
   adopter plus explicit `attach --repo`; source installer/build/wire artifacts
   are not copied.
4. Inventory, parity, links, and all three language records bind `e5acb677`.
5. The installed Runtime verifies the documentation and conformance checks.

## Evidence boundary

This Work Item proves semantic/documentation parity only. It does not prove
Android toolchain support or a post-release Android adopter acceptance.
