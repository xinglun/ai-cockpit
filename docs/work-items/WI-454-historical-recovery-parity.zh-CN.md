---
author: AI Cockpit maintainers
title: "WI-454——历史 recovery parity 质量门修正"
workItemId: WI-454-historical-recovery-parity
description: "要求 parity 投影正确识别带终态 recovery successor 的已关闭 predecessor。"
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-454-historical-recovery-parity
terminalArchive: .ai/work-items/archive/WI-454-historical-recovery-parity.contract.json
terminalVerification: .ai/evidence/WI-454-historical-recovery-parity.verification.json
terminalFinalization: .ai/decisions/WI-454-historical-recovery-parity.finalize.json
terminalDecision: .ai/decisions/WI-454-historical-recovery-parity.close.json
---

# WI-454——历史 recovery parity 质量门修正

本 Work Item 修正不可变 predecessor 已有有效终态 recovery successor 时的
CI parity 投影。predecessor 字节保持不变；质量门现在要求 successor 的归档与
close 证据。

[English](WI-454-historical-recovery-parity.md) · [日本語](WI-454-historical-recovery-parity.ja.md)

## 验证

- `bash tests/ci/governance_integrity_gate_test.sh`
- hosted quality、Windows runtime 与 reference-oracle 检查
