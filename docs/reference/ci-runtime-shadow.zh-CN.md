---
author: AI Cockpit maintainers
title: CI Runtime Verification Shadow
description: 在保留现有 Cargo 质量门的同时使用不可变公开 Runtime 做 Phase 1 CI 收敛。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-145-ci-runtime-shadow
---

# CI Runtime Verification Shadow

WI-145 建立 CI 收敛的 Phase 1。quality job 下载上一稳定且公开不可变的 `v0.2.15` Linux
Runtime，校验 archive 与 binary digest，然后让 `ai-cockpit verify` 对当前 checkout
执行验证。receipt 固定记录 tag、version、archive digest、binary digest、platform、
download source 和 Runtime verify 结果。

当前安装基线可以推进到更新的 Release（当前为 `v0.2.16`），而不改变发布前的 shadow pin。
只有在该 Release 公开并记录不可变 archive/binary identity 后才推进 pin，避免 tag workflow 依赖尚不存在的 artifact。

现有 Cargo `fmt`、`clippy` 与 package test 步骤仍保留在同一个 job 中，作为独立的
shadow comparison。Runtime shadow 通过不代表替换或弱化这些检查；本阶段也不宣称
Runtime 与 Cargo 结果已经等价，更不提供 provider/enterprise assurance。

收敛边界分阶段固定：

1. **Phase 1（当前）：** 不可变 Runtime verify 加现有 Cargo checks。
2. **Phase 2（后续）：** 长期收集 Runtime/Cargo 可比较结果并证明稳定收敛。
3. **Phase 3（后续）：** 只有 Phase 2 有证据且迁移决定经过 review 后，才删除重复的
   YAML policy。

Shadow lane 拒绝源码构建、workspace binary、未固定版本的 release artifact、
archive/binary digest 不一致以及格式错误的 Runtime 输出。
