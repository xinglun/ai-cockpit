---
author: AI Cockpit maintainers
title: "WI-389 — reference documentation batch 22"
workItemId: WI-389-reference-documentation-batch-22
description: "Compare six pinned uninstall and upgrade documents and record bounded Rust-native parity without copying source authority."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-389-reference-documentation-batch-22
terminalArchive: .ai/work-items/archive/WI-389-reference-documentation-batch-22.contract.json
terminalVerification: .ai/evidence/WI-389-reference-documentation-batch-22.verification.json
terminalFinalization: .ai/decisions/WI-389-reference-documentation-batch-22.finalize.b22804ee16ad3895f3bb0d41c77d4d85bdf2cf114f236cb7708e32422284399d.json
terminalDecision: .ai/decisions/WI-389-reference-documentation-batch-22.close.json
---

# WI-389 — reference documentation batch 22

## Intent and boundary

Compare the six pinned paths below at source commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf` one by one. Preserve their reader-facing governance meaning through the current Rust-native installed-lifecycle and upgrade routes, while keeping source installer commands, provider authority, and historical claims out of the target.

| Pinned reference path | Classification | Rust counterpart / bounded decision |
| --- | --- | --- |
| `docs/troubleshooting/uninstall.ja.md` | implemented-different-by-design | `docs/reference/installed-lifecycle.ja.md` preserves read-only inventory, owner confirmation, proposal and separate execution confirmation, bounded removal, receipt verification, evidence retention, and fail-closed unknown recovery. |
| `docs/troubleshooting/uninstall.md` | implemented-different-by-design | `docs/reference/installed-lifecycle.md` preserves read-only inventory, owner confirmation, proposal and separate execution confirmation, bounded removal, receipt verification, evidence retention, and fail-closed unknown recovery. |
| `docs/troubleshooting/uninstall.zh-CN.md` | implemented-different-by-design | `docs/reference/installed-lifecycle.zh-CN.md` preserves read-only inventory, owner confirmation, proposal and separate execution confirmation, bounded removal, receipt verification, evidence retention, and fail-closed unknown recovery. |
| `docs/upgrade.ja.md` | implemented-different-by-design | `docs/reference/upgrade.ja.md` preserves immutable Release/runtime identity, rollback-safe active configuration, conflict and downgrade stops, explicit migration, and separately reviewed `--upgrade-with-active` recovery. |
| `docs/upgrade.md` | implemented-different-by-design | `docs/reference/upgrade.md` preserves immutable Release/runtime identity, rollback-safe active configuration, conflict and downgrade stops, explicit migration, and separately reviewed `--upgrade-with-active` recovery. |
| `docs/upgrade.zh-CN.md` | implemented-different-by-design | `docs/reference/upgrade.zh-CN.md` preserves immutable Release/runtime identity, rollback-safe active configuration, conflict and downgrade stops, explicit migration, and separately reviewed `--upgrade-with-active` recovery. |

## Acceptance

- Each pinned file is read and has an explicit inventory classification and counterpart mapping.
- Inventory records are synchronized with the tri-language comparison and parity records; `migrate-gap` remains zero.
- Installed-lifecycle and upgrade routes document proposal-before-write, explicit human confirmation, immutable Release binding, rollback, conflict stops, and recovery boundaries.
- No source Python/Make command, provider authority, or historical evidence is copied or promoted.
- The shared Runtime and object/adopter inheritance boundary remain explicit: one installed binary, explicit `--repo`, isolated repository facts and evidence.
- Documentation, inventory, governance, and installed Runtime verification checks pass.

## Verification and non-claims

This is semantic/documentation parity, not source command, JSON-wire, or provider-state compatibility. The target may distribute a responsibility across installed-lifecycle and upgrade routes; absence of a same-named uninstall page is not an omission when the bounded counterpart and non-claim are recorded.

[简体中文](WI-389-reference-documentation-batch-22.zh-CN.md) · [日本語](WI-389-reference-documentation-batch-22.ja.md)
