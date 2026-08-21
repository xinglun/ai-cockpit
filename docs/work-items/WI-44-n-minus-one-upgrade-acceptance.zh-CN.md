---
author: AI Cockpit maintainers
title: "WI-44 — N-1 adopter 升级验收"
description: "可重复证明既有 adopter 能显式迁移并继续运行的发布后验收。"
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: implementation-acceptance
capabilityClaims:
  - n_minus_one_upgrade
keywords: [work-item, release, upgrade, migration, adopter]
---

# WI-44 — N-1 adopter 升级验收

## 意图与边界

本 Work Item 建立一个发布后验收脚本，只使用两个不可变的公开 Release
归档：旧 Runtime 与新 Runtime。它不构建源码、不运行 workspace binary、不修改
Release truth，也不测试第二种技术栈。

Runtime-only 升级应保持已 attach 工程不变。当 Repository Protocol schema 发生
变化时，新 Runtime 必须报告 `MIGRATION_REQUIRED`，等待显式且经过批准的迁移。

## 验收流程

`tests/release/adopter_upgrade_acceptance.sh` 下载并校验两个归档，创建隔离的
Cargo adopter，用旧 Runtime attach，记录真实 Work Item 与 evidence，然后确认：

1. 新 Runtime 能识别旧 schema；
2. migration plan 只读，未批准的 apply fail closed；
3. 批准后的迁移写入绑定 Runtime 的 receipt，同时旧 evidence 字节不变；
4. 新 Runtime 达到 `COMPATIBLE`，完成 Agent doctor，关闭旧 Work Item 并执行新验证；
5. 生成 `acceptance.json`、Runtime identity、隔离证明、历史摘要与 `SHA256SUMS`。

即使发布后验收失败，脚本也会记录 `releasePublished: true`。失败验收不能把已发布
Release 改写成未发布，也不能让失败 receipt 被复用。

## 重现

仅在新公开 Release 存在后运行：

```bash
tests/release/adopter_upgrade_acceptance.sh \
  --repository xinglun/ai-cockpit \
  --from-tag v0.1.1 \
  --to-tag v0.2.0 \
  --target aarch64-apple-darwin \
  --output ./release-adopter-upgrade-acceptance
```

发布前可运行静态测试：

```bash
bash tests/release/adopter_upgrade_acceptance_test.sh
```

输出是验收 artifact，不是发布前 gate；必须绑定到精确的公开 tag 与归档摘要。
