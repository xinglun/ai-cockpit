---
author: AI Cockpit maintainers
title: "WI-454 — historical recovery parity gate correction"
workItemId: WI-454-historical-recovery-parity
description: "Require parity projections to recognize a closed predecessor with a terminal recovery successor."
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-454-historical-recovery-parity
terminalArchive: .ai/work-items/archive/WI-454-historical-recovery-parity.contract.json
terminalVerification: .ai/evidence/WI-454-historical-recovery-parity.verification.json
terminalDecision: .ai/decisions/WI-454-historical-recovery-parity.close.json
---

# WI-454 — historical recovery parity gate correction

This Work Item fixes the CI parity projection for an immutable predecessor
that has a valid terminal recovery successor. The predecessor bytes remain
unchanged; the gate now requires the successor archive and close evidence.

[简体中文](WI-454-historical-recovery-parity.zh-CN.md) · [日本語](WI-454-historical-recovery-parity.ja.md)

## Verification

- `bash tests/ci/governance_integrity_gate_test.sh`
- hosted quality, Windows runtime, and reference-oracle checks
