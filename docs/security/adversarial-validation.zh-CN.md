---
author: AI Cockpit maintainers
title: "对抗性验证"
description: "Fail-closed 安全边界和对抗性验证面。"
audience:
  - reviewer
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - security_validation
---

# 对抗性验证

安全边界采用 fail-closed 和证据驱动原则。conformance corpus 比较语义而不是字符串：
决策状态、阻断项、未知项、安全动作、必需检查、权限和结果状态必须一致。

运行时边界测试还验证仓库文本只作为数据、Work Item ID 不能路径穿越、MCP evidence 路径
必须位于仓库内、验证命令使用 allowlist 和目标 cwd，以及 finish 不能在没有新鲜通过回执时
自我声明完成。

## Verification 与 reuse 的信任边界

在 reusable receipt 满足某个 node 前，runtime 会把候选绑定到 repository snapshot 和 source
range、attached profile/configuration 原始字节、toolchain 和 resolved executable identity、完整
执行环境、command、scope、policy、stage、runner 以及 output identity。Protected node、显式命令
和绑定 Work Item 的 verification 总是 fresh。

Receipt store 会拒绝 symlink 的父目录或文件、malformed 内容、hard-linked commit marker、不确定
的 index commit、未知 schema 字段、超大文件、被篡改的 receipt ID、失败/过期 receipt 和 binding
不一致。任何失败都会变为 unknown 或 rerun，绝不会授权 reuse。Verification 还限制命令时间、
捕获输出和 worker 数量；timeout、descendant 或 capture 失败不能算 pass。

失败或未知的 provider 结果始终不是 green。人类权限可以解决决策要求，但不能伪造验证回执。
