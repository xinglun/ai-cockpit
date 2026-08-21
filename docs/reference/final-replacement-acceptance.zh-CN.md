---
author: AI Cockpit maintainers
title: "最终替代验收"
description: "证明 Rust Runtime 替代参考 Runtime 的可重复验收边界。"
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-82
capabilityClaims:
  - final_replacement_acceptance
---

# 最终替代验收

运行 `tests/conformance/final_replacement_acceptance.sh --repo <repository>`
会生成可审计的验收目录，记录已安装 Runtime 版本和 binary digest、绑定的
repository identity、锁定的参考源 commit、每个 gate 的结果、`acceptance.json`
以及 `SHA256SUMS`。

验收 gate 独立覆盖：锁定参考源 conformance、adversarial 负向语料、带负例拒绝的
性能回归、发布 workflow policy、面向人的 Outcome 输出、可执行参考源 oracle，
以及证明没有复制 V1 Runtime 实现的 tracked-path 检查。

脚本 fail-closed，不调用 `cargo build`、`cargo run`、workspace binary 或本地
`target/` fallback。绿色 receipt 只证明本验收边界通过，不授权合并或发布。
