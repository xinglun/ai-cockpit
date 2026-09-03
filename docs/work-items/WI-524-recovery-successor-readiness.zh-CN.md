---
author: AI Cockpit maintainers
title: "WI-524——恢复 successor readiness 入口门禁绑定"
description: "在抑制归档前驱 blocker 前要求已验证的 recovery successor。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-524-recovery-successor-readiness
lastVerifiedBy: WI-524-recovery-successor-readiness
terminalArchive: .ai/work-items/archive/WI-524-recovery-successor-readiness.contract.json
terminalVerification: .ai/evidence/WI-524-recovery-successor-readiness.verification.json
terminalFinalization: .ai/decisions/WI-524-recovery-successor-readiness.finalize.cab9a20e63481aea75e8801ff86a94cec5ddc4c99fe9602500b43537567272c6.json
terminalDecision: .ai/decisions/WI-524-recovery-successor-readiness.close.json
---

[English](WI-524-recovery-successor-readiness.md) · [日本語](WI-524-recovery-successor-readiness.ja.md)

## 目标

只有 successor 已绑定本仓库、通过 manifest 校验、verification 有效并明确 close，Repository readiness 才能解除 predecessor 的入口 blocker。

## 范围与验收

- 缺失、stale、foreign、malformed、symlink 或仍开放的 successor 继续 fail closed。
- 增加隔离回归测试及三语 workflow/parity 投影，保留历史 evidence，不修改对象工程或全局配置。
- 仅有效且已关闭的终端 successor 清除对应 predecessor blocker；Rust、文档、治理和 hosted CI 全部通过。

## 验证

```text
cargo test --locked -p cockpit-repository --test lifecycle_entry --test recovery_decision -- --test-threads=1
cargo clippy --workspace --all-targets --all-features -- -D warnings
python3 tests/ci/governance_integrity_gate.py --repo <repo>
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
```
