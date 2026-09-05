---
author: AI Cockpit maintainers
title: "WI-591——v0.2.77 发布与对象工程恢复交接"
description: "发布包含 predecessor close 再验证修复的 Runtime，并校验公开产物。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-591-release-v0-2-77
lastVerifiedBy: WI-591-release-v0-2-77
terminalArchive: .ai/work-items/archive/WI-591-release-v0-2-77.contract.json
terminalVerification: .ai/evidence/WI-591-release-v0-2-77.verification.json
terminalFinalization: .ai/decisions/WI-591-release-v0-2-77.finalize.5e0a83694b8c0cc446f933fdfec8909a4fe84d4bcceb1e55ba03d5a2fbe6e7aa.json
terminalDecision: .ai/decisions/WI-591-release-v0-2-77.close.json
---

[English](WI-591-release-v0-2-77.md) · [日本語](WI-591-release-v0-2-77.ja.md)

# WI-591——v0.2.77 发布与对象工程恢复交接

## 目标

从已审查且同步的默认分支发布 v0.2.77。该版本包含 WI-589 的
Contract amendment predecessor-close revalidation 修复，并保留不可变发布证据，
为对象工程提供只读验收交接。

## 边界

本 Work Item 只修改包版本元数据和发布文档。Runtime 实现、对象工程、全局
Agent/MCP 配置、历史证据字节及参考源实现均不在范围内。公开 adopter 和 N-1
验收属于发布后证据，只能使用下载的不可变产物，不得使用源码 checkout 或
workspace 构建。

## 验收

1. Cargo 元数据、锁文件和中英日发布/版本文档标识 v0.2.77，并保留 v0.2.76
   作为前一个公开基线。
2. 发布策略检查证明 annotated tag、五目标产物、校验和、SBOM/来源证明及
   Runtime identity 绑定到同一个源提交。
3. 发布后 adopter 与 N-1 harness 只使用公开 v0.2.77 产物，证明禁止写入根和
   临时运行目录清理。
4. 不修改对象工程；发布后向对象工程团队提供准确的兼容性、恢复和再验证命令。
5. 发布或 adopter 失败保留已发布事实并记录失败 receipt；不重写失败 tag 或历史证据。

## 验证

执行 Contract 中列出的发布策略、文档、Parity 和 locked workspace 检查。发布后
使用 v0.2.77 运行公开 adopter 与 N-1 验收 harness，并保存不可变 receipt。
