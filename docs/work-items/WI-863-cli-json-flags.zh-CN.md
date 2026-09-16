---
author: AI Cockpit maintainers
workItemId: WI-863-cli-json-flags
title: 只读 CLI 诊断的一致 JSON 选项
description: 为顶层只读诊断增加明确的机器可读 JSON 标志，同时保持现有事实和退出行为不变。
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-863-cli-json-flags
terminalArchive: .ai/work-items/archive/WI-863-cli-json-flags.contract.json
terminalVerification: .ai/evidence/WI-863-cli-json-flags.verification.json
terminalDecision: .ai/decisions/WI-863-cli-json-flags.close.json
---

# WI-863 — 只读 CLI 诊断的一致 JSON 选项

为顶层 `inspect`、`status` 和 `doctor` 命令补齐文档化的 `--json` 选项。
该选项选择稳定的机器可读投影，不改变事实、退出码或仓库范围。
