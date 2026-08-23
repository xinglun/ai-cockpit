---
author: AI Cockpit maintainers
title: "WI-193 — Release acceptance isolation hardening"
description: "Make adopter harness cleanup, source manifests, and allowed-root symlink containment fail closed."
audience:
  - maintainer
  - reviewer
workItemId: WI-193-release-acceptance-isolation
status: historical
authority: canonical
lastVerifiedBy: WI-195-governance-recovery-gate
---

# WI-193 — Release acceptance isolation hardening

WI-193 is an immutable historical predecessor. Its lifecycle could not be
refreshed from the correct published Runtime context, so the predecessor stays
red/blocked and is never presented as green completion. Its bounded delivery
continues in WI-195.

The preserved implementation installed the EXIT cleanup handler before either
adopter harness creates its temporary run root. Toolchain-resolution and setup failures therefore emit
checksummed `acceptance.json` and `cleanup.json` receipts and leave no run root.

Source isolation now compares deterministic typed manifests for every tracked
or untracked source path and all `.ai` entries, including ignored content. Only
the declared output subtree is excluded; output-ancestor directory metadata is
normalized so writing evidence below the source checkout does not create a
false mutation. TMPDIR and CARGO_HOME manifests retain symlink metadata,
literal targets, and resolved targets, and reject targets outside their
classified allowed root.

The committed v0.2.23 public adopter and v0.2.22 → v0.2.23 N-1 receipts both
name `aarch64-apple-darwin`. Linux x86_64 remains Release-workflow CI coverage,
not a claimed second complete adopter-evidence target. No published Release,
tag, historical evidence, Runtime Core, crates, or CI parity file is rewritten.
The immutable recovery receipt is [WI-193 recovery](../../.ai/decisions/WI-193-release-acceptance-isolation.recovery.json).

[简体中文](WI-193-release-acceptance-isolation.zh-CN.md) ·
[日本語](WI-193-release-acceptance-isolation.ja.md)
