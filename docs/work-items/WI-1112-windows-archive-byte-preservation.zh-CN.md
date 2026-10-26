---
author: AI Cockpit maintainers
title: "WI-1112-windows-archive-byte-preservation — Windows 归档字节保真"
description: "修复 Windows 本地 Git fixture 与不可变证据检出的失败，不削弱字节完整性检查。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1112-windows-archive-byte-preservation
status: in_progress
authority: native-contract
lastVerifiedBy: WI-1112-windows-archive-byte-preservation
---

# WI-1112 — Windows 归档字节保真

此后继项处理 [PR #1074](https://github.com/xinglun/ai-cockpit/pull/1074) 的 Windows repository-library job 在 run [40271040290](https://github.com/xinglun/ai-cockpit/actions/runs/40271040290/job/121404907259) 中出现的四个失败。两个固定历史测试无法从 Windows 路径克隆本地仓库；另两个测试在 `.ai` 证据检出时被转换为 CRLF 后，严格 Markdown 字节比较失败。

## 范围与边界

- 保留 `--shared --no-checkout` 的隔离本地 Git 克隆，并采用可移植的源路径传递，避免将 Windows 路径误解析为远程传输 URL。
- 在当前和历史 fixture 检出中保留 `.ai` 治理产物的实际字节，包括 `core.autocrlf=true` 的情况。
- digest 与字节相等断言保持严格。单字节改动仍必须被拒绝；不得通过归一化报告内容来掩盖检出转换。
- 在活动 successor handoff 中增加独立版本化、只读的交付连续性章节。它验证绑定的 predecessor archive 与 recovery lineage，区分本地归档和待完成的外部验证，并且只显示 Runtime 准入的下一动作。lineage 缺失或无效时保持 unknown；`OutcomeDelivery.deliveryState` 仍表示 handoff 传输状态。
- 只读材料计划只有在验证归档回执、请求、源码 revision、repository、Contract lineage 以及每项 path/kind/cause/hunk/blob 身份后，才能显示 predecessor 先前审阅过的精确 Unknown 条目。这只是参考上下文：不会清除当前 Unknown、转移决定或满足 successor 自身的审阅请求。
- 修改限于活动 Contract 声明的 27 条路径。不改 CI workflow、全局 Git 设置或 WI-1111 归档记录。

## 验证计划

增加聚焦回归，覆盖含空格的本地仓库路径、启用 autocrlf 时的当前及固定历史字节，以及单字节证据变更后的拒绝。还需覆盖已验证/损坏的 predecessor lineage、归档字节保留、材料身份变化，以及 predecessor 上下文不得清除当前 Unknown。先运行 repository、CLI、MCP 定向测试，再执行 Contract 声明的 workspace 验证和零警告 Clippy。只有后续获准的运行在最终交付 HEAD 上完成后，才能宣称 Windows 托管 CI 已验证。

## 当前状态

此 successor 从已验证并归档的 WI-1111 恢复创建；其 Windows repository-library job 已失败。Windows 字节保真测试、已验证 lineage 的连续性测试、材料参考身份测试及 CLI/MCP successor handoff 一致性测试均已通过；相关 repository material-review（41/41）、CLI material-review（4/4）、MCP material-review（2/2）和 Outcome report（29/29）测试通过。完整 Contract 验证（15/15）和零警告 Clippy 曾在 Linux 本地通过，测试源码 HEAD 为 `bc284474c7f83407b8d7e7dda67dc0fb2de97cda`，早于本次仅更新状态的文档提交。固定 Runtime 将当前文档提交后的快照标记为 stale，因此仍需重新验证。hosted Windows CI 仍待执行，不宣称 Windows hosted 已通过。前任归档和失败的 hosted run 保持不变。
