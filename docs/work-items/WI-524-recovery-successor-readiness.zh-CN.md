---
author: AI Cockpit maintainers
title: "WI-524——恢复 successor readiness 入口门禁绑定"
description: "防止未经证明的 recovery successor 抑制仓库级 pending-close blocker。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-524-recovery-successor-readiness
lastVerifiedBy: WI-524-recovery-successor-readiness
---

[English](WI-524-recovery-successor-readiness.md) · [日本語](WI-524-recovery-successor-readiness.ja.md)

## 目标

将 Repository readiness 绑定到已验证的 recovery successor。只有 successor 已绑定本仓库、通过 manifest 校验、verification 有效并明确 close 后，前驱才能离开入口门禁。

## 范围

- 在抑制 archived predecessor 的 `pending close` blocker 前验证 recovery successor lineage。
- 缺失、stale、foreign、malformed、symlink 或仍开放的 successor 继续 fail closed。
- 增加 repository isolation 回归测试及三语 workflow/parity 文档。
- 保持历史 evidence 不可变，不修改对象工程或全局 Agent/MCP 配置。

## 验收

- 有效且已关闭的终端 successor 只清除其对应 predecessor 的 pending-close blocker。
- 无效或不完整的 successor 继续作为 blocker。
- 并行 repository 保持隔离，既有 lifecycle 行为不回归。
- Rust 测试、文档验收、治理完整性和 hosted CI 全部通过。
- 不手改 Runtime 生成的 evidence 或历史 archive bytes。

## 验证

```text
cargo test --locked -p cockpit-repository --test lifecycle_entry --test recovery_decision -- --test-threads=1
cargo clippy --workspace --all-targets --all-features -- -D warnings
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/docs/work_item_status_consistency.py --repo <repo>
bash tests/docs/documentation_acceptance.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
git diff --check
```
