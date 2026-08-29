---
workItemId: WI-394-reference-ios-swift-fixture
title: "Reference iOS Swift Package fixture adaptation"
author: AI Cockpit maintainers
description: "File-by-file semantic mapping of the pinned iOS Swift Package fixture with an explicit shared Runtime installation boundary."
type: documentation
audience:
  - adopter
  - contributor
  - maintainer
authority: canonical
status: in_progress
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
lastVerifiedBy: WI-394-reference-ios-swift-fixture
---

# WI-394 — Reference iOS Swift Package fixture adaptation

## Intent

Compare the four pinned iOS Swift Package fixture files one by one and record
the Rust-native/adopter mapping without hard-copying Swift/Xcode installation
or build implementation.

## Scope

- `examples/fixtures/ios-swift-package/Package.swift`
- `examples/fixtures/ios-swift-package/Sources/AppCore/AppCore.swift`
- `examples/fixtures/ios-swift-package/Tests/AppCoreTests/AppCoreTests.swift`
- `examples/fixtures/ios-swift-package/fixture.json`
- the tri-language iOS Swift Package adaptation, reference comparison, parity,
  index, and Work Item records

## Acceptance

1. Each source file has an individual semantic mapping or explicit bounded
   non-applicability.
2. The guide explains that Swift/Xcode checks are adopter/provider-owned and
   that missing SDK, simulator, signing, network, and CI facts remain Unknown.
3. Installation is documented as one shared immutable Runtime outside the
   adopter plus explicit `attach --repo`; source installer/build/wire artifacts
   are not copied.
4. Inventory, parity, links, and all three language records bind `e5acb677`.
5. The installed Runtime verifies the documentation and conformance checks.

## Evidence boundary

This Work Item proves semantic/documentation parity only. It does not prove
Apple/Swift toolchain support or a post-release iOS adopter acceptance.
