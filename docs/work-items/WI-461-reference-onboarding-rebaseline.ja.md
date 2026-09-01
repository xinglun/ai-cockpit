---
author: AI Cockpit maintainers
title: "WI-461 — getting-started onboarding rebaseline"
workItemId: WI-461-reference-onboarding-rebaseline
description: "変更された local reference の onboarding 9 file を再確認し、semantic inventory の判断を確定します。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-461-reference-onboarding-rebaseline
terminalArchive: .ai/work-items/archive/WI-461-reference-onboarding-rebaseline.contract.json
terminalVerification: .ai/evidence/WI-461-reference-onboarding-rebaseline.verification.json
terminalFinalization: .ai/decisions/WI-461-reference-onboarding-rebaseline.finalize.json
terminalDecision: .ai/decisions/WI-461-reference-onboarding-rebaseline.close.json
---

# WI-461 — getting-started onboarding rebaseline

この Work Item は、maintainer 管理の local reference
`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template` で、履歴比較 commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` から pinned commit
`fde3380f81fea5fd2e288f7a8849f737dc074060` までに変更された onboarding 9 page を再確認します。
public reference には接続せず、source implementation もコピーしません。

[English](WI-461-reference-onboarding-rebaseline.md) · [简体中文](WI-461-reference-onboarding-rebaseline.zh-CN.md)

## File-level decisions

| Pinned reference path | Classification | Rust-native counterpart と boundary |
| --- | --- | --- |
| `docs/getting-started/first-work-item.md` | `implemented-different-by-design` | Rust page は repository-bound の start → preflight → checkpoint → verify → finish → archive → reviewed merge → cleanup → close、visible human Outcome、human-review stop を保持します。source-only Make command と削除された `REPORT_LANGUAGE` argument は copy しません。 |
| `docs/getting-started/first-work-item.zh-CN.md` | `implemented-different-by-design` | Chinese page は同じ lifecycle と stop condition を明示的な `--repo` で保持し、presentation language は Contract fact を変えません。 |
| `docs/getting-started/first-work-item.ja.md` | `implemented-different-by-design` | Japanese page は同じ lifecycle、provider-resource boundary、exact cleanup path を保持し、この batch で重複した merge paragraph を修正しました。 |
| `docs/getting-started/security-release-verification.md` | `implemented-different-by-design` | Rust の release/distribution と installation-security page は current manifest/SHA256SUMS route で tag、digest、SBOM、provenance、provider responsibility、adopter isolation を保持します。source `release.json` projection は copy しません。 |
| `docs/getting-started/security-release-verification.zh-CN.md` | `implemented-different-by-design` | Chinese release route は evidence separation と mismatch fail-closed rule を Rust-native release asset と external-provider boundary で保持します。 |
| `docs/getting-started/security-release-verification.ja.md` | `implemented-different-by-design` | Japanese release route は digest、provenance、SBOM、public-adopter limit を保持し、source installer behavior は取り込みません。 |
| `docs/getting-started/standard-adoption-guide.md` | `implemented-different-by-design` | Rust guide は reader-first の install、attach、calibration、adapter、Work Item、Outcome、merge、cleanup、close stage を shared Runtime で保持します。source Make workflow bytes は target Contract ではありません。 |
| `docs/getting-started/standard-adoption-guide.zh-CN.md` | `implemented-different-by-design` | Chinese guide は順序付けられた adoption boundary と明示的な repository ownership を Rust CLI route で保持します。 |
| `docs/getting-started/standard-adoption-guide.ja.md` | `implemented-different-by-design` | Japanese guide は同じ ordered adoption route と shared Runtime boundary を保持し、source-specific command は copy しません。 |

これは semantic/documentation parity であり、source file や JSON-wire parity ではありません。
target は shared installed Runtime 一つと明示的な `--repo` を使い、adopter project は同じ
governance boundary を継承しながら repository state と provider configuration を分離します。

## Verification boundary

Inventory は `sourceChangedSincePrevious`、`previousBatch`、`previousClassification` を
immutable な比較 provenance として保持し、9 record を `deferred-next-batch` から evidence-backed
result に昇格します。offline source policy、inventory check、tri-language getting-started semantic
check、documentation acceptance、governance-integrity check、declared locked Rust verification を
通過しなければなりません。この documentation-only batch は Runtime behavior や object/adopter
repository を変更しません。
