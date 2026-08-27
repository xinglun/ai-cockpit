---
author: AI Cockpit maintainers
title: CI Runtime Verification Shadow
description: 使用类型化仓库质量路由，并以不可变公开 Runtime 执行 shadow 验证。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-224-ci-reference-parity
---

# CI Runtime Verification Shadow

WI-224 把仓库 CI 路由变成显式策略。`quality_route.py` 根据 changed paths、Contract
risk 与 workflow stage 选择 `light`、`standard` 或 `strict`。未知路径、release-owned
路径、高风险、merge 与 release stage 都升级为 `strict`。类型化 route receipt 绑定
Git base/head、changed paths、Contract 路径与 digest、manifest byte digest、选择原因和
有序 gate ID。`run_repository_gates.py` 会从当前仓库事实重算 receipt，并且只执行规范
manifest 中的命令；不存在任意命令 override。

Runtime shadow 与 Contract 绑定：`standard` 或 `strict` 的 pull request 只有在初始路由
解析出一个 active Contract 时才执行并上传 shadow。已经 finish/archive、没有 active
Contract 的 PR 仍执行普通仓库 gates，但会跳过这个仅用于执行身份的 shadow，因为不可变
Runtime 没有当前 Contract 就不能产生 Work Item verification evidence。这是明确的跳过，
不会削弱所选仓库 gates，也不会把缺失证据当作通过。

profile 为累加关系。`light` 执行文档与治理策略回归；`standard` 再加入 Cargo fmt、
Clippy、package gates、不可变 Runtime shadow 与源码 conformance；`strict` 继续加入
release、workflow、performance、adopter 与 source-archive gates。Pull request 使用
path/risk route；merge push 的 stage floor 是 strict。release source quality 始终显式
请求 `strict`，并上传 route receipt 与 gate report。

CI 使用两个有边界的 route plan。initial receipt 决定是否需要 Runtime shadow；`light`
会跳过 shadow。`standard` 或 `strict` 先运行 shadow，再用同一个不可变 Git base/head 和
所有 Runtime 仓库本地写入（包括 `.ai/evidence/reuse/**`）重算 final receipt。gate runner
只消费 final receipt，同时保留两个 receipt 供诊断。最终 profile 非 light 时必须生成
workspace package coverage，并且只在其 regular receipt 文件存在时上传；合法的 `light`
route 既不要求也不上传该文件。

在 `standard` 和 `strict` 中，独立 execution shadow 下载公开且不可变的 `v0.2.28`
Runtime，验证各平台 archive/binary digest，再使用仓库规范 profile 执行验证。receipt
绑定 tag、version、archive digest、binary digest、platform、download source 与
Runtime 结果。它拒绝源码构建、workspace binary、任意 `--command` 替代、未固定制品、
digest 不一致和格式错误输出。

这是仓库 CI/release 层策略，不宣称 Runtime 全局 T0–T3 路由、affected graph 完整性、
跨 Work Item 物理执行或通用 CLI `verify --command` 语义；WI-224 未授权 `crates/**`，
这些 Runtime 改动明确 deferred。shadow 只证明执行身份，不能替代所选 manifest gates，
也不提供 provider/enterprise assurance。
