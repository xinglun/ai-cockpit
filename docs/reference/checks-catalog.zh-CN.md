---
author: AI Cockpit maintainers
title: "检查目录"
description: "带有明确证据边界的工程质量与治理检查。"
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-331-checks-release-evidence
keywords: [ai-cockpit, checks, governance, verification]
---

# 检查目录

本目录描述当前 Rust 工程实际提供的检查，保留参考源对本地质量检查、Work
Item 治理门、托管 provider 证据和企业 assurance 的区分。它不是参考源 Make
目标或 Python 执行器的复制品。

## 检查层次

| 层次 | 目标入口 | 可以证明 | 不能证明 |
| --- | --- | --- | --- |
| Runtime Contract 门 | `ai-cockpit gate --repo <path> --manifest tests/ci/repository_gate_manifest.json --stage <stage>` | 当前 Contract、仓库快照、路由和门禁清单内部一致。 | 不执行托管 CI，也不授予企业 assurance。 |
| 本地 workspace 质量 | `cargo fmt --all -- --check`；`cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` | 当前 workspace 的 Rust 格式和 lint 结果。 | 本地通过不是 reviewed PR 或发布结果。 |
| 包验证 | `tests/ci/run_workspace_package_tests.sh --report <path>` | 确定性的包测试覆盖及其 receipt。 | 不证明 provider 分支保护或发布。 |
| Conformance 与文档 | `tests/conformance/reference_file_inventory_test.sh`；`tests/docs/documentation_acceptance.sh` | 参考清单、读者路线和文档不变量。 | 文档文字不能代替可执行证据。 |
| 发布与 adopter | 严格 manifest 路由中的 `tests/release/*` | 制品身份、校验和、SBOM/provenance 绑定，以及命名 harness 实际运行的隔离 adopter 生命周期。 | staged 或本地结果，除非 provider receipt 明确记录，否则不是公开 Release 证据。 |

规范集合和 profile 下限版本化保存在
`tests/ci/repository_gate_manifest.json`。路由是累积的：`light` 覆盖文档和低成本
策略检查，`standard` 增加 Rust workspace 与 conformance 检查，`strict` 再增加发布、
workflow、性能和 adopter 检查。变更路径、Contract 风险和生命周期阶段共同选择最低
profile。未知输入或发布所属路径会升级到 `strict`；调用方不能通过传入更快的命令降低
已选择的 profile。

`VerificationTier`（执行检查的强度）和 `EvidenceAssurance`（谁可以为结果背书）是
正交的。严格的本地检查不会自动成为 provider-verified 或 enterprise-verified。

## 证据所有权

Runtime receipt 绑定仓库、Work Item、Contract、快照、选定路由和 Runtime 身份。托管
CI 负责 provider run/job 结论以及外部分支或合并观察。公开 Release 负责已发布归档、
校验和、SBOM、provenance 和 attestation 事实。企业系统负责身份、保留、WORM/SIEM 和
组织审批。AI Cockpit 可以要求、绑定、校验、展示和归档委托证据，但不会伪造这些外部
声明。

所有检查都服从当前 Contract、preflight review、必要场景证据、人工决定和 reviewed PR
生命周期。本地检查绿色只是有用证据，不是跳过必要门禁或宣称生产就绪的授权。

## 失败与恢复

缺失、损坏、过期、外部仓库、或相互矛盾的 receipt 都会 fail closed。保留失败命令、
源 revision、输出 receipt 以及 provider run 身份用于诊断。修复有界原因后重新运行指定
检查；不要用未固定的命令或源码构建的 Runtime 替换失败结果。对象工程 adopter 提供
自己的技术栈命令，而每次 AI Cockpit 调用仍必须显式绑定 `--repo <path>`。
