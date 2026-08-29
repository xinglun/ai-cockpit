---
author: AI Cockpit maintainers
title: "WI-389 — reference documentation batch 22"
workItemId: WI-389-reference-documentation-batch-22
description: "六つの uninstall / upgrade reference document を比較し、source authority をコピーしない bounded Rust-native parity を記録します。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-389-reference-documentation-batch-22
terminalArchive: .ai/work-items/archive/WI-389-reference-documentation-batch-22.contract.json
terminalVerification: .ai/evidence/WI-389-reference-documentation-batch-22.verification.json
terminalFinalization: .ai/decisions/WI-389-reference-documentation-batch-22.finalize.b22804ee16ad3895f3bb0d41c77d4d85bdf2cf114f236cb7708e32422284399d.json
terminalDecision: .ai/decisions/WI-389-reference-documentation-batch-22.close.json
canonical: docs/work-items/WI-389-reference-documentation-batch-22.md
---

# WI-389 — reference documentation batch 22

## Intent と boundary

Source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` の六つの path を一つずつ比較します。現在の Rust-native installed-lifecycle / upgrade route で reader-facing な governance meaning を保ち、source installer command、provider authority、historical claim は target repository に持ち込みません。

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/troubleshooting/uninstall.ja.md` | implemented-different-by-design | `docs/reference/installed-lifecycle.ja.md` が read-only inventory、owner confirmation、proposal と別の execution confirmation、bounded removal、receipt verification、evidence retention、Unknown 時の fail-closed recovery を保持します。 |
| `docs/troubleshooting/uninstall.md` | implemented-different-by-design | `docs/reference/installed-lifecycle.md` が read-only inventory、owner confirmation、proposal と別の execution confirmation、bounded removal、receipt verification、evidence retention、Unknown 時の fail-closed recovery を保持します。 |
| `docs/troubleshooting/uninstall.zh-CN.md` | implemented-different-by-design | `docs/reference/installed-lifecycle.zh-CN.md` が read-only inventory、owner confirmation、proposal と別の execution confirmation、bounded removal、receipt verification、evidence retention、Unknown 時の fail-closed recovery を保持します。 |
| `docs/upgrade.ja.md` | implemented-different-by-design | `docs/reference/upgrade.ja.md` が immutable Release/runtime identity、rollback-safe active configuration、conflict/downgrade stop、explicit migration、別途 review された `--upgrade-with-active` recovery を保持します。 |
| `docs/upgrade.md` | implemented-different-by-design | `docs/reference/upgrade.md` が immutable Release/runtime identity、rollback-safe active configuration、conflict/downgrade stop、explicit migration、別途 review された `--upgrade-with-active` recovery を保持します。 |
| `docs/upgrade.zh-CN.md` | implemented-different-by-design | `docs/reference/upgrade.zh-CN.md` が immutable Release/runtime identity、rollback-safe active configuration、conflict/downgrade stop、explicit migration、別途 review された `--upgrade-with-active` recovery を保持します。 |

## Acceptance

- 各 pinned file を読み、inventory に明示的な classification と counterpart mapping を登録します。
- inventory、三言語 comparison、parity record を同期し、`migrate-gap` をゼロに保ちます。
- installed-lifecycle と upgrade route が proposal-before-write、明示的な human confirmation、immutable Release binding、rollback、conflict stop、recovery boundary を説明します。
- source Python/Make command、provider authority、historical evidence を copy / promote しません。
- shared Runtime と object/adopter inheritance boundary を明示します。一つの installed binary、explicit `--repo`、分離された repository fact / evidence です。
- documentation、inventory、governance、installed Runtime verification check が pass します。

## Verification と non-claims

これは semantic/documentation parity であり、source command、JSON wire、provider state compatibility ではありません。uninstall の責務は installed-lifecycle と upgrade route に分散できます。bounded counterpart と non-claim を記録していれば、同名ページがないこと自体は omission ではありません。

[English](WI-389-reference-documentation-batch-22.md) · [简体中文](WI-389-reference-documentation-batch-22.zh-CN.md)
