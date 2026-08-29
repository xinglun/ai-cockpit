---
author: AI Cockpit maintainers
title: WI-402 —— Rust Runtime 性能极限
description: 在不削弱治理事实的前提下测量并降低可避免的 Rust Runtime 成本。
workItemId: WI-402-rust-performance-extreme
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-402-rust-performance-extreme
terminalArchive: .ai/work-items/archive/WI-402-rust-performance-extreme.contract.json
terminalVerification: .ai/evidence/WI-402-rust-performance-extreme.verification.json
terminalFinalization: .ai/decisions/WI-402-rust-performance-extreme.finalize.json
terminalDecision: .ai/decisions/WI-402-rust-performance-extreme.close.json
---

# WI-402 —— Rust Runtime 性能极限

本 Work Item 面向 Cockpit 工程和附加的对象工程优化共享 Rust Runtime。
这是有测量依据的优化，不改变治理语义：验证强度、证据身份、fail-closed
行为、按请求绑定的 Repository Context 和确定性的人工 Outcome 仍然是权威事实。

## 已交付边界

- 精确验证复用会忽略 shell/mise/Agent 会话 bookkeeping，但保留 `PATH`、
  `PWD`、`TMPDIR`、`CARGO_HOME`、`RUSTFLAGS` 等命令和工具链输入。
- 源内容身份排除 Runtime 生成的 `.ai/` receipt，同时保留已跟踪源文件和非
  `.ai` 工作树变更。因此治理 receipt 不会使自己的复用结果失效，源代码变化仍会失效。
- 复用仅适用于 profile-authorized 且全身份一致的验证。显式自定义命令保持 fresh，
  任一不匹配都会执行声明的检查。
- 回归测试覆盖会话元数据、源内容身份稳定性，以及首次执行/第二次精确复用。

## 对象工程继承

优化写入共享外部 binary，不复制到对象工程。每个 repository 保留自己的 `.ai/`
证据；升级到发布版 Runtime 后才继承相同规则。每个验证上下文仍记录 Runtime
版本/digest 和 repository identity。

## 验证

Work Item evidence 记录了定向 Rust 测试、完整 workspace 质量检查和发布/对象工程验收。
耗时只是 advisory evidence，不能降低必需的 Verification Tier 或 Evidence Assurance。

### 本地测量（建议性）

2026-08-29 在 macOS arm64 上，对同一个微型附加仓库执行 10 次运行，比较已安装
的 v0.2.40 binary 与候选 release profile。热调用 P95 耗时变化为：`inspect`
72.561 ms → 72.217 ms（-0.5%）、`status` 95.573 ms → 94.500 ms（-1.1%）、
`doctor` 16.636 ms → 13.828 ms（-16.9%）、`observe` 73.057 ms → 71.957 ms
（-1.5%）。这些是本地进程耗时观察，不是 provider 或企业保证；候选 binary
在仓库外测量，不能作为公开发布验收 artifact。
