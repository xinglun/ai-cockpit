---
author: AI Cockpit maintainers
title: "WI-346 — Governance Profile と Cockpit Status の読み方"
workItemId: WI-346-reference-governance-profiles-status
description: "6 つの pinned governance/status document を比較し、bounded な三言語 Rust guide を追加します。"
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-346-reference-governance-profiles-status
capabilityClaims:
  - reference_parity
---

# WI-346 — Governance Profile と Cockpit Status の読み方

[English](WI-346-reference-governance-profiles-status.md) · [简体中文](WI-346-reference-governance-profiles-status.zh-CN.md)

## Intent と boundary

この Work Item は pinned reference の 6 document、3 つの Governance Profile と 3 つの human status reader を一つずつ比較します。
adopter が利用できる guidance を追加しますが、source Make/Python orchestration、source JSON wire shape、provider/global configuration はコピーしません。

対象は shared Rust Runtime と repository-local documentation です。Object/adopter repository は明示的な `--repo`、
隔離された `.ai/` state、human-owned Contract decision、visible Outcome handoff を継承します。

## File-by-file decision

| Pinned reference path | Classification | Bounded target decision |
| --- | --- | --- |
| `docs/reference/governance-profiles.ja.md` | `implemented-different-by-design` | 日本語 route で proportional Light/Standard/Strict、release escalation、mandatory control の fail-closed、tier/assurance/cost 分離を説明します。 |
| `docs/reference/governance-profiles.md` | `implemented-different-by-design` | 英語 canonical route で source profile guidance を `gate --repo`、typed Contract/verification evidence、Rust/CI boundary に対応づけます。 |
| `docs/reference/governance-profiles.zh-CN.md` | `implemented-different-by-design` | 中国語 route で同じ fact を説明し、source-only command を主張しません。 |
| `docs/reference/how-to-read-cockpit-status.ja.md` | `implemented-different-by-design` | 日本語 reader route で human Outcome color、stop condition、evidence boundary、next action を説明します。 |
| `docs/reference/how-to-read-cockpit-status.md` | `implemented-different-by-design` | 英語 canonical reader route で source reader label を Rust Outcome section に対応づけます。 |
| `docs/reference/how-to-read-cockpit-status.zh-CN.md` | `implemented-different-by-design` | 中国語 reader route で Contract 原文と human decision boundary を保持します。 |

Comparison/parity ledger、reference index link、inventory script/manifest、本三言語 record が delivery evidence です。

## Acceptance と verification

- 各 pinned path は inventory に一度だけ記録され、上記 classification で deferred/migrate-gap は残りません。
- 6 page は English、Simplified Chinese、Japanese の reference index と相互に link されます。
- Profile page は `VerificationTier`、`EvidenceAssurance`、cost の直交性、release が operation class であること、
  mandatory control と invalid override の fail-closed、source Make/Python command が Rust requirement でないことを示します。
- Status page は 🟢/🟡/🔴 と `unknown` が semantic signal であること、Contract 原文、CLI/MCP human handoff と machine JSON の違い、
  object/adopter の `--repo` boundary を示します。
- 三言語の comparison/parity と machine ledger は pinned source identity と current count を一致させます。
- Documentation、inventory、governance integrity、format、lint、locked workspace verification が成功します。Runtime code と global Agent/MCP configuration は変更しません。

Pinned reference commit: `e5acb677da6621004d96f0ef353c58fe8d3acfbf`。
Target base commit: `8bf06612a0f0a8adda0aacfdf65e17ece9c2ca0f`。

