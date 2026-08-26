---
author: AI Cockpit maintainers
title: "WI-300 — v0.2.33 release と install acceptance"
workItemId: WI-300-release-v0-2-33
description: "修正済み Runtime を公開し、immutable artifact を検証して repository と adopter を受入れます。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-300-release-v0-2-33
authority: canonical
---

# WI-300 — v0.2.33 release preparation

## Intent

WI-299 で adopter finalization の base binding を修正した後、review 済みの
default branch から v0.2.33 を準備します。公開 artifact の検証・install と
adopter acceptance は publication 後の必須 successor WI-301 が担当します。

## Scope

- workspace package version と release example を v0.2.33 に統一する。
- failed staged v0.2.32 の履歴を明示し、書き換えない。
- publication 前に source、documentation、policy、workspace 全体を検証する。
- hosted release workflow だけで manifest、checksum、SBOM、provenance、
  artifact smoke を tag commit に bind して公開する。
- hosted workflow と handoff を設定し、publication 後の install/adopter
  acceptance successor を明示する。

## Out of scope

v0.2.32 の履歴書換え、Runtime governance behavior の追加、外部 Homebrew tap
の公開、publication 後の install/adopter acceptance、adopter technology
matrix の拡張、global Agent/MCP configuration の変更は行いません。

## Acceptance criteria

1. 全 workspace package と Cargo.lock が 0.2.33 となり、三言語 documentation
   route が同じ current baseline を示す。
2. failed staged v0.2.32 は historical のまま保持し、公開 Release と主張しない。
3. tag 前に version consistency、documentation、governance integrity、release
   policy、workspace 全体の test が成功する。
4. hosted workflow が tag commit に bind した manifest、SHA256SUMS、target SBOM、
   provenance、artifact smoke を伴って v0.2.33 を公開する。
5. reviewed release workflow は publication 前 gate 成功後だけ publish し、公開
   artifact check を WI-301 に handoff する。
6. WI-300 は公開 artifact install/adopter acceptance を主張せず、その結論は
   immutable public Release evidence を持つ WI-301 が提供する。

## Verification

- `cargo test --locked --workspace`
- documentation、version consistency、release policy の各 script
- adopter と N-1 acceptance の static test
- hosted release quality、Windows runtime、behavioral-oracle checks
- WI-301 の post-release manifest、checksum、install、repository、adopter receipt
  （本 WI の evidence ではありません）

## Historical boundary

v0.2.32 tag は finalization base-revision defect による staged publication failure
を記録します。その failed truth は保持し、この Work Item は履歴を rewrite せず
新しい v0.2.33 Release を準備します。publication 後の公開 artifact/adopter 結論は
WI-301 が担当します。
