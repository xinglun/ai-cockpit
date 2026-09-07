---
author: AI Cockpit maintainers
title: WI-644——v0.2.86 发布
description: 完成参考源逐文件比对后发布 Runtime，并验证公开产物边界。
audience: [adopter, maintainer, reviewer]
workItemId: WI-644-release-v0-2-86
status: implemented
authority: human-authorized
lastVerifiedBy: WI-644-release-v0-2-86
terminalArchive: .ai/work-items/archive/WI-644-release-v0-2-86.contract.json
terminalVerification: .ai/evidence/WI-644-release-v0-2-86.verification.json
terminalFinalization: .ai/decisions/WI-644-release-v0-2-86.finalize.json
terminalDecision: .ai/decisions/WI-644-release-v0-2-86.close.json
capabilityClaims: [release_distribution, reference_comparison, adopter_acceptance]
---

# WI-644——v0.2.86 发布

[English](WI-644-release-v0-2-86.md) · [日本語](WI-644-release-v0-2-86.ja.md)

## 目标

在六个参考源比对批次完成后发布最终 Runtime。发布将一个经过评审的提交绑定到版本元数据、校验和、SBOM/来源证明、安装说明，以及基于下载产物的 adopter 验收。

## 边界

本 Work Item 只更新发布和版本投影，不复制参考源实现或 wire format。公开发布和 N-1 adopter 证据只能由发布工作流基于不可变公开产物生成。对象工程仍是共享 Runtime 的独立使用者。

## 验证

在 PR 评审前，workspace 测试、发布策略、源归档、Action runtime、adopter wrapper、5,175 条参考清单、三语文档和治理完整性检查均通过。公开 tag 与发布后的验收由发布工作流提供供应商绑定证据。

权威 Contract、verification、finalization 和 close 证据见页首链接的终态记录。
