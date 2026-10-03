---
author: AI Cockpit maintainers
workItemId: WI-1056-retry-amend-preflight
title: 恢复重试与 Contract 修订的 preflight 候选修复
description: 保留 Runtime start 原始拒绝和待审请求生成的本地候选测试证据。
audience: [maintainer, reviewer]
status: in-progress
authority: user-authorized-bounded-governance-repair-exception
lastVerifiedBy: focused-local-tests-only
---

[English](WI-1056-retry-amend-preflight.md) · [日本語](WI-1056-retry-amend-preflight.ja.md)

# WI-1056 有界治理修复例外

本 WI 在 `codex/wi1056-retry-amend-preflight` 上从 `origin/main`
`a5bd06bf2932072eaef1ce1c33edcd8068b9d39b` 开始。正式 `work-item new`
仅创建了 `not_ready` 骨架；`start` 因已归档 WI-1042、WI-1043 的 scope
冲突而被拒绝。按用户先前的有界治理修复例外，仅在独立分支进行候选测试和代码修复；
不能称为 Runtime 已准入、已验证或已关闭。

修复只针对有效的 `recoveryRetryPending=true`、敏感 Contract 修订及
`preflightState=not_run` 同时出现时的自锁。新的 CLI 测试在改动前因缺少
`run_preflight` 而 RED，最小状态投影改动后 GREEN。测试核对新待审请求绑定
当前 repository、WI、Contract 与 snapshot，未写人工决定，retry 标记保留，
且验证命令未启动。普通敏感修订测试和 repository 库测试也通过；这些只是本地
候选证据，不是本 WI 的 Runtime verification receipt。独立审查及后续生命周期未完成。

另一次真实使用未替换共享 Runtime：Task 9 的 WI-1049 通过一次正式修订保留 A1
`development_cycle_cost` 两条路径，只新增 A2 `runtime_benchmark_scenarios` 的 Rust
实现／测试两条路径及定向验收；B `runtime_benchmark_stats` 不在该批次。随后仅运行一次
正式 preflight，对 Contract
`sha256:7de2a941e89fbeb2cef36f5a9c016a580f76ea71420a55aaafcdcba6572866df`
及 repository snapshot
`sha256:fe207ccca471f47df7b27a61adf09901e9df622100c68d45730fc941dbd86cdd`
返回 `needs_human_confirmation`。retry 仍待处理，没有记录人工决定，也未继续写 Task 9
源码。新增回归还核对错误 repository、修订前 Contract、错误 snapshot 的审阅证据均被
拒绝且不写入决定。这只能证明待审请求可生成，不能证明人工批准或 WI-1056 正式准入。
