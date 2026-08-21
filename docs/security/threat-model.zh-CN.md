---
author: AI Cockpit maintainers
title: "威胁模型"
description: "共享 Runtime 与 repository Protocol 的信任假设、保护资产和 fail-closed 威胁。"
audience:
  - adopter
  - reviewer
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - threat_model
---

# 威胁模型

## 资产

保护对象包括 repository identity、Work Item scope/authority、verification output、reusable
receipt、archive history、Runtime identity 和 Agent adapter ownership。`.ai/` 是 repository-local
state；安装的 Runtime 是共享代码，不能创建全局 current repository。

## 信任边界

- Human request 和 Work Item Contract 是声明输入，不是证明。
- 仓库文件、日志、依赖指令和 provider message 在变成 typed facts 前都是不可信 material。
- Verification 在 Runtime 的有界控制内执行，但 Runtime 不声称是通用操作系统 sandbox。
- 外部 CI、identity、签名、SBOM、provenance、SIEM、WORM 和企业审批仍由外部证据或保留系统负责。

## 威胁与响应

Scope 扩大、权限缺失、过期或跨 Work Item evidence、仓库/日志 prompt injection、测试削弱、危险删除、
receipt 篡改、路径穿越、symlink、超大 store 数据和 executable identity 漂移会 fail closed 或要求 fresh
run。措辞本身不能授予 capability；Raw Request Binding 必须声明 operation、scope、authority 和 evidence。

模型不声称能识别所有恶意意图，只验证由请求和 evidence schema 表示的确定性边界。
