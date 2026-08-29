---
author: AI Cockpit maintainers
title: "iOS Swift Package fixture adaptation"
description: "A file-by-file Rust-native mapping of the pinned iOS Swift Package fixture without copying its installer or Xcode implementation."
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

# iOS Swift Package fixture adaptation

This page compares the four files in the pinned reference fixture
`examples/fixtures/ios-swift-package/` one by one. It preserves useful
Swift-package semantics for an adopter, but it is not a promise of Apple
platform or Xcode support and it does not copy the reference installer,
Make/Python orchestration, guard files, or legacy JSON wire shape.

## File-by-file mapping

| Pinned source file | Source fact | Rust-native counterpart and boundary |
| --- | --- | --- |
| `Package.swift` | Uses Swift tools 5.9, declares an `AppCore` library product, and connects an `AppCoreTests` test target to `AppCore`. | Treat package topology as adopter/provider-owned build metadata. A Work Item records the relevant paths and owner-approved command; the Runtime does not install SwiftPM/Xcode or infer Apple SDK readiness. |
| `Sources/AppCore/AppCore.swift` | A public `greeting()` function returns the stable value `hello`. | The path is adopter-owned source scope. Runtime Contract validation and evidence binding are inherited, while Swift execution remains provider-owned. |
| `Tests/AppCoreTests/AppCoreTests.swift` | An XCTest case imports `AppCore` and asserts the greeting. | This is an adopter/provider test capability. An owner may confirm `swift test` or an Xcode scheme; `verify --repo` records the selected command and result. The file alone does not prove macOS/iOS SDK, simulator, signing, or hosted CI readiness. |
| `fixture.json` | Declares an iOS Swift package, Swift Package stack, Swift installer metadata, macOS platform, and safe/test paths. | Project Profile/Observer may record these as facts or candidate facts. `installerStack` describes the adopter, not shared Runtime installation; `macos` is a platform label, not execution evidence. |

## Installation is intentionally different

The fixture's Swift metadata is not an installation recipe for AI Cockpit. The
target model is one immutable shared Runtime installed outside each adopter,
followed by an explicit repository attachment:

```bash
repo=/path/to/swift-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

The attachment owns the repository's `.ai/`, Contract, evidence, knowledge,
and adapter state. It does not copy the Swift fixture, install SwiftPM/Xcode,
select an Apple SDK, or bind a project to global Runtime state. Every later
command must carry the same explicit `--repo`; a different adopter receives a
separate repository identity and evidence chain.

For an adopter route, confirm the exact `swift test` or Xcode command with the
owner and provider before verification. A local result is not provider,
release, or enterprise evidence by itself.

## What is and is not inherited

An attached Swift project inherits the shared Runtime's Contract validation,
fail-closed unknown handling, evidence identity, lifecycle, and human Outcome
rules. It does not inherit the fixture's Swift toolchain, Xcode project state,
Apple SDK, simulator, signing credentials, installer variables, or a claim
that tests have run. The project keeps its own scope, profile, snapshot, and
evidence under its own repository context.

This is semantic/documentation parity, not source command, build-tool, or
JSON-wire compatibility. A real iOS/Swift adopter acceptance remains a
separate post-release test using an immutable public Runtime artifact.

[Reference index](README.md) | [中文](ios-swift-fixture-adaptation.zh-CN.md) | [日本語](ios-swift-fixture-adaptation.ja.md)
