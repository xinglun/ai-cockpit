---
author: AI Cockpit maintainers
title: "WI-206——Release tag 待关闭治理边界"
description: "允许已证明合并关系的 Release tag 发布，同时保留必须由 Runtime 完成的关闭步骤。"
audience:
  - maintainer
  - adopter
workItemId: WI-206-release-tag-pending-close
status: implemented
authority: canonical
lastVerifiedBy: WI-206-release-tag-pending-close
---

# WI-206——Release tag 待关闭治理边界

v0.2.25 的 source-quality gate 正确拒绝了一个已经合并但尚未关闭当前周期
Work Item 的 tag。这暴露了顺序死锁：合并后的 finalization transition 需要
Release 中的 Runtime，但 Release gate 在该 Runtime 能安装之前运行。

本 Work Item 明确定义这个边界。只有当 pre-merge finalization receipt 已绑定身份，
且 Git 证明其中记录的 PR head 是 tag commit 的祖先时，Release tag 才能暂时投影为
`awaiting_merge_close`。发布后的 binary 仍必须完成 finalization 和结构化 human close；
普通分支与无法证明的 tag 继续 fail-closed。

## 验收边界

1. 只有通过祖先证明的 Release tag 才能接受为 `awaiting_merge_close`。
2. 非祖先、格式错误、foreign 以及普通分支场景仍然阻断。
3. 英文、简体中文、日文工作流文档都说明发布顺序与发布后关闭要求。
