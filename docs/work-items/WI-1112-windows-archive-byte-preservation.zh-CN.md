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
- 产品修改仅限 `.gitattributes` 与 `crates/cockpit-repository/src/lib.rs`，并同步本页的英、日、中版本及三份 reference-parity 台账。不改 CI workflow、全局 Git 设置或 WI-1111 的归档记录。

## 验证计划

增加聚焦回归，覆盖含空格的本地仓库路径、启用 autocrlf 时的固定历史字节，以及单字节证据变更后的拒绝。先运行 repository 定向测试，再执行 Contract 声明的 workspace 验证和零警告 Clippy。只有后续获准的运行在最终交付 HEAD 上完成后，才能宣称 Windows 托管 CI 已验证。

## 当前状态

此 successor 从已验证并归档的 WI-1111 恢复创建；其 Windows repository-library job 已失败。目前尚未声称实现或验证结果。前任归档和失败的 hosted run 保持不变。
