---
author: AI Cockpit maintainers
title: "WI-346 — governance profiles and Cockpit status reading"
workItemId: WI-346-reference-governance-profiles-status
description: "Compare six pinned governance-profile and status-reading documents and add bounded tri-language Rust guidance."
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-346-reference-governance-profiles-status
capabilityClaims:
  - reference_parity
---

# WI-346 — governance profiles and Cockpit status reading

[简体中文](WI-346-reference-governance-profiles-status.zh-CN.md) · [日本語](WI-346-reference-governance-profiles-status.ja.md)

## Intent and boundary

This Work Item compares six pinned reference documents one by one: the three
governance-profile pages and the three human status-reading pages. It makes the
useful guidance available to adopters without copying source Make/Python
orchestration, source JSON wire shapes, or provider/global configuration.

The target is the shared Rust Runtime and its repository-local documentation.
The object/adopter contract remains explicit `--repo`, isolated `.ai/` state,
human-owned Contract decisions, and a visible Outcome handoff.

## File-by-file decisions

| Pinned reference path | Classification | Bounded target decision |
| --- | --- | --- |
| `docs/reference/governance-profiles.ja.md` | `implemented-different-by-design` | Add a Japanese route covering proportional Light/Standard/Strict quality routing, release escalation, fail-closed mandatory controls, and the separation of tier, assurance, and cost. |
| `docs/reference/governance-profiles.md` | `implemented-different-by-design` | Add the canonical English route and map source profile guidance to `gate --repo`, typed Contract/verification evidence, and the Rust/CI boundary. |
| `docs/reference/governance-profiles.zh-CN.md` | `implemented-different-by-design` | Add the Chinese route with the same facts and no source-only command claim. |
| `docs/reference/how-to-read-cockpit-status.ja.md` | `implemented-different-by-design` | Add a Japanese reader route for human Outcome colors, stop conditions, evidence boundaries, and next actions. |
| `docs/reference/how-to-read-cockpit-status.md` | `implemented-different-by-design` | Add the canonical English reader route and map source reader labels to the Rust Outcome sections. |
| `docs/reference/how-to-read-cockpit-status.zh-CN.md` | `implemented-different-by-design` | Add the Chinese reader route, preserving Contract-language text and human decision boundaries. |

The comparison and parity ledgers, reference index links, inventory script and
manifest, and this tri-language record are part of the delivery evidence.

## Acceptance and verification

- Each pinned path occurs exactly once in the inventory with the listed
  classification and no deferred or migrate-gap record.
- The six pages are linked from the reference indexes and from each other in
  English, Simplified Chinese, and Japanese.
- Profile pages state that `VerificationTier`, `EvidenceAssurance`, and cost
  are orthogonal; release is an operation class; mandatory controls and invalid
  overrides fail closed; and source Make/Python commands are not Rust
  requirements.
- Status pages explain 🟢/🟡/🔴 and `unknown` as semantic signals, preserve
  original Contract text, distinguish CLI/MCP human handoff from machine JSON,
  and make the object/adopter `--repo` boundary explicit.
- The three comparison pages, three parity pages, and machine ledger agree on
  the pinned source identity and current counts.
- Documentation, inventory, governance integrity, formatting, lint, and locked
  workspace verification pass. No Runtime code or global Agent/MCP configuration
  is changed.

Pinned reference commit: `e5acb677da6621004d96f0ef353c58fe8d3acfbf`.
Target base commit: `8bf06612a0f0a8adda0aacfdf65e17ece9c2ca0f`.

