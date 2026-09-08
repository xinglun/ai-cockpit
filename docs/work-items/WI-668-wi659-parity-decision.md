---
author: AI Cockpit maintainers
title: "WI-668 — WI-659 parity decision link repair"
description: "Project the exact versioned supersede decision for WI-659 in all reference-parity ledgers without changing historical records."
workItemId: WI-668-wi659-parity-decision
audience: [maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-668-wi659-parity-decision
capabilityClaims: [governance_integrity, reference_parity]
---

# WI-668 — WI-659 parity decision link repair

[简体中文](WI-668-wi659-parity-decision.zh-CN.md) · [日本語](WI-668-wi659-parity-decision.ja.md)

## Intent and boundary

The closed WI-659 predecessor has a canonical successor recovery and a
digest-versioned supersede recovery. The three reference-parity projections
must expose the exact terminal supersede decision so the governance gate can
verify the historical chain. This Work Item changes only documentation links;
the WI-659 archive, evidence, recovery, and close records remain immutable.

## Scope

- Add the exact versioned WI-659 supersede decision path and superseded close
  path to all three reference-parity ledgers.
- Keep the English, Simplified Chinese, and Japanese projections semantically
  aligned.
- Add this Work Item's tri-language documentation counterparts.

## Out of scope

WI-659 archive, verification, recovery, close, or any other generated `.ai`
record; WI-664/WI-665 documentation; Runtime or Rust code; schemas; tests;
release artifacts; and global Agent/MCP configuration.

## Acceptance

- Each WI-659 parity row retains its archive, verification, and canonical
  recovery references and includes the exact versioned supersede recovery and
  superseded close decision paths.
- The governance integrity gate reports no WI-659 `missing_parity_decision`
  finding.
- WI-659 archive, evidence, recovery, and close bytes are unchanged.

## Verification and terminal records

Use the installed Runtime with explicit `--repo`, targeted parity and
documentation checks, `git diff --check`, and the repository's hosted checks.
After reviewed merge, record the archive, verification, finalization, and
close paths declared by the Runtime.
