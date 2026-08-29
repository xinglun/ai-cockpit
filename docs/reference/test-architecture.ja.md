---
author: AI Cockpit maintainers
title: Test architecture
description: Rust project の layered、negative-first verification と quality gate の責任境界。
audience:
  - contributor
  - maintainer
  - reviewer
status: current
authority: translation
canonical: docs/reference/test-architecture.md
lastVerifiedBy: WI-378-reference-documentation-batch-17
capabilityClaims:
  - layered_verification
---

# Test architecture

[English](test-architecture.md) · [简体中文](test-architecture.zh-CN.md) · [日本語](test-architecture.ja.md)

Verification は layered かつ negative-first です。repository に明示的な evidence がある場合だけ layer を verified とし、利用できない layer は `not_applicable` または `unknown` として記録します。暗黙に green にはしません。

| Layer | Rust evidence boundary |
| --- | --- |
| Protocol/schema/state machine | `cargo test --workspace`、typed protocol test、lifecycle/property regression |
| Repository transaction / lifecycle | attach、Contract、checkpoint、verify、finish、archive、close、recovery、isolation の repository/CLI integration test |
| Verification executor | bounded argv 実行、worker 制限、reuse identity、failure retention、scope test |
| Security/adversarial | conformance と absurd-case fixture、path/symlink/identity tamper、prompt-injection と weakening regression |
| Hosted platform | GitHub Actions Windows/runtime と V1 semantic oracle。provider state は external evidence のままです。 |
| Release/adopter | immutable public archive、checksum/SBOM/provenance、fresh adopter と N-1 upgrade harness |
| Documentation/governance | tri-language metadata、parity、inventory、status-promotion、governance-integrity check |

Dynamic quality route は変更 surface、Contract policy、stage から `light`、`standard`、`strict` を選びます。これは Verification strength であり Evidence Assurance ではありません。低コスト route は無関係な layer を省けますが、mandatory floor を下げたり unknown を pass に変えたりしません。

Local check が証明するのは repository fact だけです。provider approval、enterprise identity、全 external consumer compatibility、universal test coverage は証明しません。`target/` の report と generated receipt は evidence output であり、手編集してはいけません。
