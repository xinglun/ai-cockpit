---
author: AI Cockpit maintainers
title: 治理复杂度边界
description: Rust Runtime 如何观察仓库增长，同时不复制参考源维护工具或改写审计历史。
audience:
  - contributor
  - maintainer
  - adopter
status: reference
authority: canonical
lastVerifiedBy: WI-345-reference-governance-cost-batch-15
---

# 治理复杂度边界

参考源工程有 Python/Make 复杂度报告。Rust Runtime 不包含该源项目专属的扫描器、阈值或全局复杂度预算。这是有意的边界：维护报告不是治理决定，不能假定它描述 adopter 仓库。

## Rust Runtime 提供的事实

所有命令都必须显式绑定 repository：

```sh
ai-cockpit inspect --repo /path/to/repository
ai-cockpit status --repo /path/to/repository
ai-cockpit doctor --repo /path/to/repository
ai-cockpit diagnose --repo /path/to/repository --work-item WI-123
```

`inspect` 报告当前 snapshot 和 changed paths；`status` 报告 repository 兼容性与 archive 计数；`doctor` 检查已 attach 的 Runtime 边界；选择 Work Item 后，`diagnose` 报告已测量的 snapshot/verification 成本，缺少的测量保持为 `unknown`。

repository CI integrity gate 检查 archive 配对、parity 元数据和文档一致性。它保护当前仓库事实，不替代参考源的历史复杂度扫描器，也不会推导复杂度阈值。

## Archive 与增长规则

归档的 Contract、Summary、Outcome、evidence 和 decision 字节属于不可变审计历史。历史增长本身不能授权删除、压缩或修改其他 Work Item。任何 index 修复或历史压缩都必须是独立、经过 review 的 Work Item，并明确 retention 决定。

成本与性能观察只是 advisory，不能降低 required verification tier、移除 protected check，或把未知测量变成绿色 Outcome。`VerificationTier` 与 `EvidenceAssurance` 始终是两个独立维度。

## 对象工程边界

adopter repository 通过共享 Runtime 继承相同的 request-scoped 规则：每个命令带 `--repo`，archive/evidence 状态只属于该 repository。参考源 Python scanner、Make target 和源阈值文件不会被悄悄安装到对象工程。

