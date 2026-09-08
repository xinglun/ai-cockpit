---
author: AI Cockpit maintainers
title: WI-650 — 验证子进程改为阻塞等待
description: 将固定10ms轮询等待在Unix上替换为阻塞等待，并给出测量过的前后对比。
workItemId: WI-650-verification-wait-blocking
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-650-verification-wait-blocking
terminalArchive: .ai/work-items/archive/WI-650-verification-wait-blocking.contract.json
terminalVerification: .ai/evidence/WI-650-verification-wait-blocking.verification.json
terminalFinalization: .ai/decisions/WI-650-verification-wait-blocking.finalize.json
terminalDecision: .ai/decisions/WI-650-verification-wait-blocking.close.json
---

# WI-650 — 验证子进程改为阻塞等待

本 Work Item 是 AI Cockpit 性能优化专项的 P1：消除 P0 前期调查中发现的固定间隔
忙轮询等待（`crates/cockpit-verification/src/lib.rs` 中 `execute_captured`
的子进程等待循环）。

## 发现并测量到的缺陷

此前每个验证子进程的等待循环是：

```rust
let deadline = Instant::now() + Duration::from_secs(MAX_EXECUTION_SECONDS);
let (status, mut timed_out) = loop {
    match child.try_wait() {
        Ok(Some(status)) => break (Some(status), false),
        Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
        Ok(None) => { terminate_process_tree(&mut child, child_id); break (child.wait().ok(), true); }
        Err(_) => { terminate_process_tree(&mut child, child_id); break (None, false); }
    }
};
```

在改动任何生产代码之前，先用一个独立的微基准测试（直接、在紧凑循环中启动
`/bin/true` 和 `sleep 1`，绕开 CLI 以消除无关的启动/git/身份噪声）量化了这个
循环的开销：

| 场景 | 忙轮询 | 阻塞 `wait()` |
|---|---|---|
| `true`（n=200） | 均值 12.192 ms，p50 12.523 ms | 均值 1.188 ms，p50 1.193 ms |
| `sleep 1`（n=5） | 均值 1013.309 ms | 均值 1009.251 ms |

对于近乎瞬时完成的命令，忙轮询循环额外增加了约 11 ms 的纯等待时间（该场景下
延迟膨胀约 10 倍）；对于耗时数秒的命令，两种机制在统计上无法区分——这正好
符合轮询间隔相对权重的预期。

## 修复内容

- 将 `execute_captured` 的等待逻辑抽取为 `wait_for_child(child, child_id,
  deadline)`，提供一个 `#[cfg(unix)]` 实现和一个未改动的 `#[cfg(windows)]`
  实现。
- 在 Unix 上，把子进程 move 进一个专用线程，该线程调用阻塞的 `child.wait()`
  并通过 `mpsc::sync_channel` 发送结果；调用方只做一次基于剩余时间的
  `recv_timeout`。超时时，通过 pid（`Copy` 类型，无需已被 move 走的
  `Child` 值）用 `libc::kill` 杀死进程，子孙进程的终止逻辑与之前完全一致，
  然后仍从 channel 中取得最终的退出状态。这保留了现有的超时、进程树终止和
  输出捕获保证（输出捕获本就使用有界的 `libc::poll`，而非忙等 sleep 循环，
  因此未改动）。
- 在 Windows 上，`wait_for_child` 与之前的循环逐字相同。本 Work Item **未对
  该平台进行验证**（没有 Windows 环境可用）；在未经验证的情况下改动它是
  不合理的风险，因此刻意保持不变。
- `terminate_process_tree`（需要 `&mut Child`，与 Unix 侧已被 move 进线程的
  `Child` 不兼容）现在仅限 `#[cfg(windows)]`。

## 正确性验证

- 三个新增单元测试（`crates/cockpit-verification/src/lib.rs` 中的
  `#[cfg(all(test, unix))] mod wait_for_child_tests`）直接调用
  `wait_for_child`，使用较短的合成截止时间，而不是真实的 300 秒
  `MAX_EXECUTION_SECONDS`（端到端验证不现实）：一个远在截止时间内完成的命令
  （未超时，退出状态正确）、一个超过 100ms 截止时间的命令（`sleep 5`，确认
  在 2 秒内（而非 5 秒）被杀死并报告 `timed_out=true`）、以及退出码的保留
  （`sh -c 'exit 7'` 报告代码 7）。三个测试总共约 0.1 秒完成。
- 现有 `cockpit-verification` 完整测试套件（9 个文件共 56 个测试，包括
  `bounded_execution_reports_plan_and_process_telemetry`、
  `detached_descendant_pipe_is_cancelled_and_fails_closed` 等与执行/超时
  相关的测试）无变化地通过。
- `cargo test --locked --workspace` 无变化地通过（120 个 test result 区块，
  0 个失败）。

## 测量到的性能（参考信息）

2026-09-08，macOS arm64，对一个小型已 attach 的夹具仓库运行
`ai-cockpit verify --repo <fixture> --command true`（一个始终 fresh、不会被
复用的自定义命令），12 次迭代，baseline（已安装的 v0.2.87）对本构建：

- baseline：典型值约 104–108 ms（排除一个 123 ms 的异常值）
- candidate：典型值约 94–98 ms（排除首次调用一个 505 ms 的异常值）

约 10 ms 的端到端改善与独立微基准测试得到的约 11 ms 结果高度吻合，证实这一
改善确实来自被移除的轮询等待，而非测量噪声。以上均为本地进程延迟观测，不
构成 provider 或 enterprise 层面的保证。

## 不在本次范围内

- Windows 的等待路径（未改动，本 Work Item 未对其验证）。
- 有界并行文件读取/哈希（另一个 P1 候选方案；本 Work Item 未涉及）。
- 对输出流捕获、依赖调度或资源预算的任何改动。
