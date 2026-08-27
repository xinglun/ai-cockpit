---
author: AI Cockpit maintainers
title: "WI-313——close 后 finalization reconciliation"
workItemId: WI-313-post-close-finalization-reconciliation
description: "强制先清理再 close，并为不可变的历史 close 记录提供严格绑定的恢复路径。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-313-post-close-finalization-reconciliation
---

# WI-313——close 后 finalization reconciliation

## 意图与边界

W312 暴露了真实的顺序缺口：旧 Runtime 可能在 provider finalization 仍为
`retained` 时写入 close，随后 closed-document promotion gate 正确拒绝宣称清理完成。
本 Work Item 修复 Runtime 边界并保留历史字节。新的 Work Item 必须在 close 前清理
provider 资源；只有不可变的历史 close 才允许在之后追加一条绑定的 deleted transition。

## 范围与验收

Rust protocol/repository lifecycle 在 close 时拒绝 retained、blocked、unknown
finalization；close 后 transition 只有在绑定 closed root digest、Work Item/repository
identity、下一 sequence 以及 branch/worktree 已删除的精确状态时才可接受。close 与原始
finalization 字节保持不变。文档 promotion gate 与三语 workflow 说明正常路径和历史路径，
并拒绝所有未绑定或不完整的例外。

## 验证

必须通过 Rust finalization 定向测试、closed-document promotion fixture、格式化、lint、
workspace 测试和 repository 文档门禁。最终 evidence 记录 hosted CI 与已安装 Runtime
identity；发布验收不以源码构建作为替代。
