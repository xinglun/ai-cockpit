---
author: AI Cockpit maintainers
title: "WI-212——WI-211 收尾恢复"
description: "恢复不可变的 WI-211 归档，并补回必需的 PR 资源收尾顺序。"
audience:
  - maintainer
  - reviewer
workItemId: WI-212-release-fixture-finalization-recovery
status: implemented
authority: canonical
lastVerifiedBy: WI-212-release-fixture-finalization-recovery
---

# WI-212——WI-211 收尾恢复

WI-211 在绑定 PR #160 之前就完成了 verification 和 archive。已安装 Runtime 正确拒绝了
之后的 finalization，因为 verification evidence 已经写入。本 successor 保留 WI-211 的
不可变 recovered history，不改写其 bytes，并补回缺失的资源收尾边界。

## 验收

1. 通过严格的 successor recovery 回执绑定 WI-211，且 WI-211 保持不可变。
2. 在本 successor verification 前绑定 PR #160 的资源上下文；合并前回执保持
   `awaiting_merge_close`。
3. WI-211 recovered、WI-212 等待合并关闭时，治理 gates 通过。
4. 只有 hosted merge、准确清理、finalize-verify 和结构化人工决定完成后才关闭 WI-212。

## 不在范围内

改写 WI-211 记录、移动 v0.2.26、参考源逐文件对比，以及用户全局 Agent/MCP 配置均不在范围内。
