---
author: AI Cockpit maintainers
title: "WI-114 发布 adopter 生命周期顺序"
description: "使公开 Release 与 N-1 验收遵守 Runtime 生命周期契约。"
audience:
  - maintainer
  - reviewer
  - release_operator
status: implemented
authority: canonical
lastVerifiedBy: v0.2.8-adopter-acceptance
capabilityClaims:
  - release_adopter_acceptance
  - fail_closed_lifecycle
---

# WI-114：发布 adopter 生命周期顺序

## 目标

修复公开 adopter 与 N-1 验收脚本，使其按 fail-closed 生命周期契约先记录
`preflight`，再记录 `checkpoint`。

## 为什么需要此 Work Item

不可变的 v0.2.8 Release 暴露了验收脚本缺陷：两个脚本都执行了
`start → checkpoint → preflight`。v0.2.8 Runtime 正确拒绝了这个顺序。本
Work Item 只修改验收脚本及其回归检查，不改写已发布 Release 或其 receipt。

## 验收标准

- 公开 adopter 验收先记录 `lifecycle-preflight`，再记录
  `lifecycle-checkpoint`。
- N-1 验收先记录 `old-preflight`，再记录 `old-checkpoint`。
- N-1 验收先用旧 Runtime 关闭旧 Work Item 并保留历史 evidence，再创建迁移后的新
  Work Item，用 v0.2.8 记录 `new-preflight` → `new-checkpoint` → `new-verify`。
- 静态测试在任一顺序回退时失败。
- 两个脚本继续只使用不可变公开 artifact，并保留清理、隔离、checksum 与
  `first-adopter-smoke=not_ready` 断言。
- 使用公开 v0.2.8 重新运行时通过，且不使用源码或 workspace binary 兜底。
- N-1 harness 不伪造旧 summary 的新生命周期状态：迁移前关闭旧生命周期，迁移后使用
  新 Work Item 的新生命周期。

## 验证

```text
bash tests/release/adopter_acceptance_test.sh
bash tests/release/adopter_upgrade_acceptance_test.sh
AI_COCKPIT_RUN_PUBLIC_ACCEPTANCE=1 AI_COCKPIT_ACCEPTANCE_TARGET=aarch64-apple-darwin \
  bash tests/release/adopter_acceptance_test.sh
bash tests/release/adopter_upgrade_acceptance.sh \
  --repository xinglun/ai-cockpit --from-tag v0.2.7 --to-tag v0.2.8 \
  --target aarch64-apple-darwin --output ./release-adopter-upgrade-acceptance
```

已发布的 v0.2.8 Release 保持不可变。任何失败 receipt 都作为失败的发布后
证据保留，不能被用于宣称 Release 成功。
修复后的公开 adopter 与 N-1 运行均通过并留下 cleanup receipt；N-1 使用公开
v0.2.7 → v0.2.8 版本对，并在创建迁移后 Work Item 前保留旧 evidence bytes。

## Outcome

状态：**验收脚本修复；发布事实保持不可变。**
