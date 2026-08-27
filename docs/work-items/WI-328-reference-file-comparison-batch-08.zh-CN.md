---
author: AI Cockpit maintainers
title: "WI-328——参考源文件对比批次 08"
workItemId: WI-328-reference-file-comparison-batch-08
description: "逐个比对固定参考源的九个校准与能力文档，并登记明确的 Rust 原生边界。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-328-reference-file-comparison-batch-08
terminalArchive: .ai/work-items/archive/WI-328-reference-file-comparison-batch-08.contract.json
terminalVerification: .ai/evidence/WI-328-reference-file-comparison-batch-08.verification.json
terminalFinalization: .ai/decisions/WI-328-reference-file-comparison-batch-08.finalize.json
terminalDecision: .ai/decisions/WI-328-reference-file-comparison-batch-08.close.json
---

# WI-328——参考源文件对比批次 08

## 意图与边界

逐个比对固定参考提交
e5acb677da6621004d96f0ef353c58fe8d3acfbf 中的九个路径。保留采用方可读的
校准和能力事实责任，不复制源 Python、Make、wizard 或 matrix 字节。

共享 Rust Runtime 仍在工程外部安装，所有 repository 请求都必须显式携带
--repo。本批次是文档与 conformance 台账工作，不增加 Runtime 命令，也不宣称
source wire 兼容。

## 逐文件对比

| 参考源路径 | 分类 | Rust/adopter 对应与边界 |
| --- | --- | --- |
| docs/reference/calibration-session-model.md | implemented-different-by-design | 仓库绑定的 profile proposal、人工确认和显式 calibration facts 保留事实/证据边界；不引入通用持久化 Session，也不把 proposal 当作 active policy。 |
| docs/reference/calibration-session-model.zh-CN.md | implemented-different-by-design | 中文读者使用同一 repository-bound proposal/confirm 边界；unknown 和人工责任保持可见。 |
| docs/reference/calibration-session.ja.md | implemented-different-by-design | 源十阶段 interactive Session 仅由目标的显式 profile proposal/confirm 语义承接；不复制 Make/Python 或 enterprise/security 声明。 |
| docs/reference/calibration-session.md | implemented-different-by-design | 源持久化十阶段 wizard 是源专属编排；目标校准保持 read-only-first、repository-bound，并要求人工确认策略变更。 |
| docs/reference/canonical-terminology.md | implemented-different-by-design | .ai/glossary.md、configuration 和 Outcome 参考页提供 canonical terms；治理 light 不暗中等同于校准 lite，release 是 operation 而不是 profile。 |
| docs/reference/capability-claim-authoring.md | reference-only | 源 lexical claim checker 与 matrix front matter 绑定不是目标 Runtime gate。目标 registry 只报告观察到的仓库事实和显式 exclusions；能力声明/证据边界作为候选 WI-329 跟进。 |
| docs/reference/capability-evidence-freshness.md | reference-only | Rust 校验 Work Item verification freshness 和 identity-bound receipt，但没有独立的 Capability Truth 行过期或 portable-environment matrix；扩展由候选 WI-329 负责。 |
| docs/reference/capability-truth-matrix.json | reference-only | 不复制源三十行 public matrix。capability_truth_registry 是 request-scoped 的观察能力 projection，不是 public claim authorization，也不是 adopter/provider proof。 |
| docs/reference/capability-truth-matrix.md | reference-only | 当前 capability/adoption 页面说明 observed fact、adopter installation、provider evidence 和 enterprise assurance 边界；在后续有界工作获批准前，不宣传源 matrix 或 claim checker。 |

四个 reference-only 项是明确的产品边界，不是未登记遗漏。候选 WI-329 不在本
批次启动；它需要人类拥有的 scope，用于 Rust 原生 claim/evidence matrix、
freshness policy、三语绑定检查及采用方文档。不会复制源 Python/Make checker。

## Cursor 采用方反馈对照

Cursor 报告是外部采用方证据，不是新的 source wire 合同。当前 Runtime 证据已经覆盖：

- lifecycle stdout 的稳定 JSON，以及可重放的 work-item outcome；
- close-before-next 入口检查和显式 readyOnBase；
- dirty 或未同步 base 时 fail-closed 的 start 检查；
- 相关变更后的 verification 失效；以及
- 仓库本地 Agent adapter 的显式安装，不自动向聊天发布。

Runtime 不能强制 IDE 展开聊天面板；adapter/host 必须展示或重放持久化 human
handoff。更详细的 mismatch remediation、controls 脚手架和 close-gap 便利命令
属于后续产品工作，本批不静默宣称已实现。本项目也有意不要求 Makefile.ai；
显式 --repo 的 CLI/MCP 是对象工程接口。

## 非目标

本 Work Item 不增加 Runtime 行为，不复制源 Python/Make/YAML，不引入通用校准
wizard，不增加 public claim matrix，不强制 Make 集成，不修改全局 Agent/MCP
配置，也不改变固定的 source/target commit。

## 验收与证据

1. 九个 pinned 路径均已阅读，每个路径恰有一条非空且有证据支持的 inventory 记录。
2. inventory 为本 WI 登记五条 implemented-different-by-design 和四条
   reference-only，不留 deferred 或隐藏分类。
3. 英文、简体中文、日文 comparison/parity 页面与本 Work Item 在 source pin、
   分类、Cursor 边界和候选 WI-329 上一致。
4. 目标不宣称通用十阶段 Session、源 Python/Make 执行、public capability-claim
   matrix、provider identity 或 enterprise assurance，除非有目标证据。
5. 安装 Runtime 的 inspect/status/doctor/agent doctor、文档/conformance 检查、
   lifecycle 收尾、hosted CI 和精确清理提供终态证据；不重写历史 evidence。

[English](WI-328-reference-file-comparison-batch-08.md) · [日本語](WI-328-reference-file-comparison-batch-08.ja.md)
