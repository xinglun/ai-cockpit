---
author: AI Cockpit maintainers
title: "Repository profile 校准"
description: "不猜测 repository 事实，由人确认工程质量命令。"
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - configuration
---

# Repository profile 校准

Attach 可以检测候选 build system，但不会决定哪个命令是 repository 的质量基线。
先检查当前 profile，再让 Runtime 给出只读候选：

```bash
repo=/path/to/repository
ai-cockpit status --repo "$repo"
ai-cockpit profile propose --repo "$repo"
```

proposal 不是已应用的变更。Repository owner 必须确认工作目录、executable、arguments、
toolchain、凭据边界、coverage 与 hosted CI 对应项。不能只因为 manifest、工程文件或 wrapper
存在就推断命令。

审查后确认一条准确的工程自有命令。以下例子只适用于 owner 已选择
`cargo test --workspace` 的 Rust repository：

```bash
ai-cockpit profile confirm --repo "$repo" --program cargo --args test,--workspace
ai-cockpit status --repo "$repo"
```

其他 stack 使用各自批准的 program 与 arguments。校准不会安装 toolchain、认证 provider 或
证明 hosted CI。Unknown 必须保持 Unknown，并阻断依赖这些事实的声明。

[首次校准](first-calibration.zh-CN.md) | [English](calibration.md) | [日本語](calibration.ja.md)
