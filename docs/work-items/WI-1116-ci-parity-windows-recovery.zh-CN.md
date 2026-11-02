---
author: AI Cockpit maintainers
title: "WI-1116-ci-parity-windows-recovery — Windows 测试 cfg 与治理 parity 恢复"
description: "修复 PR1077 的 CI 编译与真实治理投影，不扩展产品功能。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1116-ci-parity-windows-recovery
status: in_progress
authority: native-contract
lastVerifiedBy: WI-1116-ci-parity-windows-recovery
---

# WI-1116-ci-parity-windows-recovery — Windows 测试 cfg 与治理 parity 恢复

本页记录创建时已批准的计划。当前授权、证据与完成状态以原生 Contract／Runtime 为准；预登记路径不等于执行回执。

## 最小修改

- 仅将既有有限 Unix 测试兼容 wrapper 限定为 `cfg(all(test, unix))`；两个调用点原本就在 Unix 测试模块中。保留生产清理、Windows 测试选择与 `-D warnings`。
- 三语 WI-1114 parity 行依据真实恢复链、本地验证和关闭证据更新；补充本后继必需的三语工作项页与真实预归档登记。
- 沿 WI-1114 → WI-1115 → WI-1116 原生恢复链继续使用 PR #1077；不改写已关闭前驱及任何失败历史。

## 验证与边界

Contract 声明六项本地检查：parity gate 及其回归、repository collaboration 测试、repository 全目标严格 Clippy、workspace fmt check、canonical repository prechecks。每次材料审查绑定完整新请求，仅可使用已确认的可读已提交 Rust 语法 Unknown 机制。本地 Linux 通过不代表 Windows 通过；真实 hosted quality 和两个不变的 Windows suite 仍是 PR 合并条件。

本计划页不声称验证、归档、关闭、合并或发布已完成。此前 118/1 失败原因未知、Outcome 显示 ENAMETOOLONG 风险保留。Mac 实机执行／完整后代监督及必要 HCI North Star 矫正留给随后独立 WI，之后才是版本／Sentinel 验收。本范围不包含凭证、安全权限、默认 Runtime 或缓存清理变更。
