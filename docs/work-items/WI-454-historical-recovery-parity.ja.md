---
author: AI Cockpit maintainers
title: "WI-454 — historical recovery parity gate correction"
workItemId: WI-454-historical-recovery-parity
description: "終端 recovery successor を持つ closed predecessor を parity projection が認識するようにする。"
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-454-historical-recovery-parity
terminalArchive: .ai/work-items/archive/WI-454-historical-recovery-parity.contract.json
terminalVerification: .ai/evidence/WI-454-historical-recovery-parity.verification.json
terminalFinalization: .ai/decisions/WI-454-historical-recovery-parity.finalize.json
terminalDecision: .ai/decisions/WI-454-historical-recovery-parity.close.json
---

# WI-454 — historical recovery parity gate correction

この Work Item は immutable な predecessor に有効な終端 recovery successor が
ある場合の CI parity projection を修正します。predecessor bytes は変更せず、
successor の archive と close evidence を必須にします。

[English](WI-454-historical-recovery-parity.md) · [简体中文](WI-454-historical-recovery-parity.zh-CN.md)

## Verification

- `bash tests/ci/governance_integrity_gate_test.sh`
- hosted quality、Windows runtime、reference-oracle checks
