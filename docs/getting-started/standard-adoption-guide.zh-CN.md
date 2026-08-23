---
author: AI Cockpit maintainers
title: "标准采用指南"
description: "从已校验 Runtime 到首个 closed Work Item 的完整读者路线。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# 标准采用指南

按顺序完成以下阶段；每个阶段都有独立证据边界：

1. [安装](installation.zh-CN.md)不可变公开 Runtime，并验证准确制品。
2. 显式[检查并 attach](30-second-start.zh-CN.md)目标 repository。
3. 以一条 owner 批准的质量命令完成[首次校准](first-calibration.zh-CN.md)。
4. 完成[采用方配置](adopter-configuration.zh-CN.md)中的审查、安全、恢复与 CI owner 清单。
5. 如有需要，显式安装一个 repository-local Agent adapter，并用 `agent doctor` 校验。
6. 在专用 branch/worktree 与受审查 PR 上运行[首个 Work Item](first-work-item.zh-CN.md)。
7. archive 前展示 human Outcome；merge 后验证准确资源清理，并记录 structured human close decision。

不得把安装、attach、profile 确认、实现、provider 审查和 close 折叠成一次隐式批准。
一个边界通过不能证明下一个边界。证据 Unknown 或矛盾时，只阻断依赖它的声明，并写清 owner
与 recovery condition。

Release 信任与 private mirror 限制见[安全与 Release 验证](security-release-verification.zh-CN.md)。
平台示例说明如何保留 Unknown，而不发明工程事实。

[快速开始](README.zh-CN.md) | [English](standard-adoption-guide.md) | [日本語](standard-adoption-guide.ja.md)
