---
author: AI Cockpit maintainers
title: "首次校准"
description: "以可审查方式首次确认一条 repository 质量命令。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - configuration
---

# 首次校准

只有在安装、repository 检查、attach 和 `doctor` 均完成后才运行本路线。
先获取候选，不改变正式 profile：

```bash
repo=/path/to/repository
ai-cockpit profile propose --repo "$repo"
```

把候选与 repository-owned 文档和 hosted CI 对照。请工程 owner 解决 executable、arguments、
工作目录、toolchain、environment、service、credential 与 coverage 的所有 Unknown。

只确认 owner 批准的准确命令。例如批准命令为 `cargo test --workspace` 时：

```bash
ai-cockpit profile confirm --repo "$repo" --program cargo --args test,--workspace
ai-cockpit doctor --repo "$repo"
```

本地命令通过只是一份有边界的本地证据，不是 branch protection、provider、production 或
enterprise 证据。如果候选错误或必要事实仍为 Unknown，不要确认；先修正 repository-owned
决定，再重新运行只读 proposal。

继续完成[采用方配置](adopter-configuration.zh-CN.md)，然后运行[首个 Work Item](first-work-item.zh-CN.md)。

[Profile 校准](calibration.zh-CN.md) | [English](first-calibration.md) | [日本語](first-calibration.ja.md)
