---
author: AI Cockpit maintainers
title: "CI 与发布证据"
description: "明确所有权的 provider 派生 CI 与公开 Release 证据。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-331-checks-release-evidence
keywords: [ai-cockpit, ci, release, evidence]
---

# CI 与发布证据

CI 和 Release 记录属于委托证据。它们的权威来自托管 provider 和准确的公开制品，而不
来自 PR 正文、Agent 消息或本地“已通过”声明。Rust Runtime 可以绑定并校验这些记录，
但不会冒充 GitHub Actions 或企业审批系统。

## CI 证据

版本化的 `tests/ci/repository_gate_manifest.json` 与 CI 路由绑定仓库、Contract、base
revision、head revision、选定 profile、有序 gate ID 以及路由/清单摘要。最终 gate 报告
记录每个必需 gate 及其结果。托管 adapter 还应保留 provider workflow run、job 名称、job
结论和准确的 head SHA。

必需 job 是明确集合。跳过或失败的 job 仍必须出现在记录中，不能为了让聚合看起来绿色
而省略。聚合结论必须与每个 job 结果和失败原因一致。PR 正文或人的描述不能替代 provider
run，本地 fixture 也不能提升为托管 assurance。

profile 由策略选择且是累积的：`light`、`standard`、`strict` 表示验证覆盖强度，不是
assurance 等级。merge 或 release 阶段有 strict 下限。未知路径和发布所属文件会 fail
closed 到 strict 路由。Rust Contract gate 是 repository-bound 决定的 authority；收敛期间
现有脚本 runner 只是有界执行 shadow。

## Release 证据

发布 workflow 绑定 version、tag、源 commit、Cargo.lock 摘要、目标归档、可执行文件成员、
校验和清单、SBOM 和 provenance。每个目标都必须拥有预期归档布局，并从实际发布字节重新
计算校验和。SBOM 和 attestation subject 必须指向同一源和制品身份。单独的 tag 或上传文件
不是稳定的公开 Release。

Release 证据有明确状态：

| 状态 | 含义 | 授权边界 |
| --- | --- | --- |
| `candidate` | 发布前的 staged 源/制品记录。 | 可支持 review，但不证明公开 Release。 |
| `verified` | 对准确源 commit 且必需 job/制品通过的 provider 证据。 | 可支持发布步骤，仍不是已发布 Release。 |
| `published` | 绑定准确公开 Release 和完整制品集合的 verified 证据。 | 公开发布事实，不是企业认证。 |
| `failed` | provider 或制品检查失败并记录原因。 | 不能授权 `verified` 或 `published`。 |

发布后的 adopter harness 另产生 acceptance receipt，绑定下载的不可变 tag/制品、二进制和
归档摘要、隔离仓库身份、生命周期证据以及清理/隔离 manifest。成功的 adopter receipt 证明
该 binary 治理过那个 adopter；不证明覆盖所有技术栈或所有企业环境。

## 所有权与失败

本地 Runtime 和 manifest 检查是 repository evidence。托管 run/job 结果、合并观察、签名、
SBOM 发布、attestation、分支保护和企业审批仍由外部或 provider 负责。AI Cockpit 在收到
时记录它们的身份、来源、assurance、采集时间、摘要、有效性和原始引用，但不会伪造 provider
结果。

缺失 job、从聚合中隐藏跳过/失败 job、head/base 不一致、制品或 SBOM 摘要错误、校验和重复
或缺失、JSON 损坏，或没有 provider-bound evidence 的发布状态都会 fail closed。保留失败
receipt 和源身份；不要把已经发布的 Release 改写成未发布，也不要把失败 receipt 用于后续
版本。

对象工程 adopter 也遵循同一边界：共享 Runtime 在工程外，仓库状态隔离在 `.ai/`，每次
调用都使用显式 `--repo <path>`。
