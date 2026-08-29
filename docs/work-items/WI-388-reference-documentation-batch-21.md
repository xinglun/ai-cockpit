---
author: AI Cockpit maintainers
title: "WI-388 — reference documentation batch 21"
workItemId: WI-388-reference-documentation-batch-21
description: "Compare six pinned troubleshooting, adoption-stability, and threat-model documents and record bounded Rust-native parity without copying source authority."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-388-reference-documentation-batch-21
---

# WI-388 — reference documentation batch 21

## Intent and boundary

Compare the six pinned paths below at source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` one by one. Preserve their reader-facing governance meaning through the current Rust-native documentation routes, while keeping source commands, provider authority, and historical stability claims out of the target.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/security/threat-model.md` | implemented-different-by-design | Tri-language `docs/security/threat-model.*` preserves assets, trust boundaries, fail-closed threats, and external-control limits; it does not claim universal malicious-intent detection or enterprise certification. |
| `docs/template-adopter-stability-matrix.md` | implemented-different-by-design | `docs/reference/final-replacement-acceptance.md`, `docs/getting-started/standard-adoption-guide.md`, `docs/reference/ci-release-evidence.md`, and the adopter harness distribute evidence-kind and adoption boundaries; template-only evidence is not external stability proof. |
| `docs/troubleshooting.md` | implemented-different-by-design | Tri-language `docs/reference/troubleshooting.*` gives stop-state, recovery, and evidence-preservation guidance rather than a compatibility-only redirect. |
| `docs/troubleshooting/installation.ja.md` | implemented-different-by-design | Japanese install, strict verification, and troubleshooting pages preserve uncertainty stops and explicit attachment. |
| `docs/troubleshooting/installation.md` | implemented-different-by-design | English install, strict verification, and troubleshooting pages preserve uncertainty stops, immutable artifact checks, and explicit attachment. |
| `docs/troubleshooting/installation.zh-CN.md` | implemented-different-by-design | Chinese install, strict verification, and troubleshooting pages preserve the same recovery and repository-context boundary. |

## Acceptance

- Each pinned file is read and has an explicit inventory classification and counterpart mapping.
- The tri-language comparison, parity, and Work Item records agree; `migrate-gap` remains zero.
- No source Python/Make command, provider authority, or historical evidence is copied or promoted.
- The shared Runtime and object/adopter inheritance boundary remain explicit: one installed binary, explicit `--repo`, isolated repository facts and evidence.
- Documentation, inventory, governance, and installed Runtime verification checks pass.

## Verification and non-claims

This is semantic/documentation parity, not source command, JSON-wire, or provider-state compatibility. The target may distribute a responsibility across several reader routes; absence of a same-named file is not an omission when the bounded counterpart and non-claim are recorded.

[简体中文](WI-388-reference-documentation-batch-21.zh-CN.md) · [日本語](WI-388-reference-documentation-batch-21.ja.md)
