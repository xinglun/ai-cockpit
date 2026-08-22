---
author: AI Cockpit maintainers
title: "WI-117 Release adopter toolchain 隔离"
description: "将 N-1 验收绑定到已有 Rust toolchain，禁止隐式下载。"
audience:
  - maintainer
  - release-engineer
status: implemented
authority: canonical
lastVerifiedBy: release-adopter-toolchain-regression
capabilityClaims:
  - bounded_release_acceptance
  - isolated_toolchain_identity
---

# WI-117：Release adopter toolchain 隔离

## 目标

在 harness 使用隔离 HOME、TMPDIR 和 CARGO_HOME 时，让发布后 adopter 与 N-1
验收保持确定性。

## 范围

N-1 harness 解析宿主机的 Rustup home 和 active toolchain，并显式传入隔离
fixture 命令；任一 identity 不可用时，拒绝隐式网络 toolchain 下载。Runtime
Protocol 语义和全局 Rust 安装不在范围内。

## 验收

- 环境变量缺失时，`RUSTUP_HOME` 回退到 `rustup show home`。
- 从 active toolchain 解析 `RUSTUP_TOOLCHAIN`，并传给每个隔离 Cargo/Runtime 调用。
- toolchain identity 缺失时，在创建无界 fixture 前 fail closed。
- cleanup evidence 与验收 truth 分离，只删除经过验证的临时 run root。
- 英文、中文和日文发布文档同步说明边界。

## 验证

```text
bash tests/release/adopter_upgrade_acceptance_test.sh
bash tests/docs/documentation_acceptance.sh
git diff --check
```

## Outcome

状态：**已实现；toolchain identity 与有界清理已显式化。**
