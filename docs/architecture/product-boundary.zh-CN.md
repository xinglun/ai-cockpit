# 产品边界

## 产品身份

AI Cockpit 是面向 AI 辅助工程的 Repository Governance Layer。North Star 是
校准的人类—Agent 信任，核心规则是 Evidence over Self-Declaration。

治理链是：

```text
Evidence → Governance Decision → Human Control
```

## 范围内

- 确定性的 repository 观察；
- 受边界约束的 Work Item Contract；
- scope、authority、evidence 和生命周期决策；
- fail-closed verification planning 与 evidence reuse；
- repository 内的事实、决定、证据和 knowledge projection；
- CLI 以及只读/验证型 MCP adapter。

## 明确不属于范围

AI Cockpit 不是 Agent Runtime、Workflow Engine、Security Sandbox、通用
prompt-injection detector、identity provider、合规证书，也不替代人工审查。
Provider identity、branch protection、production isolation、签名、SBOM、
provenance 和企业策略属于外部证据或 adopter 责任。

## 架构约束

- 不能从 binary 路径推断 Runtime root。
- 不能把 runtime 代码复制到对象 repository。
- Repository Protocol version 与 Runtime version 独立。
- MCP 与 CLI 使用同一 application services，不能各自拥有治理规则。
- 人类决定可以解决 workflow 问题，但不能把未验证的 check 变成 pass。

