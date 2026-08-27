---
author: AI Cockpit maintainers
title: "WI-323 — reference documentation foundation"
workItemId: WI-323-reference-documentation-foundation
description: "pinned reference の 9 documentation path を比較し、Rust-native adopter/Agent boundary を記録します。"
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-323-reference-documentation-foundation
---

# WI-323 — reference documentation foundation

## Intent と goal

pinned source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の次の 9 deferred
path を一つずつ比較します。この repository と将来の adopter repository に必要な
governance semantics を保ちつつ、shared Runtime を repository 外部に置き、
repository state を分離し、全 command に明示的な `--repo` を要求します。

ユーザーが提供した Cursor adopter feedback はこの batch の external observation
です。Stable lifecycle stdout JSON、human handoff/replay、repository entry gate、
start 前の cleanliness は現行 Runtime で確認済みです。Cursor panel presentation、
diagnostic remediation、close-gap convenience、optional controls scaffold は
別の product decision とします。

## 比較対象

- `docs/contributing/installation-document-maintenance.md`
- `docs/current/README.md`
- `docs/design/harden-work-item-pr-closure.md`
- `docs/distribution.md`
- `docs/enterprise-security-boundary.md`
- `docs/examples/trust-layer-demo.sh`
- `docs/features/human-benefit-report.md`
- `docs/features/human-benefit-report.zh-CN.md`
- `docs/features/human-benefit-report.ja.md`

各 path に inventory classification と non-empty reason を記録します。8 件は
`implemented-different-by-design`、offline trust demo は `reference-only` です。
`migrate-gap` を隠しません。

## Scope と boundary

comparison inventory/generator と regression assertion、tri-language の reference
comparison、Human Benefit Report、そしてこの tri-language Work Item record を更新
します。source Make/Python/installer/demo boundary、semantic (wire/byte ではない)
parity、shared Runtime と repository-local `.ai/` state を明記します。

Runtime command/lifecycle semantics の変更、source Python/Make/YAML/JSON wire の copy、
`Makefile.ai` 要件、global Agent/MCP 設定、historical evidence の rewrite、Release
publish は対象外です。

## Acceptance と verification

1. 9 pinned source path を一つずつ読み、evidence-backed counterpart または明示的な
   reference-only decision を記録します。
2. Inventory は WI-323 を 9 records（8 implemented-different-by-design、1
   reference-only）含み、この batch に deferred/migrate-gap を残しません。
3. English、Simplified Chinese、日本語の comparison と Human Benefit Report は
   意味を揃え、互いの language route を link します。
4. 人向け output は `work-item outcome --repo ...`、MCP `work_item_outcome`、
   stdout lifecycle JSON と human handoff の違い、report order、evidence count、
   stale/malformed stop、Contract-authored acceptance の保持を説明します。
5. CLI が Cursor chat panel を開くこと、source 固有の
   `implementation_approach_report`、Make/Python generator、trust-demo authority
   を target が提供することを claim しません。
6. installed Runtime で明示的 repository context を検証し、宣言した check がすべて
   pass し、無関係な bytes を変更しません。

## Evidence

immutable source/target baseline は active Contract に記録されます。generated
inventory、documentation acceptance、diff check、Runtime verification receipt が
この batch の evidence です。

[English](WI-323-reference-documentation-foundation.md) ·
[简体中文](WI-323-reference-documentation-foundation.zh-CN.md)
